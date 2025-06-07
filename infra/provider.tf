provider "google" {
  project = var.project_id
  region  = var.region
}

terraform {
  required_version = "~> 1.12.1"
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "6.38.0"
    }
  }

  backend "gcs" {
    bucket = "state-mandown"
    prefix = "state"
  }
}
