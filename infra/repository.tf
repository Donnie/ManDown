resource "google_artifact_registry_repository" "mandown" {
  location               = var.region
  repository_id          = var.repository_name
  description            = "ManDown repo"
  format                 = "DOCKER"
  cleanup_policy_dry_run = false

  cleanup_policies {
    id     = "delete-old-images"
    action = "DELETE"
    condition {
      tag_state  = "TAGGED"
      older_than = "7d"
      tag_prefixes = [
        "dev-",
      ]
    }
  }

  cleanup_policies {
    id     = "keep-3-latest-images"
    action = "KEEP"
    most_recent_versions {
      keep_count = 3
    }
  }
}

output "registry_uri" {
  value = "${var.region}-docker.pkg.dev"
}

output "repository_uri" {
  value = "${var.region}-docker.pkg.dev/${var.project_id}/${google_artifact_registry_repository.mandown.name}"
}
