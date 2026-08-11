resource "google_secret_manager_secret" "teloxide_token" {
  secret_id = "MANDOWN_TELOXIDE_TOKEN"

  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "teloxide_token" {
  secret                 = google_secret_manager_secret.teloxide_token.id
  secret_data_wo         = var.teloxide_token
  secret_data_wo_version = 1
}

resource "google_secret_manager_secret" "mongodb_uri" {
  secret_id = "MANDOWN_MONGODB_URI"

  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "mongodb_uri" {
  secret                 = google_secret_manager_secret.mongodb_uri.id
  secret_data_wo         = var.mongodb_uri
  secret_data_wo_version = 1
}

# Generate random webhook token (path + Telegram secret_token)
resource "random_password" "webhook_token" {
  length  = 32
  special = false
}

resource "google_secret_manager_secret" "webhook_token" {
  secret_id = "MANDOWN_WEBHOOK_TOKEN"

  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "webhook_token" {
  secret                 = google_secret_manager_secret.webhook_token.id
  secret_data_wo         = random_password.webhook_token.result
  secret_data_wo_version = 1
}

resource "google_secret_manager_secret_iam_member" "teloxide_token_accessor" {
  secret_id = google_secret_manager_secret.teloxide_token.id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${data.google_project.project.number}-compute@developer.gserviceaccount.com"
}

resource "google_secret_manager_secret_iam_member" "mongodb_uri_accessor" {
  secret_id = google_secret_manager_secret.mongodb_uri.id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${data.google_project.project.number}-compute@developer.gserviceaccount.com"
}

resource "google_secret_manager_secret_iam_member" "webhook_token_accessor" {
  secret_id = google_secret_manager_secret.webhook_token.id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${data.google_project.project.number}-compute@developer.gserviceaccount.com"
}
