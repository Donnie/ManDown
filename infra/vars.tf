variable "app_name" {
  description = "Application Name"
  type        = string
}

variable "image" {
  description = "Docker image"
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
