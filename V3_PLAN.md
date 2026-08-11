# ManDown — Free-Tier Hosting Plan (GCP Cloud Run + Firestore)

> Goal: move ManDown off the always-on GCE `e2-micro` VM onto **GCP serverless free tier**, by (1) splitting the single Rust binary into two binaries — a **webhook bot** and a **one-shot poller** — and (2) switching Telegram from long polling to **webhooks**.

---

## 1. Current state (what we are changing)

ManDown is one Rust binary (`man_down`, `Cargo.toml` v2.1.6) whose `src/main.rs` does three things in one process:

```22:36:src/main.rs
#[tokio::main]
async fn main() {
    dotenv().ok();
    init_logger();

    let collection = init_mongo().await;
    let http_client = cust_client(30);
    let bot = Bot::from_env();

    log::info!("Bot started");

    start_downtime_checker(bot.clone(), collection.clone(), http_client.clone());

    start_command(bot.clone(), collection.clone(), http_client.clone()).await;
}
```

- `start_downtime_checker` (`src/poll.rs`) `tokio::spawn`s a task with an **infinite `loop` + `tokio::time::sleep(FREQ)`** that sweeps all tracked sites and alerts on status change.
- `start_command` (`src/command.rs`) runs `Command::repl` — `teloxide` **long polling** for Telegram updates.

State lives in **Firestore (MongoDB-compatible API)**, collection `mandown.websites`, accessed via the `mongodb` crate and `MONGODB_URI`. Infra is Terraform: GCE VM + Firestore + Artifact Registry.

### Why the current shape doesn't fit serverless

| Problem | Detail |
|---|---|
| **Infinite poll loop** | Cloud Run scales to zero between requests; an in-process `loop { sleep }` dies on cold-down and never restarts on its own. |
| **Long polling** | `teloxide` `Command::repl` holds a long-lived getUpdates connection. Cloud Run requests have a max timeout and scale-to-zero semantics — long polling fights the platform. |
| **Single-instance** | Two replicas would both poll and double-fire alerts. Cloud Run can scale to >1; we must pin `max-instances=1` and use a singleton scheduler. |

### Target shape

Split into a **Cargo workspace** of three crates — one shared lib (`core`) and two binaries — each a natural fit for serverless:

| Binary | Trigger | Lifetime | Cloud Run shape |
|---|---|---|---|
| `man_down_bot` | Telegram **webhook** POST | Handle one update, return 200 | Service, `min-instances=0`, `max-instances=1`, `concurrency=1` |
| `man_down_poller` | **Cloud Scheduler** HTTP POST every `FREQ` seconds | Run **one** sweep, return 200, exit | Service, `min-instances=0`, `max-instances=1`, invoked by Scheduler |

Firestore, the `mongodb` crate, Artifact Registry, and most of the Rust code are **unchanged**.

---

## 2. Splitting the binary — Cargo workspace (3 crates)

Convert the single-binary crate into a **Cargo workspace** with one shared library crate and two binary crates. Each binary compiles only the dependencies it actually needs (the poller never links `axum` or teloxide's webhook feature; the bot never links the poll-loop logic). There is no combined dev binary — local dev runs `cargo run -p man_down_bot` (webhook mode, pointed at `ngrok`) or `cargo run -p man_down_poller` (one sweep).

> **Built in two phases (see Section 7):** Phase 1 creates the workspace with `crates/core` + `crates/poller` only; Phase 2 adds `crates/bot` as the third member. The end-state layout below shows all three.

### 2.1 Workspace layout

```
Cargo.toml                  # workspace root (no [dependencies])
crates/
├── core/                   # shared lib: model, DB init, HTTP client, alert formatter, logger
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── mongo.rs        # Website struct + init_mongo (shared)
│       ├── http.rs        # cust_client + HttpClient trait + get_status_code (shared)
│       ├── alert.rs       # process() (shared)
│       └── config.rs      # init_logger (shared)
├── bot/                    # webhook bot (prod)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── command.rs      # bot-only
│       ├── handler.rs     # bot-only
│       ├── format.rs      # bot-only
│       ├── parse_url.rs   # bot-only
│       └── mongo.rs       # put_site, get_user_websites, clear/delete (bot-only)
└── poller/                 # one-shot poller (prod)
    ├── Cargo.toml
    └── src/
        ├── main.rs
        ├── poll.rs         # run_once() (poller-only)
        ├── baseline.rs    # poller-only
        ├── alert.rs       # alert_users() (poller-only)
        ├── http.rs        # get_status(), find_changed_websites() (poller-only)
        ├── mongo.rs       # get_sites(), update_db() (poller-only)
        └── config.rs      # Config (baseline_sites loader) (poller-only)
```

### 2.2 What's shared vs. bot-only vs. poller-only

| Module | Location | Why |
|---|---|---|
| `mongo::Website` struct, `mongo::init_mongo` | `core` | Both crates init the same Firestore connection and use the same document model |
| `http::cust_client`, `http::HttpClient` trait, `http::get_status_code` | `core` | Bot uses `get_status_code` for `/track` validation; poller uses it for the sweep |
| `alert::process` | `core` | Status→message formatter; bot calls it in `handle_track`, poller calls it in `alert_users` |
| `config::init_logger` | `core` | Both binaries bootstrap logging identically |
| `command`, `handler`, `format`, `parse_url` | `bot` | Telegram command parsing + `/track` `/list` `/clear` `/untrack` only |
| `mongo::put_site`, `get_user_websites`, `clear_user_websites`, `delete_sites_by_hostname` | `bot` | User-facing writes/reads only the bot performs |
| `axum`, `teloxide` `webhook` feature | `bot` | Only the bot runs an HTTP listener |
| `poll`, `baseline` | `poller` | Sweep loop + baseline connectivity check |
| `mongo::get_sites`, `mongo::update_db` | `poller` | Paginated reads + status updates only the poller performs |
| `http::get_status` (retry), `http::find_changed_websites` | `poller` | Sweep-only helpers |
| `alert::alert_users` | `poller` | Sends alerts to users |
| `config::Config` (loads `baseline_sites` from YAML) | `poller` | Only the poller reads `config.yaml` |

> The existing modules keep their `crate::` internal paths — they resolve correctly inside whichever crate they live in. The binaries reference shared code as `man_down_core::{...}`.

### 2.3 Workspace root `Cargo.toml`

Replace the current single-crate `Cargo.toml` with:

```toml
[workspace]
members  = ["crates/core", "crates/bot", "crates/poller"]
resolver = "2"

[workspace.package]
version = "2.1.6"
edition  = "2024"
```

Keeping the version in `[workspace.package]` lets all three crates share one version number; bump it once and CI updates everywhere.

### 2.4 `crates/core/Cargo.toml`

```toml
[package]
name = "man_down_core"
version.workspace = true
edition.workspace  = true

[dependencies]
async-trait = "0.1"
chrono = "0.4"
log = "0.4"
mongodb = "3.8.0"
reqwest = { version = "0.11.4", features = ["json"] }
serde = "1.0.130"
serde_derive = "1.0.130"
serde_yaml = "0.9"
url = "2.2"
```

### 2.5 `crates/bot/Cargo.toml`

```toml
[package]
name = "man_down_bot"
version.workspace = true
edition.workspace  = true

[dependencies]
man_down_core = { path = "../core" }
async-trait = "0.1"
chrono = "0.4"
dotenvy = "0.15"
env_logger = "0.11"
futures = "0.3.28"
log = "0.4"
mongodb = "3.8.0"
reqwest = { version = "0.11.4", features = ["json"] }
serde = "1.0.130"
teloxide = { version = "0.12", features = ["auto-send", "macros", "webhook"] }
tokio = { version = "1.8.3", features = ["full"] }
url = "2.2"
axum = "0.7"
```

> Verify the exact `webhook` feature name against the teloxide 0.12 docs; if it's named differently in your lockfile, adjust accordingly. `axum` is what teloxide's webhook listener is built on.

### 2.6 `crates/poller/Cargo.toml`

```toml
[package]
name = "man_down_poller"
version.workspace = true
edition.workspace  = true

[dependencies]
man_down_core = { path = "../core" }
chrono = "0.4"
dotenvy = "0.15"
env_logger = "0.11"
futures = "0.3.28"
log = "0.4"
mongodb = "3.8.0"
reqwest = { version = "0.11.4", features = ["json"] }
serde_yaml = "0.9"
teloxide = { version = "0.12", features = ["auto-send", "macros"] }
tokio = { version = "1.8.3", features = ["full"] }
```

Note: **no `axum`, no `webhook` feature** — the poller only sends messages via `bot.send_message`, it never runs an HTTP server. This keeps the poller binary substantially smaller.

### 2.7 `crates/core/src/lib.rs`

```rust
pub mod alert;
pub mod config;
pub mod http;
pub mod mongo;
```

Only the four genuinely shared modules live here. `alert` exposes `process()` (shared); `alert_users()` moves to the poller crate. `mongo` exposes the `Website` struct + `init_mongo()` (shared); the user-facing and sweep-specific query functions move to their respective binary crates.

### 2.8 `crates/poller/src/main.rs` (new, prod)

```rust
use man_down_core::{config::init_logger, http::cust_client, mongo::init_mongo};
use teloxide::prelude::*;

mod alert;       // alert_users() — poller-only
mod baseline;
mod config;      // Config (baseline_sites loader) — poller-only
mod http;        // get_status(), find_changed_websites() — poller-only
mod mongo;       // get_sites(), update_db() — poller-only
mod poll;        // run_once()

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    init_logger();

    let collection = init_mongo().await;
    let http_client = cust_client(30);
    let bot = Bot::from_env();

    poll::run_once(&collection, bot, http_client).await;
    // exit 0 — Cloud Run scales the instance back to zero
}
```

`poll::run_once()` is the extracted single iteration (no loop, no sleep):

```rust
// crates/poller/src/poll.rs (sketch)
pub async fn run_once(
    collection: &Collection<Document>,
    bot: Bot,
    client: Arc<reqwest::Client>,
) {
    log::info!("Starting downtime check");
    let changed_websites = get_changed_sites(collection, client.clone()).await;
    log::info!("Found {} changed websites", changed_websites.len());
    handle_changed_websites(collection, bot, &changed_websites).await;
}
```

### 2.9 `crates/bot/src/main.rs` (new, prod) — webhook mode

Switch from `Command::repl` (long polling) to a webhook listener. The command-handling logic (`answer()`) is reused as-is — only the dispatch source changes.

```rust
use man_down_core::{config::init_logger, http::cust_client, mongo::init_mongo};
use std::sync::Arc;
use teloxide::prelude::*;

mod command;    // answer() + webhook dispatch handler
mod format;
mod handler;
mod mongo;      // put_site, get_user_websites, clear_user_websites, delete_sites_by_hostname
mod parse_url;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    init_logger();

    let collection = init_mongo().await;
    let http_client = cust_client(30);
    let bot = Bot::from_env();

    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "8080".parse().unwrap());
    let public_url = std::env::var("WEBHOOK_URL").expect("WEBHOOK_URL must be set");

    // Register the webhook with Telegram on startup
    bot.set_webhook(public_url.clone()).await.expect("setWebhook failed");

    // Build the teloxide webhook listener (axum-based)
    let (listener, stop_flag) = teloxide::update_listener::webhooks::axum(
        bot.clone(),
        ([0, 0, 0, 0], port).into(),
        public_url,
    )
    .await
    .expect("Failed to build webhook listener");

    Dispatcher::builder(bot, command::update_handler(collection, http_client))
        .build()
        .dispatch_with_listener(listener, stop_flag)
        .await;
}
```

> The exact `teloxide::update_listener::webhooks::axum` signature and `dispatch_with_listener` API should be verified against the teloxide 0.12 docs / examples — the shape above is the idiomatic pattern, but minor method names may differ in your version. The key point: replace `Command::repl` with a webhook `UpdateListener` and reuse the existing `answer()` logic as the per-update handler.

### 2.10 `crates/bot/src/command.rs` — expose a handler for webhook dispatch

Today `start_command` owns the `Command::repl` loop. Extract the per-update logic so the webhook dispatcher can call it. `Command::repl` internally parses `/cmd args` from a `Message`; in webhook mode we do the same parse on each incoming `Update`:

```rust
// crates/bot/src/command.rs (refactored, sketch)
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

// Reusable handler used by the webhook dispatcher
pub async fn handle_update(
    bot: Bot,
    update: Update,
    collection: Arc<Collection<Document>>,
    client: Arc<reqwest::Client>,
) -> ResponseResult<()> {
    // parse the command from the update and dispatch to answer()
    // (teloxide's Command::parse on an Update does this)
    ...
}
```

The existing `answer()` function is unchanged — only the loop around it is removed. There is no long-polling `start_command` in prod; local dev runs the bot in webhook mode (point `WEBHOOK_URL` at an `ngrok` tunnel) or runs the poller for a single sweep.

### 2.11 Dockerfiles — two independent images, one per binary

Each binary gets its **own self-contained Dockerfile** next to its crate (`crates/poller/Dockerfile`, `crates/bot/Dockerfile`). Each builds the whole workspace and ships only its own binary. This keeps the two images fully independent (the bot image can later grow `axum`/CA-cert runtime deps without touching the poller) at the cost of compiling the dependency tree twice in CI — acceptable since CI/registry layer caching absorbs most of it on subsequent builds.

`crates/poller/Dockerfile`:

```dockerfile
FROM rust:alpine AS builder
RUN apk update && apk add --no-cache pkgconfig musl-dev openssl-dev
ENV RUSTFLAGS='-C target-feature=-crt-static'
WORKDIR /build
RUN rustup component add rustfmt clippy

ENV USER=appuser
ENV UID=10001
RUN adduser --disabled-password --gecos "" --home "/nonexistent" \
    --shell "/sbin/nologin" --no-create-home --uid "${UID}" "${USER}"

COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY config.yaml ./

# Tests + clippy are gated in CI (pr-test.yaml); image build only emits the release binary.
RUN cargo fmt --all -- --check
RUN cargo build --release

FROM alpine
RUN apk update && apk add --no-cache libgcc openssl
COPY --from=builder /build/target/release/man_down_poller /mandown_poller
COPY --from=builder /build/config.yaml ./
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group  /etc/group
USER appuser:appuser
ENTRYPOINT [ "/mandown_poller" ]
```

`crates/bot/Dockerfile` is identical except it copies `/mandown_bot` and **no `config.yaml`** (the webhook bot does not read it; baseline is poller-only).

Build each with its own `-f`:

```bash
podman build -f crates/poller/Dockerfile -t <registry>/mandown-poller:$TAG .
podman build -f crates/bot/Dockerfile    -t <registry>/mandown-bot:$TAG .
```

`.dockerignore` excludes `**/Dockerfile` so the Dockerfiles themselves aren't copied into the image. Phase 1 builds and deploys only the poller image; Phase 2 adds the bot image build + deploy.

---

## 3. Switching Telegram from long polling to webhooks

### 3.1 What changes conceptually

| Today (long polling) | After (webhook) |
|---|---|
| Bot opens a long-lived `getUpdates` connection to Telegram | Bot registers a URL with `setWebhook`; Telegram POSTs each `Update` to that URL |
| `Command::repl` blocks the main task | An `axum` HTTP server receives POSTs; each request handles one update |
| Works behind NAT / no inbound ports | Requires a public HTTPS endpoint (Cloud Run provides this) |
| One process must always be running | The bot service can scale to zero between user messages |

### 3.2 The webhook URL

Cloud Run gives each service a stable HTTPS URL of the form:

```
https://mandown-bot-<hash>-<region>.a.run.app
```

The bot sets this as the Telegram webhook on startup (`bot.set_webhook(public_url)`). Pass it via the `WEBHOOK_URL` env var (set in Terraform / Cloud Run). The path defaults to `/` for teloxide's webhook listener; if you want a specific path, append it to `WEBHOOK_URL` and configure the listener accordingly.

### 3.3 One-time registration

`setWebhook` is idempotent — calling it on every boot is fine and is the simplest approach (the bot re-registers on each cold start). Alternatively, register once out-of-band:

```bash
curl "https://api.telegram.org/bot<TOKEN>/setWebhook?url=https://mandown-bot-<hash>-<region>.a.run.app/"
```

To stop receiving updates (e.g. during cutover), call `deleteWebhook`.

### 3.4 Security considerations

- **Source IP filtering:** Telegram publishes its webhook IP ranges. Cloud Run does not natively support IP allow-listing on ingress, so rely on the secret-token mechanism instead: set a `secret_token` in `setWebhook` and have the bot verify the `X-Telegram-Bot-Api-Secret-Token` header on each request. teloxide's webhook listener supports this.
- **Ingress:** Set Cloud Run ingress to `INGRESS_TRAFFIC_ALL` (Telegram's source IPs are broad and rotating). The secret token is the real auth.
- **No PII change:** The bot still stores only `url`, `status`, `last_updated`, `telegram_id` — no new data is collected.

### 3.5 Local development

There is no combined dev binary. Run the bot and poller separately:

- **Bot:** `cargo run -p man_down_bot` in webhook mode. Point `WEBHOOK_URL` at an `ngrok http 8080` tunnel so Telegram can reach your local machine.
- **Poller:** `cargo run -p man_down_poller` runs **one** sweep and exits — run it manually whenever you want to test the sweep locally.

---

## 4. Architecture on GCP (after)

```
                 ┌──────────────────────────────────────────────┐
                 │                   GCP Project                │
                 │                                              │
   Telegram ───▶ │  Cloud Run service: mandown-bot              │
   webhook POST  │   - image: Artifact Registry (man_down_bot)  │
                 │   - min-instances=0, max-instances=1          │
                 │   - concurrency=1                            │
                 │   - env: TELOXIDE_TOKEN, MONGODB_URI,         │
                 │           WEBHOOK_URL, PORT, FREQ(unused)     │
                 │   - receives /webhook from Telegram          │
                 │   - handles /track, /list, etc.               │
                 │                                              │
                 │   Firestore (MongoDB-compatible API)          │
                 │   - existing DB mandown/websites (unchanged)  │
                 │                                              │
   Cloud         │  Cloud Scheduler: mandown-poll               │
   Scheduler ──▶ │   - schedule: "*/10 * * * *" (or FREQ-based) │
   (HTTP POST)   │   - target: mandown-poller Cloud Run URL      │
                 │   - OIDC auth token for the Scheduler SA      │
                 │                                              │
                 │  Cloud Run service: mandown-poller           │
                 │   - same image, entrypoint /mandown_poller   │
                 │   - min-instances=0, max-instances=1          │
                 │   - on POST: run_one sweep, return 200        │
                 │   - env: TELOXIDE_TOKEN, MONGODB_URI          │
                 │                                              │
                 │  Artifact Registry (existing, unchanged)      │
                 │  Secret Manager: TELOXIDE_TOKEN, MONGODB_URI │
                 └──────────────────────────────────────────────┘
```

### 4.1 Why this is free

| Resource | Free tier | Expected usage |
|---|---|---|
| Cloud Run requests | 2M / month | < 10K (Telegram updates + 144 poller runs/day) |
| Cloud Run vCPU-seconds | 360K / month | Bot cold-starts only; poller small — see note |
| Cloud Run GiB-seconds | 180K / month | 256MiB × runtime — comfortable |
| Firestore reads | 50K / day | < 1K |
| Firestore writes | 20K / day | < 500 |
| Firestore storage | 1 GB | < 10 MB |
| Cloud Scheduler jobs | 3 / month | 1 |
| Artifact Registry | 0.5 GB | < 100 MB |
| Secret Manager | 6 secret versions / month | 2 |
| VPC egress (NA) | 200 GB / month | < 5 GB |

> **Note on vCPU budget:** The poller is the cost driver. Keep its CPU allocation small (`--cpu=0.25`), keep `FREQ` realistic (10 min = 144 runs/day), and ensure the poller **exits as soon as the sweep is done**. With a 256MiB / 0.25 vCPU job that runs ~30s per sweep, monthly billable vCPU-seconds ≈ 144 × 30 × 0.25 ≈ **1.1K** — far under the 360K free quota.

---

## 5. Terraform changes (`infra/`)

Terraform is applied in two phases matching the migration. The VM (`infra/compute.tf`) is **kept in Phase 1** and removed in Phase 2.

### 5.1 Phase 1 — new files (VM untouched)

Phase 1 adds four new files and leaves `compute.tf`, `firestore.tf`, `repository.tf`, `provider.tf`, `vars.tf` unchanged.

**`infra/iam.tf`** — two service accounts:
- `man-down-run` — the Cloud Run revisions' runtime SA (reads Secret Manager secrets).
- `man-down-scheduler` — the Cloud Scheduler SA (invokes the poller Cloud Run service via OIDC).

**`infra/secrets.tf`** — Secret Manager secrets `mandown-teloxide-token` and `mandown-mongodb-uri`, with `google_secret_manager_secret_version` resources that write the sensitive values from the existing `var.teloxide_token` / `var.mongodb_uri` Terraform variables (same apply inputs as the old VM env), plus `roles/secretmanager.secretAccessor` bindings for the runtime SA.

**`infra/cloudrun.tf`** — the `mandown-poller` Cloud Run service only (the bot service is added in Phase 2):
- `command = ["/mandown_poller"]`, `container_concurrency = 1`, `timeout_seconds = 300`
- env `TELOXIDE_TOKEN` and `MONGODB_URI` pulled from Secret Manager
- `service_account_email = google_service_account.runtime.email`
- `lifecycle.ignore_changes = [image]` so CI can roll the image without Terraform fighting it
- IAM: `roles/run.invoker` for the scheduler SA only — **no public ingress** (only Cloud Scheduler may call the poller)

**`infra/scheduler.tf`** — one `google_cloud_scheduler_job` (`*/10 * * * *`) with an `http_target` POST to the poller Cloud Run URL, authenticated via an OIDC token for the scheduler SA.

### 5.2 Phase 2 — additions and deletion

- **Add** the `mandown-bot` Cloud Run service to `infra/cloudrun.tf` (public ingress — `allUsers` invoker — for Telegram webhooks; `WEBHOOK_URL` + `PORT` env; `axum` listener on `8080`).
- **Add** `webhook_url` to `infra/vars.tf`.
- **Delete** `infra/compute.tf` (the GCE VM) and `terraform apply`.

### 5.3 Scaling to zero (free tier)

The Cloud Run services default to `min-instances=0`. Do **not** set `min_instance_count=1` on either service — that would make them always-billed (~$5/mo each). Cold starts (~1–2s for the Rust binary) are acceptable for a personal bot.

---

## 6. GitHub Actions changes

### 6.1 `.github/workflows/build-push.yaml`

The `Update prod image` step (currently `gcloud compute instances update-container`) is replaced phase-by-phase:

**Phase 1** — build and deploy only the poller image (the VM is untouched, so no `update-container`):

```bash
# Build the poller image from its own Dockerfile
podman build -f crates/poller/Dockerfile -t ${{ vars.REPOSITORY_URI }}/mandown-poller:$TAG .
# Push + deploy
gcloud run deploy mandown-poller \
  --image ${{ vars.REPOSITORY_URI }}/mandown-poller:$TAG \
  --region ${{ vars.GCP_REGION }} \
  --platform managed --quiet
```

**Phase 2** — additionally build and deploy the bot image:

```bash
podman build -f crates/bot/Dockerfile -t ${{ vars.REPOSITORY_URI }}/mandown-bot:$TAG .
gcloud run deploy mandown-bot \
  --image ${{ vars.REPOSITORY_URI }}/mandown-bot:$TAG \
  --region ${{ vars.GCP_REGION }} \
  --platform managed --quiet
```

The two images are fully separate: each contains only its own binary (the poller image also ships `config.yaml`; the bot image does not). New GitHub variable required in Phase 1: `GCP_REGION` (e.g. `us-east1`). Phase 2 also needs `WEBHOOK_URL`. The existing `GCP_PROJECT_ID`, `GCP_WIF`, `REGISTRY_URI`, `REPOSITORY_URI`, `IMAGE`, `FREQ` are reused. `GCP_ZONE` stays in use by `terraform.yaml` (the VM still exists in Phase 1) and is dropped in Phase 2.

### 6.2 `.github/workflows/terraform.yaml`

Unchanged in shape — it already passes `TF_VAR_teloxide_token` and `TF_VAR_mongodb_uri` from GitHub secrets, which `infra/secrets.tf` now consumes to populate Secret Manager. The apply inputs do not change. On PRs the plan will show the new Cloud Run/Scheduler/Secret resources being created (Phase 1) and the VM being destroyed (Phase 2).

### 6.3 `.github/workflows/pr-test.yaml`

Unchanged. `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` now build all workspace members (core + bot + poller); tests are unaffected.

---

## 7. Migration — phased rollout

The migration is split into **two phases** so the poller can be validated in production before touching the bot. The GCE VM keeps serving users throughout Phase 1; it is only torn down in Phase 2, immediately after the webhook bot takes over.

> **Note on overlap:** during Phase 1 the VM's in-process poll loop and the new Cloud Run poller will both run, so users may receive duplicate alerts until Phase 2 completes. This is accepted — no bridge patch or VM silencing is needed, which keeps Phase 1 purely additive.

---

### 7.1 Phase 1 — poller + scheduler (low risk, additive)

**Goal:** deploy the poller on Cloud Run + Cloud Scheduler and validate it in prod for several days. The VM continues to run unchanged (long-polling bot + its own poll loop) the entire time.

1. **Back up Firestore** — `gcloud firestore export gs://state-mandown/backup-$(date +%s)`.
2. **Restructure the repo into a workspace** with two members: `crates/core` + `crates/poller` (the `crates/bot` member is added in Phase 2). Move shared modules into `core`, poller-only modules into `poller`, per Section 2. Extract `poll::run_once()`. Verify `cargo build --release -p man_down_poller` succeeds.
3. **Deploy Phase-1 Terraform** (alongside the VM): `infra/cloudrun.tf` (poller service only), `infra/scheduler.tf`, `infra/secrets.tf`. Do **not** touch `compute.tf`. Create the Secret Manager secrets + versions for `TELOXIDE_TOKEN` and `MONGODB_URI`.
4. **Build + push the poller image**, deploy to the `mandown-poller` Cloud Run service.
5. **Smoke test the poller** — `curl -X POST <poller-url>` once manually; confirm in Cloud Run logs that it sweeps all sites, detects status changes, and sends Telegram alerts.
6. **Enable the Cloud Scheduler job** (`*/10 * * * *`). Confirm it fires on schedule.
7. **Observe for 3–7 days.** Confirm: the Cloud Run poller fires reliably, Firestore quota stays in free tier, Scheduler job success rate is 100%. (Duplicate alerts from the VM's poller are expected during this window and stop in Phase 2.) Fix anything that surfaces before moving to Phase 2.

**Exit criteria for Phase 1:** the Cloud Run poller + Scheduler are validated and reliable. The VM is untouched.

---

### 7.2 Phase 2 — webhook bot + VM teardown (fast, coordinated cutover)

**Goal:** replace the VM's long-polling bot with the Cloud Run webhook bot, then destroy the VM. This phase is done in a single short window because the cutover is atomic: once `setWebhook` is called for the production token, Telegram stops sending updates to the VM's long-polling connection immediately — which also ends the VM's duplicate poller alerts.

1. **Add `crates/bot` to the workspace** (3rd member). Implement the webhook listener, refactor `command.rs` to expose `handle_update()`, move bot-only modules into the crate, per Section 2. Verify `cargo build --release` produces `man_down_bot`.
2. **Deploy Phase-2 Terraform**: add the `mandown-bot` Cloud Run service to `infra/cloudrun.tf` (+ IAM for unauthenticated ingress). The VM is still running.
3. **Build + push the image** (now contains both `man_down_bot` and `man_down_poller`), deploy the bot service to Cloud Run.
4. **Smoke test with a staging bot token** (create a second bot via BotFather, set its webhook to the Cloud Run bot URL). Send `/track example.com`, `/list`, `/untrack example.com` — verify Firestore gets the right documents and responses come from Cloud Run, not the VM.
5. **Cutover (the atomic step):** call `setWebhook` for the **production** token, pointing at the Cloud Run bot URL. Within seconds the VM's `getUpdates` long-poll returns no more updates — Telegram delivers to exactly one consumer (webhook wins over polling). The VM is now inert, and its poll loop's duplicate alerts stop too.
6. **Verify the production bot** — send `/list` from a real Telegram account; confirm the response comes from Cloud Run (check logs) and the VM logs show no new updates.
7. **Destroy the VM** — remove `infra/compute.tf` and `terraform apply` (or `terraform destroy -target=google_compute_instance.mandown`).
8. **Monitor for one week** — Cloud Run logs for both services, Firestore quota, Scheduler job success rate, alert delivery.

**Exit criteria for Phase 2:** the VM is gone, both Cloud Run services serve all traffic, free-tier quotas hold, no duplicate alerts.

---

### 7.3 Why phasing is safer

| Risk | Big-bang cutover | Phased |
|---|---|---|
| Poller bug doubles or misses alerts | Discovered during cutover, users affected | Discovered in Phase 1 over days, VM still serving |
| Webhook misconfig drops bot updates | Bot down until fixed | Bot never down — VM serves until `setWebhook` succeeds |
| VM teardown window | Must happen same day as webhook deploy | Decoupled — VM stays as long as you want |
| Rollback | Revert image + re-poll Telegram | Phase 1 rollback = disable the Scheduler job; Phase 2 rollback = `deleteWebhook` (VM long polling resumes) |

Phase 2 is intentionally a short, coordinated window (steps 5–7 are minutes apart), but it only happens after Phase 1 has de-risked the harder half (the poller). The accepted cost is duplicate alerts during Phase 1, which end the moment Phase 2's `setWebhook` cutover makes the VM inert.

---

## 8. Risks / watch-outs

- **Webhook HTTPS cert** — Cloud Run provides a valid TLS cert on its default URL. Don't use a custom domain unless you also manage the cert.
- **Cold-start latency** — `min-instances=0` adds ~1–2s to the first `/track` after idle. Acceptable for a personal bot. If not, set `min-instances=1` (≈ $5/mo, no longer free).
- **Poller overlap** — Cloud Scheduler could fire the next run before the previous finishes. With `FREQ=600s` and a ~30s sweep this is unlikely, but guard with a Firestore "lock" doc: CAS on `last_poll_started_at`, skip if `now - last < FREQ - epsilon`.
- **Poller as service vs. job** — this plan uses a Cloud Run **service** invoked by HTTP POST (simplest). A Cloud Run **Job** is also possible but requires the Scheduler to call the Jobs execute API with OIDC, which is more plumbing. The HTTP-service approach is recommended.
- **teloxide webhook API** — verify the exact `webhooks::axum` and `dispatch_with_listener` signatures against teloxide 0.12 docs before finalizing `crates/bot/src/main.rs`.
- **Secret rotation** — `MONGODB_URI` and `TELOXIDE_TOKEN` move from GCE container env to Secret Manager. Update the GitHub Actions secrets accordingly (or keep them as WIF-injected env at deploy time).
- **`ENV` and `FREQ` on the bot** — `ENV` is unused in code; `FREQ` is only read by the poll loop, which the bot no longer runs. The poller reads `FREQ` only if you keep the loop in `run_once` — you don't, so `FREQ` becomes a Terraform/Scheduler-only concept (the cron schedule replaces it).

---

## 9. Effort estimate

### Phase 1 — poller + scheduler

| Task | Hours |
|---|---|
| Workspace split: `crates/core` + `crates/poller`, extract `run_once()`, move modules | ~2.5 |
| Dockerfile update (build workspace, ship poller binary) | ~0.5 |
| Terraform: Cloud Run poller service + Scheduler + Secrets (VM untouched) | ~1.5 |
| GitHub Actions: poller deploy step | ~0.5 |
| Smoke test + enable Scheduler + 3–7 day observation | ~1 (active) |
| **Phase 1 subtotal** | **~6** |

### Phase 2 — webhook bot + VM teardown

| Task | Hours |
|---|---|
| Add `crates/bot` to workspace, webhook listener, `command.rs` refactor, move bot-only modules | ~2.5 |
| Terraform: add `mandown-bot` Cloud Run service + IAM | ~1 |
| GitHub Actions: bot deploy step + `WEBHOOK_URL` var | ~0.5 |
| Staging-token smoke test + atomic cutover (`setWebhook`) + VM destroy | ~1.5 |
| One-week monitor | ~0.5 (active) |
| **Phase 2 subtotal** | **~6** |

| | |
|---|---|
| **Total** | **~12** |

The accepted cost of skipping a bridge patch is duplicate alerts to users during the Phase 1 observation window (a few days). That ends the moment Phase 2's `setWebhook` cutover makes the VM inert.

---

## 10. Summary

- **Cargo workspace, three crates (built in two phases)**: `man_down_core` (shared: `Website` model, `init_mongo`, HTTP client, `process()` formatter, logger), `man_down_poller` (Phase 1, one-shot sweep — no `axum`, no webhook feature), `man_down_bot` (Phase 2, webhook — pulls `axum` + teloxide `webhook`). No combined dev binary; local dev runs each crate separately.
- **Telegram**: long polling → webhooks via teloxide's `webhook` feature + `axum`; Cloud Run's HTTPS URL is the webhook target.
- **Poller**: in-process `loop { sleep }` → Cloud Scheduler HTTP trigger; the poller runs one sweep and exits.
- **State**: Firestore + `mongodb` crate, completely unchanged (Enterprise edition + MongoDB-compatible API stays; fits the Enterprise daily free tier).
- **Phased rollout**: Phase 1 deploys the poller + Scheduler and validates it for several days while the VM keeps running unchanged (duplicate alerts during the overlap are accepted — no bridge patch needed). Phase 2 deploys the webhook bot, atomically cuts over with `setWebhook` (which also ends the VM's duplicate alerts), and tears down the VM in the same short window.
- **Infra**: VM destroyed in Phase 2; two Cloud Run services + one Cloud Scheduler job + Secret Manager secrets added across both phases; Firestore and Artifact Registry kept.
- **Free tier**: stays within Cloud Run / Firestore / Scheduler free quotas with comfortable headroom, as long as `min-instances=0` on both services.
