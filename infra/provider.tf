provider "google" {
  project = var.project_id
  region  = var.region
}

terraform {
  required_version = "~> 1.15"
  required_providers {
    google = {
      source = "hashicorp/google"
      # 6.44.0+ fixes empty cleanup_policies sent on update (hashicorp/terraform-provider-google#23486)
      version = "6.50.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.6"
    }
  }

  backend "gcs" {
    bucket = "state-mandown"
    prefix = "state"
  }
}
