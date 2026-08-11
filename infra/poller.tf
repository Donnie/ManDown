resource "google_service_account" "poller_scheduler" {
  account_id   = "poller-scheduler"
  display_name = "Poller Cloud Scheduler"
}

resource "google_cloud_run_v2_job" "poller" {
  name     = "poller"
  location = var.region

  template {
    template {
      service_account = "${data.google_project.project.number}-compute@developer.gserviceaccount.com"

      containers {
        image = var.image_poller

        env {
          name = "TELOXIDE_TOKEN"
          value_source {
            secret_key_ref {
              secret  = google_secret_manager_secret.teloxide_token.secret_id
              version = "latest"
            }
          }
        }

        env {
          name = "MONGODB_URI"
          value_source {
            secret_key_ref {
              secret  = google_secret_manager_secret.mongodb_uri.secret_id
              version = "latest"
            }
          }
        }
      }
    }
  }

  depends_on = [
    google_secret_manager_secret_version.teloxide_token,
    google_secret_manager_secret_version.mongodb_uri,
    google_secret_manager_secret_iam_member.teloxide_token_accessor,
    google_secret_manager_secret_iam_member.mongodb_uri_accessor,
  ]
}

resource "google_cloud_run_v2_job_iam_member" "poller_scheduler_invoker" {
  name     = google_cloud_run_v2_job.poller.name
  location = google_cloud_run_v2_job.poller.location
  role     = "roles/run.invoker"
  member   = "serviceAccount:${google_service_account.poller_scheduler.email}"
}

resource "google_service_account_iam_member" "poller_scheduler_sa_user" {
  service_account_id = google_service_account.poller_scheduler.name
  role               = "roles/iam.serviceAccountUser"
  member             = "serviceAccount:service-${data.google_project.project.number}@gcp-sa-cloudscheduler.iam.gserviceaccount.com"
}

resource "google_cloud_scheduler_job" "poller" {
  name             = "poller"
  description      = "Trigger poller Cloud Run Job every 10 minutes"
  schedule         = "*/10 * * * *"
  time_zone        = "Etc/UTC"
  region           = var.region
  attempt_deadline = "320s"

  http_target {
    http_method = "POST"
    uri         = "https://${var.region}-run.googleapis.com/apis/run.googleapis.com/v1/namespaces/${var.project_id}/jobs/${google_cloud_run_v2_job.poller.name}:run"

    oauth_token {
      service_account_email = google_service_account.poller_scheduler.email
    }
  }

  depends_on = [
    google_cloud_run_v2_job_iam_member.poller_scheduler_invoker,
    google_service_account_iam_member.poller_scheduler_sa_user,
  ]
}
