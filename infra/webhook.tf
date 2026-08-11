locals {
  webhook_url = "https://webhook-${data.google_project.project.number}.${var.region}.run.app"
}

resource "google_cloud_run_v2_service" "webhook" {
  name     = "webhook"
  location = var.region
  ingress  = "INGRESS_TRAFFIC_ALL"

  template {
    service_account                  = "${data.google_project.project.number}-compute@developer.gserviceaccount.com"
    timeout                          = "60s"
    max_instance_request_concurrency = 1

    scaling {
      min_instance_count = 0
      max_instance_count = 1
    }

    containers {
      image = var.image_webhook

      ports {
        container_port = 8080
      }

      env {
        name  = "WEBHOOK_URL"
        value = local.webhook_url
      }

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

      env {
        name = "WEBHOOK_TOKEN"
        value_source {
          secret_key_ref {
            secret  = google_secret_manager_secret.webhook_token.secret_id
            version = "latest"
          }
        }
      }
    }
  }

  depends_on = [
    google_secret_manager_secret_version.teloxide_token,
    google_secret_manager_secret_version.mongodb_uri,
    google_secret_manager_secret_version.webhook_token,
    google_secret_manager_secret_iam_member.teloxide_token_accessor,
    google_secret_manager_secret_iam_member.mongodb_uri_accessor,
    google_secret_manager_secret_iam_member.webhook_token_accessor,
  ]
}

# Allow unauthenticated invocations (required for Telegram webhooks)
resource "google_cloud_run_v2_service_iam_member" "webhook_invoker" {
  name     = google_cloud_run_v2_service.webhook.name
  location = google_cloud_run_v2_service.webhook.location
  role     = "roles/run.invoker"
  member   = "allUsers"
}

# Set Telegram webhook URL; secret_token is the separate WEBHOOK_TOKEN
resource "terraform_data" "set_webhook" {
  triggers_replace = [
    local.webhook_url,
    random_password.webhook_token.result,
    var.teloxide_token,
  ]

  provisioner "local-exec" {
    command = "curl -sS -X POST \"https://api.telegram.org/bot${var.teloxide_token}/setWebhook\" -d \"url=${local.webhook_url}\" -d \"secret_token=${random_password.webhook_token.result}\""
  }

  depends_on = [
    google_cloud_run_v2_service_iam_member.webhook_invoker,
  ]
}
