data "google_project" "project" {
}

module "container" {
  source  = "terraform-google-modules/container-vm/google"
  version = "~> 2.0" # Upgrade the version if necessary.

  container = {
    image = var.image
  }
}

resource "google_compute_instance" "mandown" {
  name         = "mandown"
  machine_type = "e2-micro"
  zone         = var.zone

  boot_disk {
    auto_delete = true
    initialize_params {
      image = module.container.source_image
      size  = 10
      type  = "pd-standard"
    }
  }

  labels = {
    container-vm = module.container.vm_container_label
  }

  metadata = {
    gce-container-declaration = module.container.metadata_value
    google-logging-enabled    = "true"
    google-monitoring-enabled = "true"
  }

  network_interface {
    network = "default"
  }

  service_account {
    email = "${data.google_project.project.number}-compute@developer.gserviceaccount.com"
    scopes = [
      "cloud-platform"
    ]
  }

  tags = ["mandown"]
}
