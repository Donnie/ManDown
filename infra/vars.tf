variable "project_id" {
  description = "GCP Project ID"
  type        = string
}

variable "region" {
  description = "GCP Region"
  type        = string
}

variable "repository_name" {
  description = "Artifact Registry Repository Name"
  type        = string
}

variable "state_bucket" {
  description = "State Bucket Name"
  type        = string
}
