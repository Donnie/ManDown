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

variable "image_poller" {
  description = "Poller image"
  type        = string
}

variable "image_webhook" {
  description = "Webhook image"
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
