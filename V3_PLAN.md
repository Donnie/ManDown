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

Split into **two binaries** that share the same crate (lib + bins), each a natural fit for serverless:

| Binary | Trigger | Lifetime | Cloud Run shape |
|---|---|---|---|
| `man_down_bot` | Telegram **webhook** POST | Handle one update, return 200 | Service, `min-instances=0`, `max-instances=1`, `concurrency=1` |
| `man_down_poller` | **Cloud Scheduler** HTTP POST every `FREQ` seconds | Run **one** sweep, return 200, exit | Service, `min-instances=0`, `max-instances=1`, invoked by Scheduler |

Firestore, the `mongodb` crate, Artifact Registry, and most of the Rust code are **unchanged**.

---

## 2. Splitting the binary

### 2.1 Crate layout: lib + three bins

Convert the single-binary crate into a **library + binaries** so the bins share `src/poll.rs`, `src/mongo.rs`, `src/http.rs`, etc. without duplicating code.

Proposed layout:

```
src/
├── lib.rs            # NEW: declares pub mod ...; re-exports shared API
├── main.rs           # KEPT: local-dev combined mode (long polling + loop), unchanged
├── bin/
│   ├── bot.rs        # NEW: webhook bot entrypoint (prod)
│   └── poller.rs     # NEW: one-shot poller entrypoint (prod)
├── alert.rs          # unchanged
├── baseline.rs       # unchanged
├── command.rs        # refactored: expose handler for webhook dispatch
├── config.rs         # unchanged
├── format.rs         # unchanged
├── handler.rs        # unchanged
├── http.rs           # unchanged
├── mongo.rs          # unchanged
├── parse_url.rs      # unchanged
└── poll.rs           # refactored: extract run_once()
```

### 2.2 `Cargo.toml` changes

Add a `[lib]` and explicit `[[bin]]` targets, and the `webhook` feature for `teloxide` plus `axum` for the HTTP listener:

```toml
[lib]
name = "man_down"
path = "src/lib.rs"

[[bin]]
name = "man_down"          # local-dev combined binary (kept for `cargo run`)
path = "src/main.rs"

[[bin]]
name = "man_down_bot"      # prod webhook bot
path = "src/bin/bot.rs"

[[bin]]
name = "man_down_poller"   # prod one-shot poller
path = "src/bin/poller.rs"
```

Dependencies diff:

```toml
teloxide = { version = "0.12", features = ["auto-send", "macros", "webhook"] }
axum = "0.7"
```

> Verify the exact `webhook` feature name against the teloxide 0.12 docs; if the feature is named differently in your lockfile, adjust accordingly. The `axum` integration is what teloxide's webhook listener is built on.

### 2.3 `src/lib.rs` (new)

Re-export the modules so binaries can `use man_down::{...}`:

```rust
pub mod alert;
pub mod baseline;
pub mod command;
pub mod config;
pub mod format;
pub mod handler;
pub mod http;
pub mod mongo;
pub mod parse_url;
pub mod poll;
```

The existing modules keep their `crate::` internal paths — those resolve correctly inside the lib crate. The only changes are: `main.rs` and the two new bins reference `man_down::` instead of `crate::`.

### 2.4 `src/poll.rs` — extract `run_once()`

Today `downtime_check` is a `loop` that calls the sweep then sleeps. Extract one iteration so the poller binary can call it once and exit. The existing `start_downtime_checker` (used by `main.rs` for local dev) is kept unchanged.

```rust
// src/poll.rs (refactored, sketch)
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

`downtime_check` becomes `loop { run_once(...).await; sleep(interval).await; }` — same behavior, no functional change for local dev.

### 2.5 `src/bin/poller.rs` (new, prod)

```rust
use man_down::{config::init_logger, http::cust_client, mongo::init_mongo, poll::run_once};
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    init_logger();

    let collection = init_mongo().await;
    let http_client = cust_client(30);
    let bot = Bot::from_env();

    run_once(&collection, bot, http_client).await;
    // exit 0 — Cloud Run scales the instance back to zero
}
```

No loop, no sleep. Cloud Scheduler invokes this service every `FREQ` seconds.

### 2.6 `src/bin/bot.rs` (new, prod) — webhook mode

Switch from `Command::repl` (long polling) to a webhook listener. The command-handling logic in `src/command.rs` (`answer()`) is reused as-is — only the dispatch source changes.

```rust
use man_down::command;
use man_down::config::init_logger;
use man_down::http::cust_client;
use man_down::mongo::init_mongo;
use std::sync::Arc;
use teloxide::prelude::*;

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

> The exact `teloxide::update_listener::webhooks::axum` signature and the `dispatch_with_listener` API should be verified against the teloxide 0.12 docs / examples — the shape above is the idiomatic pattern, but minor method names may differ in your version. The key point: replace `Command::repl` with a webhook `UpdateListener` and reuse the existing `answer()` logic as the per-update handler.

### 2.7 `src/command.rs` — expose a handler for webhook dispatch

Today `start_command` owns the `Command::repl` loop. Extract the per-update logic so the webhook dispatcher can call it. `Command::repl` internally parses `/cmd args` from a `Message`; in webhook mode we do the same parse on each incoming `Update`:

```rust
// src/command.rs (refactored, sketch)
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

// Reusable handler used by both long-poll (main.rs) and webhook (bin/bot.rs)
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

The existing `answer()` function is unchanged — only the loop around it changes. `start_command` (used by `main.rs` for local dev) stays as the `Command::repl` wrapper.

### 2.8 `src/main.rs` — unchanged (local dev)

Keep `src/main.rs` exactly as it is so `cargo run` and `cargo run --bin man_down` still work for local development with long polling + the in-process poll loop. Only the prod deployment uses the two new binaries.

### 2.9 Dockerfile — build and ship both prod binaries

The current Dockerfile builds one binary (`man_down`). Change the final stage to copy both prod binaries and default to the bot:

```dockerfile
FROM alpine
RUN apk update && apk add --no-cache libgcc openssl

COPY --from=builder /build/target/release/man_down_bot    /mandown_bot
COPY --from=builder /build/target/release/man_down_poller /mandown_poller
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group  /etc/group
COPY config.yaml ./

USER appuser:appuser

# Cloud Run sets PORT and invokes the container; default to the bot.
ENV ENTRYPOINT_BIN=/mandown_bot
ENTRYPOINT [ "sh", "-c", "exec ${ENTRYPOINT_BIN}" ]
```

The build stage already runs `cargo build --release`, which produces all three binaries. The poller Cloud Run service overrides the entrypoint to `/mandown_poller`.

> Continued in next section.

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

Keep `src/main.rs` (long polling + loop) for local dev so you don't need a public URL or ngrok. Only prod runs `man_down_bot` in webhook mode. If you want to test webhooks locally, use `ngrok http 8080` and point `WEBHOOK_URL` at the ngrok tunnel.

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

### 5.1 Delete

- `infra/compute.tf` — the `google_compute_instance` + `container-vm` module (the GCE VM).

### 5.2 Keep unchanged

- `infra/firestore.tf` — Firestore DB + MongoDB-compatible indexes (reused as-is).
- `infra/repository.tf` — Artifact Registry.
- `infra/provider.tf` — provider + GCS backend.
- `infra/vars.tf` — keep `app_name`, `project_id`, `region`; drop `zone` (Cloud Run is regional, not zonal); keep `freq`, `image`, `teloxide_token`, `mongodb_uri`; add `webhook_url`.

### 5.3 New: `infra/cloudrun.tf`

```hcl
# Bot service (webhook receiver)
resource "google_cloud_run_service" "bot" {
  name     = "${var.app_name}-bot"
  location = var.region

  template {
    spec {
      container_concurrency = 1
      containers {
        image = var.image
        ports { container_port = 8080 }
        env {
          name  = "TELOXIDE_TOKEN"
          value_from { secret_key_ref { name = google_secret_manager_secret.teloxide_token.secret_id; key = "latest" } }
        }
        env {
          name  = "MONGODB_URI"
          value_from { secret_key_ref { name = google_secret_manager_secret.mongodb_uri.secret_id; key = "latest" } }
        }
        env { name = "WEBHOOK_URL"; value = var.webhook_url }
        env { name = "PORT"; value = "8080" }
      }
    }
  }
  traffic { percent = 100 }
  autogenerate_revision_name = true
}

# Poller service (one sweep per HTTP POST from Scheduler)
resource "google_cloud_run_service" "poller" {
  name     = "${var.app_name}-poller"
  location = var.region

  template {
    spec {
      container_concurrency = 1
      timeout_seconds = 300
      containers {
        image = var.image
        command = ["/mandown_poller"]
        env {
          name  = "TELOXIDE_TOKEN"
          value_from { secret_key_ref { name = google_secret_manager_secret.teloxide_token.secret_id; key = "latest" } }
        }
        env {
          name  = "MONGODB_URI"
          value_from { secret_key_ref { name = google_secret_manager_secret.mongodb_uri.secret_id; key = "latest" } }
        }
      }
    }
  }
  traffic { percent = 100 }
}

# Allow unauthenticated invocations (Telegram + Scheduler)
resource "google_cloud_run_service_iam_member" "bot_public" {
  service  = google_cloud_run_service.bot.name
  location = var.region
  role     = "roles/run.invoker"
  member   = "allUsers"
}
resource "google_cloud_run_service_iam_member" "poller_invoker" {
  service  = google_cloud_run_service.poller.name
  location = var.region
  role     = "roles/run.invoker"
  member   = "serviceAccount:${google_service_account.scheduler.email}"
}
```

### 5.4 New: `infra/scheduler.tf`

```hcl
resource "google_service_account" "scheduler" {
  account_id   = "${var.app_name}-scheduler"
  display_name = "ManDown Cloud Scheduler invoker"
}

resource "google_cloud_scheduler_job" "poll" {
  name        = "${var.app_name}-poll"
  description = "Trigger ManDown poller sweep"
  schedule    = "*/10 * * * *"
  time_zone   = "Etc/UTC"

  http_target {
    uri         = google_cloud_run_service.poller.status[0].url
    http_method = "POST"
    oidc_token {
      service_account_email = google_service_account.scheduler.email
    }
  }
}
```

### 5.5 New: `infra/secrets.tf`

```hcl
resource "google_secret_manager_secret" "teloxide_token" {
  secret_id = "${var.app_name}-teloxide-token"
  replication { auto = true }
}
resource "google_secret_manager_secret" "mongodb_uri" {
  secret_id = "${var.app_name}-mongodb-uri"
  replication { auto = true }
}
# + google_secret_manager_secret_version for the actual values (applied manually or via CI)
# + google_secret_manager_secret_iam_member bindings so the Cloud Run runtime SA can read them
```

### 5.6 Scaling to zero (free tier)

The Cloud Run services default to `min-instances=0`. Do **not** set `min_instance_count=1` on either service — that would make them always-billed (~$5/mo each). Cold starts (~1–2s for the Rust binary) are acceptable for a personal bot.

---

## 6. GitHub Actions changes

### 6.1 `.github/workflows/build-push.yaml`

Replace the `Update prod image` step (currently `gcloud compute instances update-container`) with Cloud Run deploys:

```bash
# Deploy bot
gcloud run deploy mandown-bot \
  --image ${{ env.image_name }} \
  --region ${{ vars.GCP_REGION }} \
  --platform managed \
  --no-traffic --quiet   # then shift 100% traffic to the new revision

# Deploy poller (same image, different entrypoint)
gcloud run deploy mandown-poller \
  --image ${{ env.image_name }} \
  --region ${{ vars.GCP_REGION }} \
  --platform managed --quiet
```

Add new GitHub variables: `GCP_REGION`, `WEBHOOK_URL`. The existing `GCP_PROJECT_ID`, `GCP_WIF`, `REGISTRY_URI`, `REPOSITORY_URI`, `IMAGE`, `FREQ` are reused. `GCP_ZONE` is no longer needed.

### 6.2 `.github/workflows/terraform.yaml`

Unchanged in shape — just applies the new config. The `terraform plan` on PRs will show the GCE VM being destroyed and Cloud Run services being created.

### 6.3 `.github/workflows/pr-test.yaml`

Unchanged. `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` now build all three binaries; tests are unaffected.

---

## 7. Migration / cutover steps

1. **Back up Firestore** — `gcloud firestore export gs://state-mandown/backup-$(date +%s)`.
2. **Implement the code changes** (Section 2) — lib + two bins, `run_once()`, webhook listener. Verify `cargo build --release` produces `man_down`, `man_down_bot`, `man_down_poller`.
3. **Deploy new Terraform** (Section 5) alongside the existing GCE VM. Cloud Run services come up empty (no traffic yet). Create the Secret Manager secrets and versions.
4. **Push the new image** to Artifact Registry (via a staging tag, not a release tag yet).
5. **Deploy to Cloud Run** from the staging image.
6. **Smoke test with a staging bot token** (create a second bot via BotFather, set its `WEBHOOK_URL` to the Cloud Run bot URL). Send `/track example.com`, `/list`, `/untrack example.com` — verify Firestore gets the right documents.
7. **Smoke test the poller** — `curl -X POST <poller-url>` manually; verify it sweeps and alerts the staging bot.
8. **Enable the Cloud Scheduler job.** Verify it fires every 10 min.
9. **Cut over the production bot** — call `setWebhook` for the production token pointing at the Cloud Run bot URL. The old GCE VM's long polling stops receiving updates immediately (Telegram sends updates to only one webhook/polling consumer).
10. **Observe for one poll cycle** — confirm alerts fire from the new poller.
11. **Destroy the GCE VM** — `terraform destroy -target=google_compute_instance.mandown` (or remove `compute.tf` and apply).
12. **Monitor for one week** — Cloud Run logs, Firestore quota, Scheduler job success rate.

---

## 8. Risks / watch-outs

- **Webhook HTTPS cert** — Cloud Run provides a valid TLS cert on its default URL. Don't use a custom domain unless you also manage the cert.
- **Cold-start latency** — `min-instances=0` adds ~1–2s to the first `/track` after idle. Acceptable for a personal bot. If not, set `min-instances=1` (≈ $5/mo, no longer free).
- **Poller overlap** — Cloud Scheduler could fire the next run before the previous finishes. With `FREQ=600s` and a ~30s sweep this is unlikely, but guard with a Firestore "lock" doc: CAS on `last_poll_started_at`, skip if `now - last < FREQ - epsilon`.
- **Poller as service vs. job** — this plan uses a Cloud Run **service** invoked by HTTP POST (simplest). A Cloud Run **Job** is also possible but requires the Scheduler to call the Jobs execute API with OIDC, which is more plumbing. The HTTP-service approach is recommended.
- **teloxide webhook API** — verify the exact `webhooks::axum` and `dispatch_with_listener` signatures against teloxide 0.12 docs before finalizing `src/bin/bot.rs`.
- **Secret rotation** — `MONGODB_URI` and `TELOXIDE_TOKEN` move from GCE container env to Secret Manager. Update the GitHub Actions secrets accordingly (or keep them as WIF-injected env at deploy time).
- **`ENV` and `FREQ` on the bot** — `ENV` is unused in code; `FREQ` is only read by the poll loop, which the bot no longer runs. The poller reads `FREQ` only if you keep the loop in `run_once` — you don't, so `FREQ` becomes a Terraform/Scheduler-only concept (the cron schedule replaces it).

---

## 9. Effort estimate

| Phase | Hours |
|---|---|
| Code: lib + 2 bins, `run_once()`, webhook listener, `command.rs` refactor | ~5 |
| Dockerfile update (ship both binaries) | ~0.5 |
| Terraform: delete VM, add Cloud Run + Scheduler + Secrets | ~3 |
| GitHub Actions: deploy commands + new vars | ~1 |
| Cutover + smoke test (staging token) | ~2 |
| **Total** | **~11.5** |

---

## 10. Summary

- **One crate, three binaries**: keep `man_down` (local dev, long polling + loop), add `man_down_bot` (prod, webhook) and `man_down_poller` (prod, one-shot sweep).
- **Telegram**: long polling → webhooks via teloxide's `webhook` feature + `axum`; Cloud Run's HTTPS URL is the webhook target.
- **Poller**: in-process `loop { sleep }` → Cloud Scheduler HTTP trigger; the poller runs one sweep and exits.
- **State**: Firestore + `mongodb` crate, completely unchanged.
- **Infra**: delete GCE VM, add two Cloud Run services + one Cloud Scheduler job + Secret Manager secrets; keep Firestore and Artifact Registry.
- **Free tier**: stays within Cloud Run / Firestore / Scheduler free quotas with comfortable headroom, as long as `min-instances=0` on both services.
