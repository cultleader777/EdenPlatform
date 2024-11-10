use std::collections::HashSet;

use crate::{
    database::{Database, TableRowPointerBackendApplication, TableRowPointerFrontendApplication, TableRowPointerRegion},
    static_analysis::{server_runtime::{ProvisioningScriptTag, ServerRuntime}, L1Projections, get_global_settings},
};

pub fn build_epl_jobs_script(database: &Database, l1proj: &L1Projections, region: TableRowPointerRegion, runtime: &mut ServerRuntime) {
    let mut res = String::new();

    res += "set -e\n";

    let mut backend_apps_built: HashSet<TableRowPointerBackendApplication> = HashSet::new();
    let mut frontend_apps_built: HashSet<TableRowPointerFrontendApplication> = HashSet::new();
    for ce in database.rust_compilation_environment().rows_iter() {
        if let Some(app) = database
            .rust_compilation_environment()
            .c_referrers_backend_application__build_environment(ce)
            .iter()
            .next()
        {
            if l1proj.backend_apps_in_region.value(region).contains(app) {
                assert!(backend_apps_built.insert(*app));

                // build all different compilation environments in parallel first
                res += "cd $EPL_PROVISIONING_DIR/apps/";
                res += database.backend_application().c_application_name(*app);
                res += " && nix build &\n";
            }
        }

        if let Some(app) = database
            .rust_compilation_environment()
            .c_referrers_frontend_application__build_environment(ce)
            .iter()
            .next()
        {
            if l1proj.frontend_apps_in_region.value(region).contains(app) {
                assert!(frontend_apps_built.insert(*app));

                // build all different compilation environments in parallel first
                res += "cd $EPL_PROVISIONING_DIR/apps/";
                res += database.frontend_application().c_application_name(*app);
                res += " && nix build &\n";
            }
        }
    }

    // wait for all compilation environments to build
    res += "wait\n";

    // build the rest of the backend apps
    for app in database.backend_application().rows_iter() {
        if l1proj.backend_apps_in_region.value(region).contains(&app) && backend_apps_built.insert(app) {
            res += "cd $EPL_PROVISIONING_DIR/apps/";
            res += database.backend_application().c_application_name(app);
            res += " && nix build &\n";
        }
    }

    // build the rest of the frontend apps
    for app in database.frontend_application().rows_iter() {
        if l1proj.frontend_apps_in_region.value(region).contains(&app) && frontend_apps_built.insert(app) {
            res += "cd $EPL_PROVISIONING_DIR/apps/";
            res += database.frontend_application().c_application_name(app);
            res += " && nix build &\n";
        }
    }

    res += "\n";
    res += "wait\n";
    res += "\n";

    runtime.add_provisioning_script(
        region,
        ProvisioningScriptTag::EplApplicationBuild,
        "build-epl-apps.sh",
        res,
    );
}

pub fn push_epl_apps_to_registry(database: &Database, l1proj: &L1Projections, region: TableRowPointerRegion, runtime: &mut ServerRuntime) {
    let settings = get_global_settings(database);
    let mut res = String::new();

    res += "set -e\n";

    // push to private registry
    let reg_name = &settings.docker_registry_service_name;
    let reg_port = settings.docker_registry_port;
    let docker_image_prefix = format!("{reg_name}.service.consul:{reg_port}/epl-app");
    res += "IMAGE_PREFIX=";
    res += &docker_image_prefix;
    res += "\n";
    for app in database.backend_application().rows_iter() {
        if l1proj.backend_apps_in_region.value(region).contains(&app) {
            let app_name = database.backend_application().c_application_name(app);
            let image_load_expr =
                format!("docker load -i $EPL_PROVISIONING_DIR/apps/{app_name}/result");
            res += &image_load_expr;
            res += "\n";
            res += "IMAGE_TAG=$( ";
            res += &image_load_expr;
            res += " | sed -E 's/^.*: //g' )\n";
            res += "RETAGGED_IMAGE=$IMAGE_PREFIX/$IMAGE_TAG\n";
            res += "docker tag $IMAGE_TAG $RETAGGED_IMAGE\n";
            res += "docker push $RETAGGED_IMAGE\n";
            res += "RETAGGED_DIGEST_TAG=$( docker image inspect $RETAGGED_IMAGE | jq -r '.[0].RepoDigests[0]' )\n";
            res += "echo \"RETAGGED_DIGEST_TAG -> [$RETAGGED_DIGEST_TAG]\"\n";
            // replace image tag in all deployments
            for depl in database
                .backend_application()
                .c_referrers_backend_application_deployment__application_name(app)
            {
                let workload_architecture = database.backend_application_deployment().c_workload_architecture(*depl);
                let ns = database.nomad_namespace().c_namespace(database.backend_application_deployment().c_namespace(*depl));
                assert_eq!(workload_architecture, "x86_64", "Only x86_64 builds supported so far");
                let depl_name = database
                    .backend_application_deployment()
                    .c_deployment_name(*depl);
                res += &format!("sed -i \"s#@@EPL_APP_IMAGE_{workload_architecture}:{app_name}@@#$RETAGGED_DIGEST_TAG#g\" $EPL_PROVISIONING_DIR/nomad-jobs/{ns}_app-{depl_name}.hcl\n");
            }
        }
    }

    for app in database.frontend_application().rows_iter() {
        if l1proj.frontend_apps_in_region.value(region).contains(&app) {
            let app_name = database.frontend_application().c_application_name(app);
            let image_load_expr =
                format!("docker load -i $EPL_PROVISIONING_DIR/apps/{app_name}/result");
            // Load image first time in case tag is renamed such message is emitted:
            // The image frontend-test:v0.1.0-50sdf2frvbfhhi0s9f348d8zi52wxb2p already exists, renaming the old one with ID sha256:5fa78ca306efda31c8656177ca0e28fc910c4c76950f798cf9f077479c467676 to empty string
            res += &image_load_expr;
            res += "\n";
            res += "IMAGE_TAG=$( ";
            res += &image_load_expr;
            res += " | sed -E 's/^.*: //g' )\n";
            res += "RETAGGED_IMAGE=$IMAGE_PREFIX/$IMAGE_TAG\n";
            res += "echo \"RETAGGED_IMAGE -> [$RETAGGED_IMAGE]\"\n";
            res += "echo \"IMAGE_TAG -> [$IMAGE_TAG]\"\n";
            res += "docker tag $IMAGE_TAG $RETAGGED_IMAGE\n";
            res += "docker push $RETAGGED_IMAGE\n";
            res += "RETAGGED_DIGEST_TAG=$( docker image inspect $RETAGGED_IMAGE | jq -r '.[0].RepoDigests[0]' )\n";
            res += "echo \"RETAGGED_DIGEST_TAG -> [$RETAGGED_DIGEST_TAG]\"\n";
            // replace image tag in all deployments
            // TODO: do hash of the image
            for depl in database
                .frontend_application()
                .c_referrers_frontend_application_deployment__application_name(app)
            {
                let architecture = database.frontend_application_deployment().c_workload_backend_architecture(*depl);
                let ns = database.nomad_namespace().c_namespace(database.frontend_application_deployment().c_namespace(*depl));
                let depl_name = database
                    .frontend_application_deployment()
                    .c_deployment_name(*depl);
                res += &format!("sed -i \"s#@@EPL_APP_IMAGE_{architecture}:{app_name}@@#$RETAGGED_DIGEST_TAG#g\" $EPL_PROVISIONING_DIR/nomad-jobs/{ns}_app-{depl_name}.hcl\n");
            }
        }
    }

    runtime.add_provisioning_script(
        region,
        ProvisioningScriptTag::EplApplicationPush,
        "dr-push-epl-apps.sh",
        res,
    );
}
