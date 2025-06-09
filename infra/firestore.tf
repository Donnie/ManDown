resource "google_firestore_database" "firestore" {
  database_edition                  = "ENTERPRISE"
  delete_protection_state           = "DELETE_PROTECTION_ENABLED"
  location_id                       = var.region
  name                              = var.app_name
  point_in_time_recovery_enablement = "POINT_IN_TIME_RECOVERY_ENABLED"
  project                           = var.project_id
  type                              = "FIRESTORE_NATIVE"
}

resource "google_firestore_index" "url" {
  project     = var.project_id
  database    = google_firestore_database.firestore.name
  api_scope   = "MONGODB_COMPATIBLE_API"
  query_scope = "COLLECTION_GROUP"
  density     = "SPARSE_ANY"
  collection  = "websites"

  fields {
    field_path = "url"
    order      = "ASCENDING"
  }
}
resource "google_firestore_index" "telegram_id" {
  project     = var.project_id
  database    = google_firestore_database.firestore.name
  api_scope   = "MONGODB_COMPATIBLE_API"
  query_scope = "COLLECTION_GROUP"
  density     = "SPARSE_ANY"
  collection  = "websites"

  fields {
    field_path = "telegram_id"
    order      = "ASCENDING"
  }
}
