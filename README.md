# ManDown
[![Rust Report Card](https://rust-reportcard.xuri.me/badge/github.com/donnie/mandown)](https://rust-reportcard.xuri.me/report/github.com/donnie/mandown)

Telegram bot that tracks website availability by polling HTTP status codes.
Any change (for example `200 → 500` or `502 → 404`) is reported to you on Telegram.

## Production

Try it now: https://t.me/ManDownBot

Production runs on Google Cloud Run:

- **webhook** — Cloud Run service that receives Telegram updates
- **poller** — Cloud Run job triggered by Cloud Scheduler every 10 minutes

State is stored in Google Firestore (MongoDB API). Infra lives in [`infra/`](./infra/).

I have been running this bot since Jul 2020. Cost is covered by the Google Free Tier.
For uptime guarantees, host it on your own cloud account.

## Workspace

```
crates/core      shared library (mongo, HTTP checks, commands, alerts)
crates/poller    one-shot downtime sweep binary
crates/webhook   Telegram webhook listener binary
```

## Dev setup

Copy `.env.dist` to `.env` and fill in values:

```bash
cp .env.dist .env
```

Required for both binaries:

- `TELOXIDE_TOKEN` — Telegram bot token
- `MONGODB_URI` — MongoDB / Firestore connection string

Additional for the webhook:

- `WEBHOOK_URL` — public HTTPS URL Telegram should call (use something like `ngrok http 8080` locally)
- `WEBHOOK_TOKEN` — shared secret for Telegram's `secret_token`
- `PORT` — listen port (defaults to `8080`)

Run one binary at a time:

```bash
# one sweep, then exit
cargo run -p man_down_poller

# webhook listener (needs WEBHOOK_URL + WEBHOOK_TOKEN)
cargo run -p man_down_webhook
```

Release build:

```bash
cargo build --release --workspace
```

Container images:

```bash
podman build -f crates/poller/Containerfile -t mandown-poller .
podman build -f crates/webhook/Containerfile -t mandown-webhook .
```

## Commands

### `/track`
1. Send `/track google.in`
2. The bot validates the URL
3. It checks both `http` and `https`
4. If the site is not already tracked, it is added

### Polling
1. Cloud Scheduler triggers the poller job on a fixed schedule (every 10 minutes in production)
2. The poller checks whether any tracked site's status changed
3. On change, it sends you a Telegram message

### `/untrack`
1. Send `/untrack google.in`
2. The bot validates the URL
3. It removes both `http` and `https` forms

### `/list`
1. Send `/list`
2. The bot replies with the domains you are tracking and their status codes

## Contributing

1. Fork it
2. Clone: `git clone https://github.com/Donnie/ManDown`
3. Create a feature branch: `git checkout -b new-feature`
4. Make changes and add them: `git add .`
5. Commit: `git commit -m "Add some feature"`
6. Push: `git push origin new-feature`
7. Open a pull request against `main`

Suggestions and bug reports via issues are welcome.

## CI / CD

1. PRs run formatting, clippy, tests, and container builds
2. Merges to `main` build and push the poller and webhook images, and can apply Terraform changes under `infra/`

## Questions

Feel free to raise issues when you have questions or you are stuck somewhere.
