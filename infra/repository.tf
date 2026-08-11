resource "google_artifact_registry_repository" "mandown" {
  location               = var.region
  repository_id          = var.app_name
  description            = "ManDown repo"
  format                 = "DOCKER"
  cleanup_policy_dry_run = false

  cleanup_policies {
    id     = "delete-untagged"
    action = "DELETE"
    condition {
      tag_state  = "UNTAGGED"
      older_than = "1d"
    }
  }

  cleanup_policies {
    id     = "delete-old-versions"
    action = "DELETE"
    condition {
      tag_state  = "ANY"
      older_than = "14d"
    }
  }

  cleanup_policies {
    id     = "keep-minimum-versions"
    action = "KEEP"
    most_recent_versions {
      package_name_prefixes = [
        "mandown-poller",
        "mandown-webhook",
      ]
      keep_count = 10
    }
  }
}

output "registry_uri" {
  value = "${var.region}-docker.pkg.dev"
}

output "repository_uri" {
  value = "${var.region}-docker.pkg.dev/${var.project_id}/${google_artifact_registry_repository.mandown.name}"
}
