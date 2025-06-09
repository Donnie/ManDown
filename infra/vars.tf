variable "app_name" {
  description = "Application Name"
  type        = string
}

variable "project_id" {
  description = "GCP Project ID"
  type        = string
}

variable "region" {
  description = "GCP Region"
  type        = string
}

variable "zone" {
  description = "GCP Zone"
  type        = string
}

## Compute Engine
variable "freq" {
  description = "Frequency in seconds"
  type        = string
  default     = "600"
}

variable "image" {
  description = "Docker image"
  type        = string
}

variable "teloxide_token" {
  description = "Teloxide token"
  type        = string
  sensitive   = true
}

variable "mongodb_uri" {
  description = "MongoDB URI"
  type        = string
  sensitive   = true
}
