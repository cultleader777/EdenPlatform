use std::collections::BTreeMap;

use regex::Regex;

use crate::database::{Database, TableRowPointerDockerImage, TableRowPointerDockerImagePin, TableRowPointerBackendApplicationDeployment, TableRowPointerFrontendApplicationDeployment};

use super::PlatformValidationError;


lazy_static! {
    pub static ref DOCKER_IMAGE_REPO_REGEX: Regex =
        Regex::new(r#"^([a-z0-9.-]+/)?([a-zA-Z0-9_-]+/)?[a-zA-Z0-9_-]+$"#)
        .unwrap();
    pub static ref DOCKER_IMAGE_CHECKSUM_REGEX: Regex =
        Regex::new(r#"^sha256:[0-9a-f]{64}$"#)
        .unwrap();
}

pub fn image_to_full_name(db: &Database, di: TableRowPointerDockerImage) -> String {
    let repo = db.docker_image().c_repository(di);
    let checksum = db.docker_image().c_checksum(di);
    format!("{repo}@{checksum}")
}

pub fn run_docker_image_checks(db: &Database) -> Result<(), PlatformValidationError> {
    for di in db.docker_image().rows_iter() {
        let checksum = db.docker_image().c_checksum(di);
        if !checksum.starts_with("sha256:") {
            return Err(PlatformValidationError::DockerImageChecksumDoesntStartWithSha256 {
                bad_image_checksum: checksum.clone(),
                image_set: db.docker_image_set().c_set_name(db.docker_image().c_image_set(di)).clone(),
                repository: db.docker_image().c_repository(di).clone(),
                expected_prefix: "sha256:".to_string(),
            });
        }

        let expected_length = 71;
        if checksum.len() != expected_length {
            return Err(PlatformValidationError::DockerImageChecksumBadLength {
                bad_image_checksum: checksum.clone(),
                image_set: db.docker_image_set().c_set_name(db.docker_image().c_image_set(di)).clone(),
                repository: db.docker_image().c_repository(di).clone(),
                actual_length: checksum.len(),
                expected_length,
            });
        }

        if !DOCKER_IMAGE_CHECKSUM_REGEX.is_match(&checksum) {
            return Err(PlatformValidationError::DockerImageChecksumBadSymbols {
                bad_image_checksum: checksum.clone(),
                image_set: db.docker_image_set().c_set_name(db.docker_image().c_image_set(di)).clone(),
                repository: db.docker_image().c_repository(di).clone(),
                only_allowed_checksum_characters: "0123456789abcdef".to_string(),
            });
        }
    }

    for pin in db.docker_image_pin().rows_iter() {
        let mut architectures: BTreeMap<String, String> = BTreeMap::new();
        for child in db.docker_image_pin().c_children_docker_image_pin_images(pin) {
            let di = db.docker_image_pin_images().c_checksum(*child);
            let fname = image_to_full_name(db, di);
            let di_arch = db.docker_image().c_architecture(di);
            if let Some(res) = architectures.insert(di_arch.clone(), fname) {
                return Err(PlatformValidationError::DockerImagePinContainsMultipleImagesForSameArchitecture {
                    image_pin_name: db.docker_image_pin().c_pin_name(pin).clone(),
                    architecture: di_arch.clone(),
                    previous_docker_image: res,
                    duplicate_docker_image: image_to_full_name(db, di),
                });
            }
        }
    }

    Ok(())
}

#[derive(Clone)]
pub enum DockerImageHandle {
    DockerImagePin {
        pin: TableRowPointerDockerImagePin,
        chosen: TableRowPointerDockerImage,
        full_image_name: String,
    },
    EplBackendApplicationDeployment {
        depl: TableRowPointerBackendApplicationDeployment,
        full_image_name: String,
    },
    EplFrontendApplicationDeployment {
        depl: TableRowPointerFrontendApplicationDeployment,
        full_image_name: String,
    },
}

impl DockerImageHandle {
    pub fn image_placeholder(&self) -> &str {
        match self {
            DockerImageHandle::DockerImagePin { full_image_name, .. } => full_image_name.as_str(),
            DockerImageHandle::EplBackendApplicationDeployment { full_image_name, .. } => full_image_name.as_str(),
            DockerImageHandle::EplFrontendApplicationDeployment { full_image_name, .. } => full_image_name.as_str(),
        }
    }

    pub fn docker_image_ptr(&self) -> Option<TableRowPointerDockerImage> {
        match &self {
            DockerImageHandle::DockerImagePin { chosen, .. } => Some(*chosen),
            DockerImageHandle::EplBackendApplicationDeployment { .. } => None,
            DockerImageHandle::EplFrontendApplicationDeployment { .. } => None,
        }
    }
}

pub fn image_handle_from_pin(
    db: &Database,
    target_architecture: &str,
    pin: TableRowPointerDockerImagePin,
    expected_docker_image_set: &str,
) -> Result<DockerImageHandle, PlatformValidationError>
{
    for pin_image in db.docker_image_pin().c_children_docker_image_pin_images(pin) {
        let image = db.docker_image_pin_images().c_checksum(*pin_image);
        if db.docker_image().c_architecture(image).as_str() == target_architecture {
            let image_set = db.docker_image().c_image_set(image);
            if db.docker_image_set().c_set_name(image_set) != expected_docker_image_set {
                return Err(
                    PlatformValidationError::DockerImageDoesNotBelongToTheExpectedSet {
                        image_pin_name: db.docker_image_pin().c_pin_name(pin).clone(),
                        expected_docker_image_set: expected_docker_image_set.to_string(),
                        found_docker_image_set: db.docker_image_set().c_set_name(image_set).to_string(),
                        image_architecture: target_architecture.to_string(),
                        image_checksum: db.docker_image().c_checksum(image).clone(),
                        image_repository: db.docker_image().c_repository(image).clone(),
                    }
                );
            }
            return Ok(DockerImageHandle::DockerImagePin {
                pin, chosen: image, full_image_name: image_to_full_name(db, image)
            });
        }
    }

    Err(
        PlatformValidationError::DockerImageNotFoundForArchitectureForPin {
            image_pin_name: db.docker_image_pin().c_pin_name(pin).clone(),
            architecture_image_not_found: target_architecture.to_string(),
            found_architecture_images:
                db.docker_image_pin()
                  .c_children_docker_image_pin_images(pin)
                  .iter()
                  .map(|i| {
                      let image = db.docker_image_pin_images().c_checksum(*i);
                      db.docker_image().c_architecture(image).clone()
                  })
                  .collect()
        }
    )
}


pub fn image_handle_from_backend_app_deployment(
    db: &Database,
    backend_app_depl: TableRowPointerBackendApplicationDeployment,
) -> Result<DockerImageHandle, PlatformValidationError> {
    let arch = db.backend_application_deployment().c_workload_architecture(backend_app_depl);
    if arch != "x86_64" {
        return Err(PlatformValidationError::NonAmd64BuildsNotSupportedYetForApplications {
            application_deployment: db.backend_application_deployment().c_deployment_name(backend_app_depl).clone(),
            workload_architecture: arch.clone(),
            only_allowed_workload_architecture: "x86_64".to_string(),
        });
    }
    let app = db.backend_application_deployment().c_application_name(backend_app_depl);
    let app_name = db.backend_application().c_application_name(app);
    Ok(DockerImageHandle::EplBackendApplicationDeployment {
        depl: backend_app_depl,
        full_image_name: format!("@@EPL_APP_IMAGE_{arch}:{app_name}@@"),
    })
}

pub fn image_handle_from_frontend_app_deployment(
    db: &Database,
    frontend_app_depl: TableRowPointerFrontendApplicationDeployment,
) -> Result<DockerImageHandle, PlatformValidationError> {
    let arch = db.frontend_application_deployment().c_workload_backend_architecture(frontend_app_depl);
    if arch != "x86_64" {
        return Err(PlatformValidationError::NonAmd64BuildsNotSupportedYetForApplications {
            application_deployment: db.frontend_application_deployment().c_deployment_name(frontend_app_depl).clone(),
            workload_architecture: arch.clone(),
            only_allowed_workload_architecture: "x86_64".to_string(),
        });
    }
    let app = db.frontend_application_deployment().c_application_name(frontend_app_depl);
    let arch = db.frontend_application_deployment().c_workload_backend_architecture(frontend_app_depl);
    let app_name = db.frontend_application().c_application_name(app);
    Ok(DockerImageHandle::EplFrontendApplicationDeployment {
        depl: frontend_app_depl,
        full_image_name: format!("@@EPL_APP_IMAGE_{arch}:{app_name}@@"),
    })
}
