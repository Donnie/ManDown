resource "google_firestore_database" "firestore" {
  database_edition                  = "ENTERPRISE"
  delete_protection_state           = "DELETE_PROTECTION_ENABLED"
  location_id                       = var.region
  name                              = var.app_name
  point_in_time_recovery_enablement = "POINT_IN_TIME_RECOVERY_ENABLED"
  project                           = var.project_id
  type                              = "FIRESTORE_NATIVE"
}
