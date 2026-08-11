resource "google_secret_manager_secret" "teloxide_token" {
  secret_id = "MANDOWN_TELOXIDE_TOKEN"

  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "teloxide_token" {
  secret      = google_secret_manager_secret.teloxide_token.id
  secret_data = var.teloxide_token
}

resource "google_secret_manager_secret" "mongodb_uri" {
  secret_id = "MANDOWN_MONGODB_URI"

  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "mongodb_uri" {
  secret      = google_secret_manager_secret.mongodb_uri.id
  secret_data = var.mongodb_uri
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
