use std::{fmt::Write, collections::{BTreeSet, BTreeMap}};
use convert_case::Casing;

use crate::{codegen::{CheckedDB, CodegenPlan}, database::{TableRowPointerServer, TableRowPointerGrafana, TableRowPointerRegion}, static_analysis::{networking::{first_first_region_vault_server, first_region_server}, get_global_settings}};

use super::{l1_provisioning::utils::epl_arch_to_linux_arch, L1ProvOutputs};

pub fn generate_makefile(checked: &CheckedDB, plan: &mut CodegenPlan, l1_outputs: &L1ProvOutputs) {
    let mut res = String::new();

    let dev_vms_exist = vms_exist(checked);

    makefile_core_part(checked, dev_vms_exist, &mut res);
    makefile_l1_provisioning_part(checked, &mut res);
    makefile_l1_provisioning_wait_part(checked, &mut res);
    makefile_l1_provisioning_with_wait_parts(checked, &mut res);
    makefile_fast_l1_provisioning_part(checked, &mut res);
    makefile_preconditions_part(checked, &mut res);
    makefile_regional_part(checked, &mut res);
    makefile_networks_part(checked, dev_vms_exist, &mut res);
    makefile_server_part(checked, dev_vms_exist, &mut res);
    makefile_vpn_part(&mut res);
    if checked.projections.cloud_topologies.cloud_needed() {
        makefile_terraform_part(checked, &mut res);
        makefile_aws_public_subnet_bootstrap_internet(checked, &mut res);
    }
    makefile_compile_environments_part(checked, &mut res);
    makefile_apps_part(checked, &mut res);
    makefile_integration_tests_target(checked, &mut res);
    makefile_refresh_server_infra_state(checked, &mut res, l1_outputs);
    makefile_refresh_prom_metrics(checked, &mut res);

    plan.root_dir.create_file("Makefile", res);
    plan.root_dir.create_file_if_not_exists("custom.mk", "# user custom tasks included in Makefile to be added here".to_string());
}

fn makefile_core_part(checked: &CheckedDB, dev_vms_exist: bool, res: &mut String) {
    tag(res, "Core Makefile part");

    let settings = get_global_settings(&checked.db);
    let proj_name = &settings.project_name;

    let is_multi_dc = checked.db.datacenter().len() > 1;
    let multi_dc_l1_prov = if is_multi_dc {
        "
	# if wireguard is used run l1-provision second time
	$(RUN_AS_NON_ROOT) $(MAKE) l1-provision-with-wait -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID_2)"
    } else { "" };
    let ci_multi_dc_l1_prov = if is_multi_dc {
        // TODO: step to wait for ci-l1-provision for all servers?
        // poll prometheus endpoint with metric?
        "
	# if wireguard is used run l1-provision second time
	$(RUN_AS_NON_ROOT) $(MAKE) ci-l1-provision -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID_2)"
    } else { "" };
    let cross_dc_ping = if is_multi_dc {
        "
	$(RUN_AS_NON_ROOT) $(MAKE) wait-cross-dc-ping_all-regions"
    } else { "" };
    let restart_consul_masters = if is_multi_dc {
        "
	$(RUN_AS_NON_ROOT) $(MAKE) restart-consul-masters_all-regions"
    } else { "" };

    let machine_init_step = if dev_vms_exist {
        "
	$(MAKE) run-all-servers"
    } else { "" };

    let terraform_step = if checked.projections.cloud_topologies.cloud_needed() {
        "
	$(RUN_AS_NON_ROOT) $(MAKE) terraform-provision-all-clouds
	$(RUN_AS_NON_ROOT) $(MAKE) compile-project # public ips might be updated for servers, recompile project
	$(RUN_AS_NON_ROOT) $(MAKE) wait-ready-public-servers -j $(L1_PROVISIONING_JOBS) # wait for servers ready before running preconditions
	$(RUN_AS_NON_ROOT) $(MAKE) public-servers-preconditions -j $(L1_PROVISIONING_JOBS) # preconditions for public nodes
	$(RUN_AS_NON_ROOT) $(MAKE) ci-bootstrap-l1-public-nodes -j $(L1_PROVISIONING_JOBS) # provision public ip servers to establish wireguard"
    } else { "" };
    // Targets with ci- prefix are idempotent, we're tracking state in local filesystem
    // of what needs to be done depending on changes not to do expensive actions like terraform apply
    let ci_terraform_step = if checked.projections.cloud_topologies.cloud_needed() {
        "
	$(RUN_AS_NON_ROOT) $(MAKE) ci-terraform-provision-all-clouds
	$(RUN_AS_NON_ROOT) $(MAKE) compile-project # public ips might be updated for servers, recompile project
	$(RUN_AS_NON_ROOT) $(MAKE) ci-wait-ready-public-servers -j $(L1_PROVISIONING_JOBS) # wait for servers ready before running preconditions
	$(RUN_AS_NON_ROOT) $(MAKE) ci-public-servers-preconditions -j $(L1_PROVISIONING_JOBS) # preconditions for public nodes
	$(RUN_AS_NON_ROOT) $(MAKE) ci-bootstrap-l1-public-nodes -j $(L1_PROVISIONING_JOBS) # provision public ip servers to establish wireguard"
    } else { "" };
    let has_coproc_dcs = checked.sync_res.network.region_coprocessor_gws.len() > 0;
    let all_dcs_vpn = if has_coproc_dcs || checked.projections.cloud_topologies.cloud_needed() {
        "
	$(MAKE) all-dcs-start-vpn"
    } else { "" };
    let maybe_bootstrap_aws_private_subnets = if !checked.projections.cloud_topologies.aws.dcs.is_empty() {
        "
	$(RUN_AS_NON_ROOT) $(MAKE) aws-private-ips-bootstrap-internet"
    } else { "" };

    let maybe_dev_cache_steps = if dev_vms_exist {
        "
	$(RUN_AS_NON_ROOT) $(MAKE) run-nix-serve
	$(RUN_AS_NON_ROOT) $(MAKE) run-docker-cache
	# cache builds by building in our machine
	$(RUN_AS_NON_ROOT) $(MAKE) build-remote-vm-configs"
    } else { "" };

    let maybe_aws_build_img = if !checked.projections.cloud_topologies.aws.is_empty() {
        "
	$(RUN_AS_NON_ROOT) $(MAKE) aws-images"
    } else { "" };
    let maybe_google_cloud_build_img = if !checked.projections.cloud_topologies.gcloud.is_empty() {
        "
	$(RUN_AS_NON_ROOT) $(MAKE) gcloud-images"
    } else { "" };
    let maybe_aws_tasks = if !checked.projections.cloud_topologies.aws.is_empty() {
        aws_images_part(checked)
    } else { "".to_string() };
    let maybe_google_cloud_tasks = if !checked.projections.cloud_topologies.gcloud.is_empty() {
        google_cloud_images_part(checked)
    } else { "".to_string() };

    let maybe_dev_vm_tasks = if dev_vms_exist {
        let nix_version = checked.db.nixpkgs_version().c_version(
            checked.projections.default_used_nixpkgs_version
        );
        let mut vm_images_builds = String::new();
        let mut vm_image_deps = String::new();
        for arch in &checked.projections.used_architectures {
            let build_arch = epl_arch_to_linux_arch(arch);
            write!(&mut vm_image_deps, " servers/vm-template-{arch}.txt").unwrap();
            write!(&mut vm_images_builds, r#"
servers/vm-template-{arch}.txt: servers/vm-template-{arch}.nix
	cd servers && \
	  $(RUN_AS_NON_ROOT) $(NIXOS_GENERATE) \
	  -I nixpkgs=channel:nixos-{nix_version} \
	  -f qcow-zfs --system {build_arch}-linux \
	  -c vm-template-{arch}.nix > vm-template-{arch}.tmp && \
	  mv -f vm-template-{arch}.tmp vm-template-{arch}.txt
"#).unwrap();
        };
        format!(r#"
.PHONY: vm-images
vm-images:{vm_image_deps}

{vm_images_builds}

# cache dependencies for remote servers inside our current nix store
.PHONY: build-remote-vm-configs
build-remote-vm-configs:
	nix-build l1-provisioning/build-all-servers.nix --no-out-link

.PHONY: teardown
teardown: remove-markers all-dcs-stop-vpn stop-all-servers down-vm-networks destroy-vm-disks stop-nix-serve

.PHONY: remove-markers
remove-markers:
	rm -f infra-state.sqlite
	rm -rf markers

.PHONY: destroy-vm-disks
destroy-vm-disks:
	rm -rf servers/disks

# kill existing nix serve after teardown because we might
# run tests in different environment with different key
.PHONY: stop-nix-serve
stop-nix-serve:
	-ps aux | grep nix-serve.psgi | grep -v grep | awk '{{print $$2}}' | xargs kill

.PHONY: run-nix-serve
run-nix-serve:
	$(LOAD_SHELL_LIB) maybe_run_nix_serve

.PHONY: run-docker-cache
run-docker-cache: $(DOCKER_CACHE_CONFIG)
	docker ps --format='{{{{.Names}}}}' | grep $(DOCKER_CACHE_CONTAINER_NAME) || \
		docker run -d --rm \
			--network=host \
			-v $(realpath $(DOCKER_CACHE_DIR))/images:/var/lib/registry \
			-v $(realpath $(DOCKER_CACHE_CONFIG)):/etc/docker/registry/config.yml:ro \
			--name $(DOCKER_CACHE_CONTAINER_NAME) \
			$(DOCKER_CACHE_IMAGE)

.PHONY: stop-docker-cache
stop-docker-cache:
	docker rm -f $(DOCKER_CACHE_CONTAINER_NAME) || true

.PHONY: up-libvirt-nat-disable-rule
up-libvirt-nat-disable-rule:
ifneq ($(shell id -u), 0)
	$(error "You are not root, changing ip table rules requires sudo")
else
	iptables -t nat -I LIBVIRT_PRT -s 10.0.0.0/8 -d 10.0.0.0/8 -j RETURN
endif

# You're welcome, for I had to burn two days to figure this out on my own
.PHONY: ensure-bridge-nf-tables-set
ensure-bridge-nf-tables-set:
	sysctl net.bridge.bridge-nf-call-iptables | grep 'net.bridge.bridge-nf-call-iptables = 0' || \
	 ( echo "net.bridge.bridge-nf-call-iptables is not disabled \
	you might have a bad time with VM networking \
	https://wiki.libvirt.org/Net.bridge.bridge-nf-call_and_sysctl.conf.html" && exit 7 )

.PHONY: down-libvirt-nat-disable-rule
down-libvirt-nat-disable-rule:
ifneq ($(shell id -u), 0)
	$(error \"You are not root, changing ip table rules requires sudo\")
else
	iptables -t nat -D LIBVIRT_PRT -s 10.0.0.0/8 -d 10.0.0.0/8 -j RETURN || true
endif
"#)
    } else { "".to_string() };

    let maybe_multi_region_nomad_federation = if checked.db.region().len() > 1 {
        "\n\t$(RUN_AS_NON_ROOT) $(MAKE) nomad-cross-region-federation"
    } else { "" };

    // for development executable we want
    // to recompile rust project and depend on that
    // for release we just assume pre build binary
    let (
        epl_executable_var,
        maybe_build_epl_executable,
        maybe_depend_epl_exec_build,
        maybe_epl_sources,
        maybe_epl_exec_build,
        docker_cache_dir,
        docker_cache_cont_name,
    ) = if cfg!(debug_assertions) {
        (
            "EPL_EXECUTABLE ?= $(realpath $(EPL_PROJECT_DIR)/target/debug/epl)",
            "
.PHONY: epl-executable
epl-executable: $(EPL_EXECUTABLE)
",
            " epl-executable",
            "EPL_SOURCES = $(EPL_PROJECT_DIR)/Cargo.lock $(EPL_PROJECT_DIR)/Cargo.toml $(EPL_PROJECT_DIR)/build.rs $(shell find $(EPL_PROJECT_DIR)/src -type f) $(shell find $(EPL_PROJECT_DIR)/edb-src -type f)",
            " $(EPL_SOURCES)",
            "DOCKER_CACHE_DIR ?= ../../docker-cache",
            "DOCKER_CACHE_CONTAINER_NAME = epl_docker_image_cache",
        )
    } else {
        (
            "EPL_EXECUTABLE ?= epl",
            "",
            "",
            "",
            "",
            "DOCKER_CACHE_DIR ?= ./docker-cache",
            "DOCKER_CACHE_CONTAINER_NAME = epl_docker_image_cache_$(EPL_ENV_NAME)",
        )
    };

    let mut all_regions_vec = checked.db.region().rows_iter().map(|i| checked.db.region().c_region_name(i).as_str()).collect::<Vec<_>>();
    all_regions_vec.sort();
    let all_regions_sorted = all_regions_vec.join(" ");

    write!(res, r#"
EPL_ENV_NAME = {proj_name}
include custom.mk # user custom tasks
L1_PROVISIONING_ID := $(shell date --utc +%Y%m%d%H%M%S)
# for next runs
L1_PROVISIONING_ID_2 := $(shell echo $$(($(L1_PROVISIONING_ID)+1)))
L1_PROVISIONING_ID_3 := $(shell echo $$(($(L1_PROVISIONING_ID)+2)))
L1_PROVISIONING_ID_4 := $(shell echo $$(($(L1_PROVISIONING_ID)+3)))
L1_PROVISIONING_TOLERATE_REBUILD_FAIL := false
L1_PROVISIONING_JOBS := 10
ifneq ($(origin TF_DESTROY_AUTO_APPROVE),undefined)
TF_DESTROY_FLAGS := -auto-approve
endif
L1_RESTART_CONSUL_POST_SECRETS := false
MAKEFILE_DIRECTORY = $(shell pwd)
{docker_cache_cont_name}
EPL_PROJECT_DIR ?= $(realpath ../../..)
{epl_executable_var}
# TODO: build our own edendb and use that
EDENDB_EXECUTABLE ?= edendb
{docker_cache_dir}
# DOCKER_CACHE_IMAGE ?= registry:2.8.1
DOCKER_CACHE_IMAGE ?= registry@sha256:cc6393207bf9d3e032c4d9277834c1695117532c9f7e8c64e7b7adcda3a85f39

DOCKER_CACHE_CONFIG = $(DOCKER_CACHE_DIR)/config.yml
{maybe_epl_sources}
LOAD_SHELL_LIB=cd servers; source ./library.sh;

RUN_AS_NON_ROOT = $(shell [ "$$(id -u)" -eq 0 ] && echo 'sudo -E -u $$SUDO_USER' )

ifeq ($(origin EPL_SHELL),undefined)
    $(error this makefile should only be run inside eden platform nix shell)
endif

.PHONY: build-env-projects
build-env-projects:{maybe_google_cloud_build_img}{maybe_aws_build_img}
	$(RUN_AS_NON_ROOT) $(MAKE) build-all-apps
	$(RUN_AS_NON_ROOT) $(MAKE) build-integration-tests

markers/all-regions.txt:
	mkdir -p markers
	echo '{all_regions_sorted}' > markers/all-regions.txt.tmp
	cmp --silent markers/all-regions.txt.tmp markers/all-regions.txt || mv -f markers/all-regions.txt.tmp markers/all-regions.txt

.PHONY: full-provision-pre-l1
full-provision-pre-l1:
ifneq ($(shell id -u), 0)
	$(error "You are not root, full provision routine requires sudo")
else{machine_init_step}{terraform_step}{all_dcs_vpn}{maybe_bootstrap_aws_private_subnets}
	$(RUN_AS_NON_ROOT) $(MAKE) wait-ready-all-servers -j $(L1_PROVISIONING_JOBS)
	$(RUN_AS_NON_ROOT) $(MAKE) all-servers-preconditions -j $(L1_PROVISIONING_JOBS)
endif

# target is tweaked for being run all the time so that targets are skipped
.PHONY: ci-full-provision-pre-l1
ci-full-provision-pre-l1:
ifneq ($(shell id -u), 0)
	$(error "You are not root, full provision routine requires sudo")
else{machine_init_step}{ci_terraform_step}{all_dcs_vpn}{maybe_bootstrap_aws_private_subnets}
	$(RUN_AS_NON_ROOT) $(MAKE) ci-wait-ready-all-servers -j $(L1_PROVISIONING_JOBS)
	$(RUN_AS_NON_ROOT) $(MAKE) ci-all-servers-preconditions -j $(L1_PROVISIONING_JOBS)
endif

.PHONY: full-provision-pre-l2
full-provision-pre-l2:
	# initial l1 will fail because consul doesn't work yet
	$(RUN_AS_NON_ROOT) $(MAKE) l1-provision-with-wait -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID) -e L1_RESTART_CONSUL_POST_SECRETS=true -e L1_PROVISIONING_TOLERATE_REBUILD_FAIL=true{multi_dc_l1_prov}{cross_dc_ping}{restart_consul_masters}
	# consul is bootstrapped now
	$(RUN_AS_NON_ROOT) $(MAKE) consul-bootstrap_all-regions
	# l1 provision will fully succeed because it uses
	# consul to register services
	# but nomad and vault aren't bootstrapped yet
	$(RUN_AS_NON_ROOT) $(MAKE) l1-provision-with-wait -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID_3)
	$(RUN_AS_NON_ROOT) $(MAKE) nomad-bootstrap_all-regions
	$(RUN_AS_NON_ROOT) $(MAKE) vault-init_all-regions -j 1 # force only 1 job for init not to initialize multiple clusters at once
	$(RUN_AS_NON_ROOT) $(MAKE) vault-unseal_all-regions -j $(L1_PROVISIONING_JOBS)
	$(RUN_AS_NON_ROOT) $(MAKE) nomad-policies_all-regions
	$(RUN_AS_NON_ROOT) $(MAKE) vault-policies_all-regions
	# after propogate vault policy keys to hive.nix for l1 provisioning
	$(RUN_AS_NON_ROOT) $(MAKE) compile-project
	# after vault/nomad policies are set, provision correct nomad vault token
	# and restart nomad service
	$(RUN_AS_NON_ROOT) $(MAKE) l1-provision-with-wait -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID_4){maybe_multi_region_nomad_federation}

markers/consul-all-regions-bootstrapped: markers/all-regions.txt
	# initial l1 will fail because consul doesn't work yet
	$(RUN_AS_NON_ROOT) $(MAKE) ci-l1-provision -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID) -e L1_RESTART_CONSUL_POST_SECRETS=true -e L1_PROVISIONING_TOLERATE_REBUILD_FAIL=true{ci_multi_dc_l1_prov}{cross_dc_ping}{restart_consul_masters}
	# consul is bootstrapped now
	$(RUN_AS_NON_ROOT) $(MAKE) consul-bootstrap_all-regions
	$(RUN_AS_NON_ROOT) mkdir -p markers && touch markers/consul-all-regions-bootstrapped

markers/nomad-all-regions-bootstrapped: markers/all-regions.txt
	# l1 provision will fully succeed because it uses
	# consul to register services
	$(RUN_AS_NON_ROOT) $(MAKE) ci-l1-provision -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID_3)
	$(RUN_AS_NON_ROOT) $(MAKE) nomad-bootstrap_all-regions
	# but nomad and vault aren't bootstrapped yet
	$(RUN_AS_NON_ROOT) mkdir -p markers && touch markers/nomad-all-regions-bootstrapped

markers/vault-all-regions-initialized: markers/all-regions.txt
	$(RUN_AS_NON_ROOT) $(MAKE) vault-init_all-regions -j 1 # force only 1 job for init not to initialize multiple clusters at once
	$(RUN_AS_NON_ROOT) mkdir -p markers && touch markers/vault-all-regions-initialized

markers/nomad-provision-vault-token: markers/all-regions.txt
	# after propogate vault policy keys to hive.nix for l1 provisioning
	$(RUN_AS_NON_ROOT) $(MAKE) compile-project
	# after vault/nomad policies are set, provision correct nomad vault token
	# and restart nomad service
	$(RUN_AS_NON_ROOT) $(MAKE) ci-l1-provision -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID_4){maybe_multi_region_nomad_federation}
	$(RUN_AS_NON_ROOT) mkdir -p markers && touch markers/nomad-provision-vault-token

.PHONY: ci-full-provision-pre-l2
ci-full-provision-pre-l2:
	$(RUN_AS_NON_ROOT) $(MAKE) markers/consul-all-regions-bootstrapped -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(RUN_AS_NON_ROOT) $(MAKE) markers/nomad-all-regions-bootstrapped -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(RUN_AS_NON_ROOT) $(MAKE) markers/vault-all-regions-initialized
	$(RUN_AS_NON_ROOT) $(MAKE) ci-vault-unseal_all-regions -j $(L1_PROVISIONING_JOBS)
	# will not run unless region not processed
	$(RUN_AS_NON_ROOT) $(MAKE) nomad-policies_all-regions -j $(L1_PROVISIONING_JOBS)
	# will not run unless region not processed
	$(RUN_AS_NON_ROOT) $(MAKE) vault-policies_all-regions -j $(L1_PROVISIONING_JOBS)
	$(RUN_AS_NON_ROOT) $(MAKE) markers/nomad-provision-vault-token -j $(L1_PROVISIONING_JOBS)

.PHONY: full-provision
full-provision:
ifneq ($(shell id -u), 0)
	$(error "You are not root, full provision routine requires sudo")
else
	$(RUN_AS_NON_ROOT) $(MAKE) compile-project{maybe_dev_cache_steps}
	$(RUN_AS_NON_ROOT) $(MAKE) build-env-projects
	$(MAKE) full-provision-pre-l1
	$(RUN_AS_NON_ROOT) $(MAKE) full-provision-pre-l2
	# deploy nomad jobs and provision resources
	$(RUN_AS_NON_ROOT) $(MAKE) l2-provision
endif

.PHONY: full-provision-with-tests
full-provision-with-tests: full-provision
	$(RUN_AS_NON_ROOT) $(MAKE) -B integration-tests/grafana-instances-admin-passwords.txt
	$(RUN_AS_NON_ROOT) $(MAKE) wait-until-integration-tests

.PHONY: compile-project
compile-project:{maybe_depend_epl_exec_build}
	$(EPL_EXECUTABLE) compile \
		--output-directory $(MAKEFILE_DIRECTORY) \
		$(MAKEFILE_DIRECTORY)/data/main.edl

.PHONY: build-integration-tests
build-integration-tests:
	cd integration-tests; \
	cargo build --tests

.PHONY: l1-provision-with-wait
l1-provision-with-wait:
	$(RUN_AS_NON_ROOT) $(MAKE) l1-provision -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(RUN_AS_NON_ROOT) $(MAKE) wait-l1-provision -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)

{maybe_dev_vm_tasks}
{maybe_google_cloud_tasks}
{maybe_aws_tasks}

{maybe_build_epl_executable}

.PHONY: wait-until-integration-tests
wait-until-integration-tests:
	for I in $$(seq 1 77); \
	do \
		timeout 600s $(MAKE) integration-tests && exit 0; \
		echo Test round failed, sleeping for few seconds and retrying...; \
		sleep 10; \
	done; \
	echo Integration tests failed to succeed in a timeframe, exiting ; \
	exit 7

.PHONY: ci-universal-provision
ci-universal-provision:
ifneq ($(shell id -u), 0)
	$(error "You are not root, ci-universal-provision routine requires sudo (for VPN)")
else
	# if new scraped metrics targets are created
	$(RUN_AS_NON_ROOT) $(MAKE) compile-project
	$(RUN_AS_NON_ROOT) $(MAKE) scrape-planned-metrics
	$(RUN_AS_NON_ROOT) $(MAKE) refresh-static-server-infra-state
	$(RUN_AS_NON_ROOT) $(MAKE) refresh-l1-provisioning-state
	# we compile again to make decisions on scraped metrics, set main compilation environment
	# variable to user code would know this is the time to make all decisions based on present data
	$(RUN_AS_NON_ROOT) $(MAKE) compile-project -e EPL_MAIN_COMPILATION=true{maybe_dev_cache_steps}
	$(RUN_AS_NON_ROOT) $(MAKE) build-env-projects
	$(MAKE) ci-full-provision-pre-l1
	$(RUN_AS_NON_ROOT) $(MAKE) ci-full-provision-pre-l2
	$(RUN_AS_NON_ROOT) $(MAKE) ci-l1-provision
	$(RUN_AS_NON_ROOT) $(MAKE) l2-provision
endif

# nothing target in case target sql is evaluated and is empty
.PHONY: nothing
nothing:
	true

prometheus/scraped_metrics.sqlite:
	cat prometheus/db_schema.sql | sqlite3 prometheus/scraped_metrics.sqlite

.PHONY: ci-l1-provision
ci-l1-provision: prometheus/scraped_metrics.sqlite
	# provision all bootstrapped servers with fast provision
	$(MAKE) fast-l1-provision
	# bootstrap the rest of the servers that didn't receive provisioning
	# infra-state.sqlite added if empty servers returned from sqlite
	$(MAKE) -j $(L1_PROVISIONING_JOBS) \
	  -e L1_PROVISIONING_ID=$$( cat l1-fast/epl-prov-id ) \
	  nothing \
	  $$( echo " \
	        SELECT 'l1-provision-ww_' || hostname \
	        FROM servers_for_slow_l1_provision \
	      " | sqlite3 infra-state.sqlite )

infra-state.sqlite:
	$(LOAD_SHELL_LIB) init_infra_state_db

.PHONY: refresh-l1-provisioning-state
refresh-l1-provisioning-state:
	$(LOAD_SHELL_LIB) refresh_l1_provisioning_state

$(EPL_EXECUTABLE):{maybe_epl_exec_build}
	cd $(EPL_PROJECT_DIR) && cargo build
"#).unwrap();
}

fn makefile_l1_provisioning_part(checked: &CheckedDB, res: &mut String) {
    tag(res, "all servers l1 provision");
    *res += ".PHONY: l1-provision\n";
    *res += "l1-provision:";
    for server in checked.db.server().rows_iter() {
        let hostname = checked.db.server().c_hostname(server);
        *res += " l1-provision_";
        *res += hostname;
    }
    *res += "\n";

    tag(res, "separate servers l1 provision");
    let ssh_opts = "-o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10";
    for server in checked.db.server().rows_iter() {
        let hostname = checked.db.server().c_hostname(server);
        let ssh_iface = checked.db.server().c_ssh_interface(server);
        let ip = checked.db.network_interface().c_if_ip(ssh_iface);
        write!(res, ".PHONY: l1-provision_{hostname}\n").unwrap();
        write!(res, "l1-provision_{hostname}:\n").unwrap();
        write!(res, "\t$(LOAD_SHELL_LIB) cat $(or $(CUSTOM_SCRIPT),../l1-provisioning/{hostname}/provision.sh) | \\\n").unwrap();
        write!(res, "\t  sed 's/L1_EPL_PROVISIONING_ID/$(L1_PROVISIONING_ID)/g' | \\\n").unwrap();
        write!(res, "\t  sed 's/L1_PROVISIONING_TOLERATE_REBUILD_FAIL/$(L1_PROVISIONING_TOLERATE_REBUILD_FAIL)/g' | \\\n").unwrap();
        write!(res, "\t  sed 's/L1_RESTART_CONSUL_POST_SECRETS/$(L1_RESTART_CONSUL_POST_SECRETS)/g' | \\\n").unwrap();
        write!(res, "\t  gzip -9 | \\\n").unwrap();
        write!(res, "\t  ssh {ssh_opts} -i aux/root_ssh_key \\\n").unwrap();
        write!(res, "\t  admin@{ip} \"gunzip | sudo sh\"\n").unwrap();
    }
    *res += "\n";
}

fn makefile_l1_provisioning_wait_part(checked: &CheckedDB, res: &mut String) {
    tag(res, "wait all servers l1 provision");
    *res += ".PHONY: wait-l1-provision\n";
    *res += "wait-l1-provision:";
    for server in checked.db.server().rows_iter() {
        let hostname = checked.db.server().c_hostname(server);
        *res += " wait-l1-provision_";
        *res += hostname;
    }
    *res += "\n";

    tag(res, "wait separate servers l1 provision");
    for server in checked.db.server().rows_iter() {
        let dc = checked.db.server().c_dc(server);
        let region = checked.db.datacenter().c_region(dc);
        let region_name = checked.db.region().c_region_name(region);
        let hostname = checked.db.server().c_hostname(server);
        let ssh_iface = checked.db.server().c_ssh_interface(server);
        let ip = checked.db.network_interface().c_if_ip(ssh_iface);
        write!(res, ".PHONY: wait-l1-provision_{hostname}\n").unwrap();
        write!(res, "wait-l1-provision_{hostname}: infra-state.sqlite\n").unwrap();
        write!(res, "\t$(LOAD_SHELL_LIB) wait_l1_provisioning_finished $(L1_PROVISIONING_ID) {ip} {hostname} {region_name}\n").unwrap();
    }
    *res += "\n";
}

fn makefile_l1_provisioning_with_wait_parts(checked: &CheckedDB, res: &mut String) {
    tag(res, "separate servers l1 provision with wait");
    for server in checked.db.server().rows_iter() {
        let hostname = checked.db.server().c_hostname(server);
        write!(res, ".PHONY: l1-provision-ww_{hostname}\n").unwrap();
        write!(res, "l1-provision-ww_{hostname}:\n").unwrap();
        write!(res, "\t$(MAKE) l1-provision_{hostname} -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)\n").unwrap();
        write!(res, "\t$(MAKE) wait-l1-provision_{hostname} -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)\n").unwrap();
        write!(res, "# marker\n").unwrap();
        write!(res, "markers/l1-bootstrapped/{hostname}:\n").unwrap();
        write!(res, "\t$(MAKE) l1-provision_{hostname} -e L1_PROVISIONING_TOLERATE_REBUILD_FAIL=true -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)\n").unwrap();
        write!(res, "\t$(MAKE) wait-l1-provision_{hostname} -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)\n").unwrap();
        write!(res, "\tmkdir -p markers/l1-bootstrapped && touch markers/l1-bootstrapped/{hostname}\n").unwrap();
    }
    *res += "\n";
}

fn makefile_fast_l1_provisioning_part(checked: &CheckedDB, res: &mut String) {
    tag(res, "all servers fast l1 provision");
    *res += ".PHONY: fast-l1-provision\n";
    *res += "fast-l1-provision:";
    for region in checked.db.region().rows_iter() {
        if let Some(_first_srv) = first_region_server(&checked.db, region) {
            let region_name = &checked.db.region().c_region_name(region);
            *res += " fast-l1-provision_";
            *res += region_name;
        }
    }
    *res += "\n";

    tag(res, "separate regions fast l1 provision");
    for region in checked.db.region().rows_iter() {
        if let Some(server) = first_region_server(&checked.db, region) {
            let region_name = &checked.db.region().c_region_name(region);
            let ssh_iface = checked.db.server().c_ssh_interface(server);
            let ip = checked.db.network_interface().c_if_ip(ssh_iface);
            write!(res, ".PHONY: fast-l1-provision_{region_name}\n").unwrap();
            write!(res, "fast-l1-provision_{region_name}:\n").unwrap();
            write!(res, "\tif [ -e markers/fast-l1-needed/{region_name} ]; then cd l1-fast && tar -zcvf {region_name}.tar.gz {region_name}/consul-push.sh \\\n").unwrap();
            // all bootstrapped region servers
            write!(res, "\t $$( echo \" SELECT '{region_name}/plan_' || hostname || '.bin' FROM servers_for_fast_l1_provision WHERE region = '{region_name}' \" | sqlite3 ../infra-state.sqlite ); fi\n").unwrap();
            write!(res, "\tif [ -e markers/fast-l1-needed/{region_name} ]; then $(LOAD_SHELL_LIB) fast_l1_provisioning_for_region {ip} {region_name} ../l1-fast/{region_name}.tar.gz $$( cat ../l1-fast/epl-prov-id ) $(EPL_EXECUTABLE); fi\n").unwrap();
        }
    }
    *res += "\n";
}

fn makefile_refresh_server_infra_state(checked: &CheckedDB, res: &mut String, l1_outputs: &L1ProvOutputs) {
    tag(res, "all servers refresh in infra db");

    *res += ".PHONY: refresh-static-server-infra-state\n";
    *res += "refresh-static-server-infra-state: infra-state.sqlite\n";
    *res += "\techo \" \\\n";
    *res += "\t  DELETE FROM servers; \\\n";
    *res += "\t  INSERT INTO servers(hostname, region, expected_l1_hash) \\\n";
    *res += "\t  VALUES \\\n";
    for (idx, server) in checked.db.server().rows_iter().enumerate() {
        let dc = checked.db.server().c_dc(server);
        let region = checked.db.datacenter().c_region(dc);
        let region_name = checked.db.region().c_region_name(region);
        let is_last = idx + 1 == checked.db.server().len();
        *res += "\t    ('";
        *res += checked.db.server().c_hostname(server);
        *res += "','";
        *res += region_name;
        *res += "','";
        let server_l1_hash: &str = l1_outputs.regions.get(&region)
            .and_then(|reg_data| reg_data.get(&server))
            .and_then(|serv_data| Some(serv_data.provisioning_hash.as_str()))
            .unwrap_or("");
        *res += server_l1_hash;
        if !is_last {
            *res += "\'), \\\n";
        } else {
            *res += "\'); \\\n";
        }
    }
    *res += "\t\" | sqlite3 infra-state.sqlite\n";
    *res += "\n";
}

fn makefile_refresh_prom_metrics(checked: &CheckedDB, res: &mut String) {
    if checked.db.monitoring_cluster().len() == 0 {
        return;
    }

    tag(res, "refresh all prometheus clusters metrics");

    *res += ".PHONY: scrape-planned-metrics\n";
    *res += "scrape-planned-metrics:\n";
    *res += "\t# only scrape metrics if l2 provisioning finished\n";
    *res += "\ttest markers/l2-provisioning-done && $(EPL_EXECUTABLE) scrape-prometheus-metrics prometheus/metric_scrape_plan.yml prometheus/scraped_metrics.sqlite || true\n";
    *res += "\n";

    *res += ".PHONY: refresh-metrics-db\n";
    *res += "refresh-metrics-db:\n";
    *res += "\t$(EPL_EXECUTABLE) refresh-prometheus-metrics";
    for mon_c in checked.db.monitoring_cluster().rows_iter() {
        let cluster_name = checked.db.monitoring_cluster().c_cluster_name(mon_c);
        let prom_port = checked.db.monitoring_cluster().c_prometheus_port(mon_c);
        let inst = checked.db.monitoring_cluster().c_children_monitoring_instance(mon_c)[0];
        let vol = checked.db.monitoring_instance().c_monitoring_server(inst);
        let srv = checked.db.server_volume().c_parent(vol);
        let lan_iface = checked.projections.consul_network_iface.value(srv);
        let lan_ip = checked.db.network_interface().c_if_ip(*lan_iface);

        *res += " --prometheus-url ";
        *res += cluster_name;
        *res += ",http://";
        *res += lan_ip;
        *res += ":";
        write!(res, "{prom_port}").unwrap();
    }
    *res += " > metrics_db.yml.tmp\n";
    *res += "\tmv -f metrics_db.yml.tmp metrics_db.yml\n";
    *res += "\n";
}

fn makefile_preconditions_part(checked: &CheckedDB, res: &mut String) {
    let mut public_servers: Vec<TableRowPointerServer> = Vec::new();
    let mut private_servers: Vec<TableRowPointerServer> = Vec::new();
    for server in checked.db.server().rows_iter() {
        if checked.projections.internet_network_iface.contains_key(&server) {
            public_servers.push(server);
        } else {
            private_servers.push(server);
        }
    }

    tag(res, "public servers preconditions");
    *res += ".PHONY: public-servers-preconditions\n";
    *res += "public-servers-preconditions:";
    for server in &public_servers {
        let hostname = checked.db.server().c_hostname(*server);
        *res += " preconditions_";
        *res += hostname;
    }
    *res += "\n";

    tag(res, "ci public servers preconditions");
    *res += ".PHONY: ci-public-servers-preconditions\n";
    *res += "ci-public-servers-preconditions:\n";
    *res += "\t$(MAKE) -j $(L1_PROVISIONING_JOBS) ";
    for server in &public_servers {
        let hostname = checked.db.server().c_hostname(*server);
        *res += " markers/server-preconditions/";
        *res += hostname;
    }
    *res += "\n";

    tag(res, "private servers preconditions");
    *res += ".PHONY: private-servers-preconditions\n";
    *res += "private-servers-preconditions:";
    for server in &private_servers {
        let hostname = checked.db.server().c_hostname(*server);
        *res += " preconditions_";
        *res += hostname;
    }
    *res += "\n";

    tag(res, "all servers preconditions");
    *res += ".PHONY: all-servers-preconditions\n";
    *res += "all-servers-preconditions: public-servers-preconditions private-servers-preconditions";
    *res += "\n";

    tag(res, "ci all servers preconditions");
    *res += ".PHONY: ci-all-servers-preconditions\n";
    *res += "ci-all-servers-preconditions:\n";
    *res += "\t$(MAKE) -j $(L1_PROVISIONING_JOBS)";
    for server in checked.db.server().rows_iter() {
        let hostname = checked.db.server().c_hostname(server);
        write!(res, " markers/server-preconditions/{hostname}").unwrap();
    }

    tag(res, "separate servers preconditions");
    let ssh_opts = "-o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10";
    for server in checked.db.server().rows_iter() {
        let hostname = checked.db.server().c_hostname(server);
        let ssh_iface = checked.db.server().c_ssh_interface(server);
        let ip = checked.db.network_interface().c_if_ip(ssh_iface);
        write!(res, ".PHONY: preconditions_{hostname}\n").unwrap();
        write!(res, "preconditions_{hostname}:\n").unwrap();
        write!(res, "\t$(LOAD_SHELL_LIB) cat ../l1-provisioning/{hostname}/preconditions.sh | \\\n").unwrap();
        write!(res, "\t  gzip -9 | \\\n").unwrap();
        write!(res, "\t  ssh {ssh_opts} -i aux/root_ssh_key \\\n").unwrap();
        write!(res, "\t  admin@{ip} \"gunzip | sh\"\n").unwrap();
        write!(res, "\tmkdir -p markers/server-preconditions && touch markers/server-preconditions/{hostname}\n").unwrap();

        write!(res, "markers/server-preconditions/{hostname}: l1-provisioning/{hostname}/preconditions.sh\n").unwrap();
        write!(res, "\t$(MAKE) preconditions_{hostname}\n").unwrap();
    }
    *res += "\n";
}

fn makefile_regional_part(checked: &CheckedDB, res: &mut String) {
    tag(res, "all regions provisioning targets");

    let mut regional_target = |target: &str, has_marker: bool| {
        *res += ".PHONY: ";
        *res += target;
        *res += "_all-regions\n";
        *res += target;
        *res += "_all-regions:";
        if has_marker {
            *res += "\n";
            *res += "\t$(MAKE)";
            for region in checked.db.region().rows_iter() {
                *res += " markers/";
                *res += target;
                *res += "/";
                *res += checked.db.region().c_region_name(region);
            }
        } else {
            for region in checked.db.region().rows_iter() {
                *res += " ";
                *res += target;
                *res += "_region_";
                *res += checked.db.region().c_region_name(region);
            }
        }
        *res += "\n";
    };

    regional_target("consul-bootstrap", true);
    regional_target("nomad-bootstrap", true);
    regional_target("vault-init", true);
    regional_target("vault-unseal", false);
    regional_target("nomad-policies", true);
    regional_target("vault-policies", true);

    tag(res, "l2 provisioning all regions target");
    *res += ".PHONY: l2-provision\n";
    *res += "l2-provision:";
    for region in checked.db.region().rows_iter() {
        let region_name = checked.db.region().c_region_name(region);
        *res += " l2-provision_";
        *res += region_name;
    }
    *res += "\n";
    *res += "\tmkdir -p markers && touch markers/l2-provisioning-done\n";

    tag(res, "separate regions provisioning targets");
    let target = "consul-bootstrap";
    let ssh_opts = "-o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10";
    for region in checked.db.region().rows_iter() {
        let consul_server = checked.sync_res.network.consul_masters.get(&region).unwrap()[0];
        let network_iface = checked.projections.consul_network_iface.value(consul_server);
        let ip = checked.db.network_interface().c_if_ip(*network_iface);
        let region_name = checked.db.region().c_region_name(region);
        *res += ".PHONY: ";
        *res += target;
        *res += "_region_";
        *res += region_name;
        *res += "\n";

        *res += target;
        *res += "_region_";
        *res += region_name;
        *res += ":";

        write!(res, r#"
	$(LOAD_SHELL_LIB) \
	ssh admin@{ip} -i aux/root_ssh_key {ssh_opts} \
	  'epl-consul-bootstrap; epl-consul-vrrp-acl; epl-nomad-consul-acl-bootstrap; epl-vault-consul-acl-bootstrap'
"#).unwrap();

        write!(res, r#"
# marker
markers/{target}/{region_name}:
	$(MAKE) {target}_region_{region_name}
	mkdir -p markers/{target} && touch markers/{target}/{region_name}
"#).unwrap();
    }

    let target = "nomad-bootstrap";
    for region in checked.db.region().rows_iter() {
        let nomad_server = checked.sync_res.network.nomad_masters.get(&region).unwrap()[0];
        let network_iface = checked.projections.consul_network_iface.value(nomad_server);
        let ip = checked.db.network_interface().c_if_ip(*network_iface);
        let region_name = checked.db.region().c_region_name(region);
        *res += ".PHONY: ";
        *res += target;
        *res += "_region_";
        *res += region_name;
        *res += "\n";

        *res += target;
        *res += "_region_";
        *res += region_name;
        *res += ":";

        write!(res, r#"
	$(LOAD_SHELL_LIB) maybe_bootstrap_nomad {ip} {region_name} $(EPL_EXECUTABLE)
"#).unwrap();

        write!(res, r#"
# marker
markers/{target}/{region_name}:
	$(MAKE) {target}_region_{region_name}
	mkdir -p markers/{target} && touch markers/{target}/{region_name}
"#).unwrap();
    }

    let target = "vault-init";
    for region in checked.db.region().rows_iter() {
        let region_name = checked.db.region().c_region_name(region);
        *res += ".PHONY: ";
        *res += target;
        *res += "_region_";
        *res += region_name;
        *res += "\n";

        *res += target;
        *res += "_region_";
        *res += region_name;
        *res += ":";
        for dc in checked.db.region().c_referrers_datacenter__region(region) {
            for server in checked.db.datacenter().c_referrers_server__dc(*dc) {
                if checked.db.server().c_is_vault_instance(*server) {
                    *res += " ";
                    write!(res, "{}_region_{}_server_{}", target, region_name, checked.db.server().c_hostname(*server)).unwrap();
                    // only the first is initialized
                    break;
                }
            }
        }
        *res += "\n";

        for dc in checked.db.region().c_referrers_datacenter__region(region) {
            for server in checked.db.datacenter().c_referrers_server__dc(*dc) {
                if checked.db.server().c_is_vault_instance(*server) {
                    *res += ".PHONY: ";
                    write!(res, "{}_region_{}_server_{}", target, region_name, checked.db.server().c_hostname(*server)).unwrap();
                    *res += "\n";
                    write!(res, "{}_region_{}_server_{}", target, region_name, checked.db.server().c_hostname(*server)).unwrap();
                    *res += ":\n";
                    *res += "\t$(LOAD_SHELL_LIB) \\\n";
                    let network_iface = checked.projections.consul_network_iface.value(*server);
                    let fqdn = checked.projections.server_fqdns.value(*server);
                    let ip = checked.db.network_interface().c_if_ip(*network_iface);
                    *res += "\tmaybe_init_vault ";
                    *res += ip;
                    *res += " https://";
                    *res += fqdn;
                    *res += ":8200 ";
                    *res += region_name;
                    *res += " $(EPL_EXECUTABLE)";
                    *res += "\n";
                    // we only want to init one vault per region and rest sync from raft
                    break;
                }
            }
        }
        *res += "\techo Vault init done\n";

        write!(res, r#"
# marker
markers/{target}/{region_name}:
	$(MAKE) {target}_region_{region_name}
	mkdir -p markers/{target} && touch markers/{target}/{region_name}
"#).unwrap();
    }

    let target = "vault-unseal";
    for region in checked.db.region().rows_iter() {
        let region_name = checked.db.region().c_region_name(region);
        *res += ".PHONY: ";
        *res += target;
        *res += "_region_";
        *res += region_name;
        *res += "\n";

        *res += target;
        *res += "_region_";
        *res += region_name;
        *res += ":";
        for dc in checked.db.region().c_referrers_datacenter__region(region) {
            for server in checked.db.datacenter().c_referrers_server__dc(*dc) {
                if checked.db.server().c_is_vault_instance(*server) {
                    *res += " ";
                    write!(res, "{}_region_{}_server_{}", target, region_name, checked.db.server().c_hostname(*server)).unwrap();
                }
            }
        }
        *res += "\n";

        for dc in checked.db.region().c_referrers_datacenter__region(region) {
            for server in checked.db.datacenter().c_referrers_server__dc(*dc) {
                if checked.db.server().c_is_vault_instance(*server) {
                    *res += ".PHONY: ";
                    write!(res, "{}_region_{}_server_{}", target, region_name, checked.db.server().c_hostname(*server)).unwrap();
                    *res += "\n";
                    write!(res, "{}_region_{}_server_{}", target, region_name, checked.db.server().c_hostname(*server)).unwrap();
                    *res += ":\n";
                    *res += "\t$(LOAD_SHELL_LIB) \\\n";
                    let network_iface = checked.projections.consul_network_iface.value(*server);
                    let fqdn = checked.projections.server_fqdns.value(*server);
                    let ip = checked.db.network_interface().c_if_ip(*network_iface);
                    *res += "\tmaybe_unseal_vault ";
                    *res += ip;
                    *res += " https://";
                    *res += fqdn;
                    *res += ":8200 ";
                    *res += region_name;
                    *res += " $(EPL_EXECUTABLE)";
                    *res += "\n";
                }
            }
        }
        *res += "\techo Vault unseal done\n";
    }

    let target = "ci-vault-unseal";
    *res += ".PHONY: ";
    *res += target;
    *res += "_all-regions\n";
    *res += target;
    *res += "_all-regions:\n";
    *res += "\t$(MAKE) -j $(L1_PROVISIONING_JOBS)";
    for region in checked.db.region().rows_iter() {
        let region_name = checked.db.region().c_region_name(region);
        *res += " ";
        *res += target;
        *res += "_region_";
        *res += region_name;
    }
    *res += "\n";

    for region in checked.db.region().rows_iter() {
        let region_name = checked.db.region().c_region_name(region);
        let mon_c = checked.projections.monitoring_clusters.region_default(region);
        let maybe_mon_cluster = mon_c
            .map(|c| format!(" {}", checked.db.monitoring_cluster().c_cluster_name(c)))
            .unwrap_or_else(|| String::new());

        *res += ".PHONY: ";
        *res += target;
        *res += "_region_";
        *res += region_name;
        *res += "\n";

        *res += target;
        *res += "_region_";
        *res += region_name;
        *res += ":";
        for dc in checked.db.region().c_referrers_datacenter__region(region) {
            for server in checked.db.datacenter().c_referrers_server__dc(*dc) {
                if checked.db.server().c_is_vault_instance(*server) {
                    *res += " ";
                    write!(res, "{}_region_{}_server_{}", target, region_name, checked.db.server().c_hostname(*server)).unwrap();
                }
            }
        }
        *res += "\n";

        for dc in checked.db.region().c_referrers_datacenter__region(region) {
            for server in checked.db.datacenter().c_referrers_server__dc(*dc) {
                if checked.db.server().c_is_vault_instance(*server) {
                    *res += ".PHONY: ";
                    write!(res, "{}_region_{}_server_{}", target, region_name, checked.db.server().c_hostname(*server)).unwrap();
                    *res += "\n";
                    write!(res, "{}_region_{}_server_{}", target, region_name, checked.db.server().c_hostname(*server)).unwrap();
                    *res += ":\n";
                    // do l1 provisioning first if server was rebooted
                    *res += "\t$(MAKE) -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$$( cat l1-fast/epl-prov-id ) nothing \\\n";
                    *res += "\t $$( echo \" SELECT 'l1-provision-ww_' || hostname FROM servers_for_slow_l1_provision WHERE hostname = '";
                    *res += checked.db.server().c_hostname(*server);
                    *res += "' \" | sqlite3 infra-state.sqlite )\n";
                    let network_iface =
                        checked.projections.internet_network_iface.get(server)
                        // go through VPN if internet unavailable
                        .unwrap_or_else(|| checked.projections.consul_network_iface.value(*server));
                    let fqdn = checked.projections.server_fqdns.value(*server);
                    let ip = checked.db.network_interface().c_if_ip(*network_iface);
                    *res += "\t$(LOAD_SHELL_LIB) maybe_unseal_vault ";
                    *res += ip;
                    *res += " https://";
                    *res += fqdn;
                    *res += ":8200 ";
                    *res += region_name;
                    *res += " $(EPL_EXECUTABLE)";
                    *res += &maybe_mon_cluster;
                    *res += "\n";
                }
            }
        }
        *res += "\techo Vault unseal done\n";
    }

    let target = "nomad-policies";
    for region in checked.db.region().rows_iter() {
        let nomad_server = checked.sync_res.network.nomad_masters.get(&region).unwrap()[0];
        let network_iface = checked.projections.consul_network_iface.value(nomad_server);
        let ip = checked.db.network_interface().c_if_ip(*network_iface);
        let region_name = checked.db.region().c_region_name(region);
        *res += ".PHONY: ";
        *res += target;
        *res += "_region_";
        *res += region_name;
        *res += "\n";

        *res += target;
        *res += "_region_";
        *res += region_name;
        *res += ":";

        write!(res, r#"
	$(LOAD_SHELL_LIB) nomad_policies_provision {ip} {region_name} $(EPL_EXECUTABLE)
"#).unwrap();

        write!(res, r#"
# marker
markers/{target}/{region_name}:
	$(MAKE) {target}_region_{region_name}
	mkdir -p markers/{target} && touch markers/{target}/{region_name}
"#).unwrap();
    }

    let target = "vault-policies";
    for region in checked.db.region().rows_iter() {
        let vault_server = checked.sync_res.network.vault_masters.get(&region).unwrap()[0];
        let network_iface = checked.projections.consul_network_iface.value(vault_server);
        let ip = checked.db.network_interface().c_if_ip(*network_iface);
        let region_name = checked.db.region().c_region_name(region);
        *res += ".PHONY: ";
        *res += target;
        *res += "_region_";
        *res += region_name;
        *res += "\n";

        *res += target;
        *res += "_region_";
        *res += region_name;
        *res += ":";

        write!(res, r#"
	$(LOAD_SHELL_LIB) vault_nomad_policies_provision {ip} {region_name} $(EPL_EXECUTABLE)
	$(LOAD_SHELL_LIB) vault_acme_policies_provision {ip} {region_name} $(EPL_EXECUTABLE)
"#).unwrap();

        write!(res, r#"
# marker
markers/{target}/{region_name}:
	$(MAKE) {target}_region_{region_name}
	mkdir -p markers/{target} && touch markers/{target}/{region_name}
"#).unwrap();
    }

    for region in checked.db.region().rows_iter() {
        let provisioning_server = checked.projections.provisioning_server_in_region.value(region).unwrap();
        let network_iface = checked.projections.consul_network_iface.value(provisioning_server);
        let ip = checked.db.network_interface().c_if_ip(*network_iface);
        let region_name = checked.db.region().c_region_name(region);
        *res += ".PHONY: l2-provision_";
        *res += region_name;
        *res += "\n";
        *res += "l2-provision_";
        *res += region_name;
        *res += ":";
        write!(res, r#"
	$(LOAD_SHELL_LIB) EPL_EXECUTABLE=$(EPL_EXECUTABLE) ./provision {region_name} {ip}
"#).unwrap();
    }

    if checked.db.region().len() > 1 {
        let mut iter = checked.db.region().rows_iter();
        let first_region = iter.next().unwrap();
        let first_region_nomad_servers = checked.sync_res.network.nomad_masters.get(&first_region).unwrap();
        *res += ".PHONY: nomad-cross-region-federation\n";
        *res += "nomad-cross-region-federation:\n";
        if let Some(first_server) = first_region_nomad_servers.get(0) {
            let to_server_iface = checked.projections.consul_network_iface.value(*first_server);
            let to_server_ip = checked.db.network_interface().c_if_ip(*to_server_iface);
            for other_region in iter {
                let other_region_name = checked.db.region().c_region_name(other_region);
                let other_region_nomad_servers = checked.sync_res.network.nomad_masters.get(&other_region).unwrap();
                if let Some(other_server) = other_region_nomad_servers.get(0) {
                    let from_server_iface = checked.projections.consul_network_iface.value(*other_server);
                    let from_server_ip = checked.db.network_interface().c_if_ip(*from_server_iface);
                    write!(res, "\t$(LOAD_SHELL_LIB) EPL_EXECUTABLE=$(EPL_EXECUTABLE) nomad_server_join {to_server_ip} {from_server_ip} {other_region_name}\n").unwrap();
                }
            }
        }

        *res += "markers/cross-region-federation: markers/all-regions.txt\n";
        *res += "\t$(MAKE) nomad-cross-region-federation\n";
        *res += "\ttouch markers/cross-region-federation\n";
    }
}

fn makefile_networks_part(checked: &CheckedDB, dev_vms_exist: bool, res: &mut String) {
    if dev_vms_exist {
        tag(res, "up all networks");
        *res += ".PHONY: up-vm-networks\n";
        *res += "up-vm-networks: ensure-bridge-nf-tables-set";
        for (n, _) in &checked.sync_res.network.libvirt_network_topology.networks {
            *res += " up-net_";
            *res += n;
        }
        *res += " up-libvirt-nat-disable-rule\n";

        tag(res, "down all networks");
        *res += ".PHONY: down-vm-networks\n";
        *res += "down-vm-networks: down-libvirt-nat-disable-rule ";
        for (n, _) in &checked.sync_res.network.libvirt_network_topology.networks {
            *res += " down-net_";
            *res += n;
        }
        *res += "\n";

        tag(res, "up every network");
        for (n, _) in &checked.sync_res.network.libvirt_network_topology.networks {
            *res += ".PHONY: up-net_";
            *res += n;
            *res += "\n";
            *res += "up-net_";
            *res += n;
            *res += ":";
            write!(res, r#"
ifneq ($(shell id -u), 0)
	$(error "You are not root, to create vm networks root is required")
else
	virsh net-list --all | grep {n} || \
	  virsh net-define servers/networks/{n}.xml
	virsh net-list | grep {n} || \
	  virsh net-start {n}
endif
"#).unwrap();
        }

        tag(res, "down every network");
        for (n, _) in &checked.sync_res.network.libvirt_network_topology.networks {
            *res += ".PHONY: down-net_";
            *res += n;
            *res += "\n";
            *res += "down-net_";
            *res += n;
            *res += ":";
            write!(res, r#"
ifneq ($(shell id -u), 0)
	$(error "You are not root, to remove vm networks root is required")
else
	virsh net-list | grep {n} && \
	  virsh net-destroy {n} || true
	virsh net-list --all | grep {n} && \
	  virsh net-undefine {n} || true
endif
"#).unwrap();
        }
    }

    if checked.db.datacenter().len() > 1 {
        tag(res, "wait for cross dc ping all regions");
        *res += ".PHONY: wait-cross-dc-ping_all-regions\n";
        *res += "wait-cross-dc-ping_all-regions:";
        for region in checked.db.region().rows_iter() {
            let region_name = checked.db.region().c_region_name(region);
            *res += " wait-cross-dc-ping_region_";
            *res += region_name;
        }
        *res += "\n";

        tag(res, "wait for cross dc conection inside regions");
        for region in checked.db.region().rows_iter() {
            let region_name = checked.db.region().c_region_name(region);
            *res += ".PHONY: wait-cross-dc-ping_region_";
            *res += region_name;
            *res += "\n";
            *res += "wait-cross-dc-ping_region_";
            *res += region_name;
            *res += ":\n";
            for pinger_dc in checked.db.region().c_referrers_datacenter__region(region) {
                for pingee_dc in checked.db.region().c_referrers_datacenter__region(region) {
                    if pinger_dc != pingee_dc {
                        if !checked.db.datacenter().c_referrers_server__dc(*pinger_dc).is_empty()
                            && !checked.db.datacenter().c_referrers_server__dc(*pingee_dc).is_empty()
                        {
                            let pinger_srv = checked.db.datacenter().c_referrers_server__dc(*pinger_dc)[0];
                            let pingee_srv = checked.db.datacenter().c_referrers_server__dc(*pingee_dc)[0];
                            let ssh_iface = checked.db.server().c_ssh_interface(pinger_srv);
                            let ssh_ip = checked.db.network_interface().c_if_ip(ssh_iface);

                            let pinger_ip = checked.db.network_interface().c_if_ip(*checked.projections.consul_network_iface.value(pinger_srv));
                            let pingee_ip = checked.db.network_interface().c_if_ip(*checked.projections.consul_network_iface.value(pingee_srv));
                            write!(res, "\t$(LOAD_SHELL_LIB) wait_until_ping_succeeds {ssh_ip} {pinger_ip} {pingee_ip}\n").unwrap();
                        }
                    }
                }
            }
            *res += "\n";
        }

        // needed because consul doesn't bootstrap
        // after wireguard is setup
        tag(res, "restart consul masters for all regions");
        *res += "restart-consul-masters_all-regions:";
        for region in checked.db.region().rows_iter() {
            let region_name = checked.db.region().c_region_name(region);
            *res += " restart-consul-masters_region_";
            *res += region_name;
        }
        *res += "\n";

        tag(res, "restart consul masters by region");
        let ssh_opts = "-o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10";
        for region in checked.db.region().rows_iter() {
            let region_name = checked.db.region().c_region_name(region);
            *res += ".PHONY: restart-consul-masters_region_";
            *res += region_name;
            *res += "\n";
            *res += "restart-consul-masters_region_";
            *res += region_name;
            *res += ":";
            for dc in checked.db.region().c_referrers_datacenter__region(region) {
                for srv in checked.db.datacenter().c_referrers_server__dc(*dc) {
                    if checked.db.server().c_is_consul_master(*srv) {
                        let ip = checked.db.network_interface().c_if_ip(*checked.projections.consul_network_iface.value(*srv));
                        write!(res, r#"
	$(LOAD_SHELL_LIB) \
	ssh admin@{ip} -i aux/root_ssh_key {ssh_opts} \
	  'consul members || sudo systemctl restart consul.service && sleep 1'
"#).unwrap();
                    }
                }
            }
        }
        *res += "\n";
    }
}

fn tag(res: &mut String, tag: &str) {
    *res += "\n";
    *res += "#############################################\n";
    *res += "# ";
    *res += tag;
    *res += "\n";
    *res += "#############################################\n";
}

pub fn vms_exist(checked: &CheckedDB) -> bool {
    for dc in checked.db.datacenter().rows_iter() {
        if checked.db.datacenter().c_implementation(dc) == "testvms" {
            if !checked.db.datacenter().c_referrers_server__dc(dc).is_empty() {
                return true;
            }
        }
    }

    return false;
}

fn makefile_aws_public_subnet_bootstrap_internet(checked: &CheckedDB, res: &mut String) {
    if checked.projections.cloud_topologies.aws.dcs.len() > 0 {
        tag(res, "aws public subnets internet bootstrap");
        *res += ".PHONY: aws-private-ips-bootstrap-internet\n";
        *res += "aws-private-ips-bootstrap-internet:\n";

        // always first bootstrap dcrouter nodes
        let mut lines_to_push: Vec<(usize, String)> = Vec::new();
        for (dc, dc_data) in &checked.projections.cloud_topologies.aws.dcs {
            let dc_net = checked.sync_res.network.networking_answers.dcs.get(&dc).unwrap();
            let router_set = dc_net.all_routers_set();
            let dc_cidr = checked.db.datacenter().c_network_cidr(*dc);

            for subnet in dc_data.subnet_map.keys() {
                let subnet_exp = &format!("{subnet}.0/24");
                let first_gw = &format!("{subnet}.1");
                let subnet_to_check: ipnet::Ipv4Net = subnet_exp.parse().unwrap();
                let sub_routing = dc_net.subnets.get(&subnet_to_check).unwrap();
                let hosts = checked.sync_res.network.subnets_to_interfaces_map.get(&subnet_to_check);
                if let Some(hosts) = hosts {
                    for iface in &hosts.interfaces {
                        let srv = checked.db.network_interface().c_parent(*iface);
                        let hostname = checked.db.server().c_hostname(srv);
                        let ri = &sub_routing.routing_interfaces[0];
                        let gw_ip = checked.db.network_interface().c_if_ip(ri.lan_iface);
                        let has_internet = checked.projections.internet_network_iface.get(&srv).is_some();
                        let is_router = router_set.contains(&srv);
                        let private_ip =
                            if is_router {
                                let dcrouter_iface =
                                    checked.db.server().c_children_network_interface(srv)
                                                       .iter()
                                                       .find(|i| {
                                                           let network = checked.db.network_interface().c_if_network(**i);
                                                           "dcrouter" == checked.db.network().c_network_name(network)
                                                       });
                                if let Some(dcrouter_iface) = dcrouter_iface {
                                    checked.db.network_interface().c_if_ip(*dcrouter_iface)
                                } else {
                                    checked.db.network_interface().c_if_ip(*iface)
                                }
                            } else {
                                checked.db.network_interface().c_if_ip(*iface)
                            };
                        if !has_internet {
                            let route_to_ip =
                                if is_router {
                                    checked.db.network_interface().c_if_ip(
                                        dc_net.routers_with_internet_interfaces[0]
                                    )
                                } else {
                                    gw_ip
                                };
                            let priority =
                                if is_router { 100 } else { 1000 };
                            let maybe_ip_fwd =
                                if is_router { " true" } else { "" };
                            lines_to_push.push((priority, format!(
                                "\t$(LOAD_SHELL_LIB) aws_bootstrap_private_node_internet {hostname} {private_ip} {route_to_ip} 1.1.1.1 {dc_cidr} {first_gw}{maybe_ip_fwd}\n"
                            )));
                        }
                    }
                }
            }
        }

        // sort to bootstrap by priority
        lines_to_push.sort();
        for line in &lines_to_push {
            *res += &line.1;
        }

        *res += "\n";
    }
}

fn makefile_vpn_part(res: &mut String) {
    tag(res, "vpn");

    *res += ".PHONY: all-dcs-start-vpn\n";
    *res += "all-dcs-start-vpn:\n";
    *res += r#"
ifneq ($(shell id -u), 0)
	$(error "You are not root, to create wireguard VPN networks root is required")
else
	which wg
	-ip link add dev wg7 type wireguard
	-ip address add dev wg7 172.21.7.254/24
	wg setconf wg7 $(MAKEFILE_DIRECTORY)/admin-wg.conf
	ip link set up dev wg7
	wg show wg7
	ip route add 10.0.0.0/8 dev wg7 || true
endif
"#;

    *res += ".PHONY: all-dcs-stop-vpn\n";
    *res += "all-dcs-stop-vpn:\n";
    *res += r#"
ifneq ($(shell id -u), 0)
	$(error "You are not root, to create vm networks root is required")
else
	-ip route del 10.0.0.0/8 dev wg7
	-ip link del dev wg7
"#;

    *res += r#"
endif
"#;

    *res += "\n";
}

fn makefile_terraform_part(checked: &CheckedDB, res: &mut String) {
    let global_settings = get_global_settings(&checked.db);
    tag(res, "terraform");

    let mut provision_steps = String::new();
    let mut destroy_steps = String::new();
    let mut ci_provision_steps = String::new();
    let mut add_cloud = |cloud: &str| {
        let _ = write!(&mut provision_steps, " terraform-provision-{cloud}").unwrap();
        let _ = write!(&mut ci_provision_steps, " terraform/{cloud}/terraform.tfstate").unwrap();
        let _ = write!(&mut destroy_steps, " terraform-destroy-{cloud}").unwrap();
    };
    if !checked.projections.cloud_topologies.aws.is_empty() {
        add_cloud("aws");
    }
    if !checked.projections.cloud_topologies.gcloud.is_empty() {
        add_cloud("gcloud");
    }

    // provision
    *res += ".PHONY: terraform-provision-all-clouds\n";
    *res += "terraform-provision-all-clouds:";
    *res += &provision_steps;
    *res += "\n";

    // provision ci
    *res += ".PHONY: ci-terraform-provision-all-clouds\n";
    *res += "ci-terraform-provision-all-clouds:\n";
    *res += "\t$(MAKE) -j $(L1_PROVISIONING_JOBS)";
    *res += &ci_provision_steps;
    *res += "\n";

    // destroy, for testing
    *res += ".PHONY: terraform-destroy-all-clouds\n";
    *res += "terraform-destroy-all-clouds:";
    *res += &destroy_steps;
    *res += "\n";

    if !checked.projections.cloud_topologies.aws.is_empty() {
        *res += ".PHONY: terraform-provision-aws\n";
        *res += "terraform-provision-aws: aws-images\n";
        *res += "\tcd terraform/aws && stat .terraform.lock.hcl || terraform init\n";
        *res += "\tcd terraform/aws && terraform apply -auto-approve\n";
        *res += "\tcd terraform/aws && cat terraform.tfstate | jq '{\"network_interface\":[ .resources[] | select(.type | contains(\"aws_instance\")) | select(.instances[0].attributes.public_ip != \"\") | {\"primary_key\": (.name + \"=>void\"),\"replacements\":{\"if_ip\":.instances[0].attributes.public_ip}} ], \"server\": [.resources[] | select(.type | contains(\"aws_instance\")) | select(.instances[0].attributes.ipv6_addresses[0]) | {\"primary_key\":.name,\"replacements\":{\"public_ipv6_address\":.instances[0].attributes.ipv6_addresses[0]}}]}' > replacements.json ; \\\n";
        if global_settings.update_edl_public_ips_from_terraform {
            *res += "\t$(EDENDB_EXECUTABLE) --replacements-file replacements.json ../../data/main.edl $(EPL_PROJECT_DIR)/edb-src/main.edl\n";
        } else {
            *res += "\n";
        }

        write!(res, r#"
terraform/aws/terraform.tfstate: terraform/aws/main.tf
	$(MAKE) terraform-provision-aws
"#).unwrap();

        *res += ".PHONY: terraform-destroy-aws\n";
        *res += "terraform-destroy-aws:\n";
        *res += "\tcd terraform/aws && terraform destroy $(TF_DESTROY_FLAGS)\n";
    }

    if !checked.projections.cloud_topologies.gcloud.is_empty() {
        *res += ".PHONY: terraform-provision-gcloud\n";
        *res += "terraform-provision-gcloud: gcloud-images\n";
        *res += "\tcd terraform/gcloud && stat .terraform.lock.hcl || terraform init\n";
        *res += "\tcd terraform/gcloud && terraform apply -auto-approve\n";
        *res += "\tcd terraform/gcloud && cat terraform.tfstate | jq '{\"network_interface\":[ .resources[] | select(.type | contains(\"google_compute_instance\")) | select(.instances[0].attributes.network_interface[0].access_config[0].nat_ip != null) | { \"primary_key\": (.name + \"=>void\"), \"replacements\":{\"if_ip\":.instances[0].attributes.network_interface[0].access_config[0].nat_ip} } ]}' > replacements.json ; \\\n";
        if global_settings.update_edl_public_ips_from_terraform {
            *res += "\t$(EDENDB_EXECUTABLE) --replacements-file replacements.json ../../data/main.edl $(EPL_PROJECT_DIR)/edb-src/main.edl\n";
        } else {
            *res += "\n";
        }

        write!(res, r#"
terraform/gcloud/terraform.tfstate: terraform/gcloud/main.tf
	$(MAKE) terraform-provision-gcloud
"#).unwrap();

        *res += ".PHONY: terraform-destroy-gcloud\n";
        *res += "terraform-destroy-gcloud:\n";
        *res += "\tcd terraform/gcloud && terraform destroy $(TF_DESTROY_FLAGS)\n";
    }

    // public hosts provisiong
    if checked.projections.cloud_topologies.cloud_needed() {
        let mut servers = Vec::new();
        for dc in checked.db.datacenter().rows_iter() {
            if ["aws", "gcloud"].contains(&checked.db.datacenter().c_implementation(dc).as_str()) {
                for server in checked.db.datacenter().c_referrers_server__dc(dc) {
                    if checked.projections.internet_network_iface.get(server).is_some() {
                        servers.push(*server);
                    }
                }
            }
        }

        let bootstrap_targets = servers
            .iter()
            .map(|srv| format!("markers/l1-bootstrapped/{}", checked.db.server().c_hostname(*srv)))
            .collect::<Vec<_>>()
            .join(" ");
        let server_ready_targets = servers
            .iter()
            .map(|srv| format!("markers/server-ready/{}", checked.db.server().c_hostname(*srv)))
            .collect::<Vec<_>>()
            .join(" ");

        *res += ".PHONY: ci-bootstrap-l1-public-nodes\n";
        *res += "ci-bootstrap-l1-public-nodes:\n";
        *res += "\t$(MAKE) -j $(L1_PROVISIONING_JOBS) ";
        *res += &server_ready_targets;
        *res += "\n";
        *res += "\t$(MAKE) -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID) ";
        *res += &bootstrap_targets;
        *res += "\n";
        *res += "\n";

        *res += ".PHONY: l1-provision-public-servers\n";
        *res += "l1-provision-public-servers:";
        for server in &servers {
            *res += " l1-provision_";
            *res += checked.db.server().c_hostname(*server);
        }
        *res += "\n";

        *res += ".PHONY: wait-l1-provision-public-servers\n";
        *res += "wait-l1-provision-public-servers:";
        for server in &servers {
            *res += " wait-l1-provision_";
            *res += checked.db.server().c_hostname(*server);
        }
        *res += "\n";
        *res += "\n";
    }

    *res += "\n";
}

fn makefile_server_part(checked: &CheckedDB, dev_vms_exist: bool, res: &mut String) {
    tag(res, "wait ready public servers");
    *res += ".PHONY: wait-ready-public-servers\n";
    *res += "wait-ready-public-servers:";
    for server in checked.db.server().rows_iter() {
        if checked.projections.internet_network_iface.contains_key(&server) {
            *res += " wait-ready_";
            *res += checked.db.server().c_hostname(server);
        }
    }
    *res += "\n";

    tag(res, "ci wait ready public servers");
    *res += ".PHONY: ci-wait-ready-public-servers\n";
    *res += "ci-wait-ready-public-servers:\n";
    *res += "\t$(MAKE) -j $(L1_PROVISIONING_JOBS)";
    for server in checked.db.server().rows_iter() {
        if checked.projections.internet_network_iface.contains_key(&server) {
            *res += " markers/server-ready/";
            *res += checked.db.server().c_hostname(server);
        }
    }
    *res += "\n";

    tag(res, "wait ready all servers");
    *res += ".PHONY: wait-ready-all-servers\n";
    *res += "wait-ready-all-servers:";
    for server in checked.db.server().rows_iter() {
        *res += " wait-ready_";
        *res += checked.db.server().c_hostname(server);
    }
    *res += "\n";

    tag(res, "ci wait ready all servers with completion markers");
    *res += ".PHONY: ci-wait-ready-all-servers\n";
    *res += "ci-wait-ready-all-servers:\n";
    *res += "\t$(MAKE) -j $(L1_PROVISIONING_JOBS)";
    for server in checked.db.server().rows_iter() {
        *res += " markers/server-ready/";
        *res += checked.db.server().c_hostname(server);
    }
    *res += "\n";

    if dev_vms_exist {
        tag(res, "run all servers");
        *res += ".PHONY: run-all-servers\n";
        *res += "run-all-servers: up-vm-networks";
        for server in checked.db.server().rows_iter() {
            *res += " run-server_";
            *res += checked.db.server().c_hostname(server);
        }
        *res += "\n";

        tag(res, "stop all servers");
        *res += ".PHONY: stop-all-servers\n";
        *res += "stop-all-servers:";
        for server in checked.db.server().rows_iter() {
            *res += " stop-server_";
            *res += checked.db.server().c_hostname(server);
        }
        *res += "\n";

        tag(res, "create all vm disks");
        *res += ".PHONY: create-vm-disks\n";
        *res += "create-vm-disks:";
        for server in checked.db.server().rows_iter() {
            for disk in checked.db.server().c_children_server_disk(server) {
                let disk_id = checked.db.server_disk().c_disk_id(*disk);
                *res += " servers/disks/";
                *res += checked.db.server().c_hostname(server);
                *res += "_";
                *res += disk_id;
                *res += ".qcow";
            }
        }
        *res += "\n";

        tag(res, "create every server disk targets");
        for server in checked.db.server().rows_iter() {
            let sk = checked.projections.server_kinds.value(server);
            let arch = checked.db.server_kind().c_architecture(*sk);
            let hostname = checked.db.server().c_hostname(server);
            let root_disk = checked.db.server().c_root_disk(server);
            for disk in checked.db.server().c_children_server_disk(server) {
                let disk_id = checked.db.server_disk().c_disk_id(*disk);
                let capacity_bytes = checked.projections.server_disk_sizes.get(disk).unwrap();
                let is_root = root_disk == *disk;
                let maybe_vm_template =
                    if is_root {
                        format!(" servers/vm-template-{arch}.txt")
                    } else { "".to_string() };
                *res += "servers/disks/";
                *res += hostname;
                *res += "_";
                *res += disk_id;
                write!(res, ".qcow:{maybe_vm_template}\n").unwrap();
                *res += "\t$(LOAD_SHELL_LIB) prepare_disk_img disks/";
                *res += hostname;
                *res += "_";
                *res += disk_id;
                *res += ".qcow";
                *res += " ";
                let maybe_vm_template =
                    if is_root {
                        format!(" vm-template-{arch}.txt")
                    } else { "".to_string() };
                write!(res, "{}{}", capacity_bytes, maybe_vm_template).unwrap();
                *res += "\n";
            }
        }

        tag(res, "run every server targets");
        for server in checked.db.server().rows_iter() {
            let kind = checked.projections.server_kinds.value(server);
            let arch = checked.db.server_kind().c_architecture(*kind);
            let run_arch = epl_arch_to_linux_arch(arch);
            let hostname = checked.db.server().c_hostname(server);
            let root_disk = checked.db.server().c_root_disk(server);
            let root_disk_id = checked.db.server_disk().c_disk_id(root_disk);
            let memory_mb = checked.db.server_kind().c_memory_bytes(*kind) / 1024 / 1024;
            let cores = checked.db.server_kind().c_cores(*kind);
            *res += ".PHONY: run-server_";
            *res += hostname;
            *res += "\n";
            *res += "run-server_";
            *res += hostname;
            *res += ": up-vm-networks |";
            for server_disk in checked.db.server().c_children_server_disk(server) {
                let disk_id = checked.db.server_disk().c_disk_id(*server_disk);
                *res += " servers/disks/";
                *res += hostname;
                *res += "_";
                *res += disk_id;
                *res += ".qcow";
            }
            write!(res, r#"
ifneq ($(shell id -u), 0)
	$(error "You are not root, to create vms root is required")
else
	$(LOAD_SHELL_LIB) start_server {hostname} {memory_mb} {cores} {run_arch} \
"#).unwrap();

            let root_serial = root_disk_id.strip_prefix("virtio-").unwrap_or(root_disk_id.as_str());
            // first root disk
            write!(res, "\t  --disk path=disks/{hostname}_{root_disk_id}.qcow,device=disk,serial={root_serial} \\\n").unwrap();
            for disk in checked.db.server().c_children_server_disk(server) {
                if *disk != root_disk {
                    let disk_id = checked.db.server_disk().c_disk_id(*disk);
                    let disk_serial = disk_id.strip_prefix("virtio-").unwrap_or(disk_id.as_str());
                    write!(res, "\t  --disk path=disks/{hostname}_{disk_id}.qcow,device=disk,serial={disk_serial} \\\n").unwrap();
                }
            }
            let mut interfaces: Vec<_> = checked.sync_res.network.libvirt_network_topology.networks.values().collect();
            // it happens that we're sorting by two keys
            // internet and lan, lan happens to be behind the internet
            // but we want lan to always be first so we reverse after sorting
            interfaces.sort_by_key(|i| checked.db.network().c_network_name(i.network));
            interfaces.reverse();
            for n in &interfaces {
                if let Some(interface) = n.servers.get(&server) {
                    *res += "\t  ";
                    *res += "--network ";
                    *res += "network=";
                    *res += &n.libvirt_name;
                    *res += ",mac=";
                    write!(res, "{}", interface.mac.to_hex_string()).unwrap();
                    *res += " \\\n";
                }
            }
            *res += "\t&& true\n";
            *res += "endif\n";
        }

        tag(res, "stop every separate server targets");
        for server in checked.db.server().rows_iter() {
            let hostname = checked.db.server().c_hostname(server);
            *res += ".PHONY: stop-server_";
            *res += hostname;
            *res += "\n";
            *res += "stop-server_";
            *res += hostname;
            *res += ":";
            write!(res, r#"
ifneq ($(shell id -u), 0)
	$(error "You are not root, to stop vms root is required")
else
	$(LOAD_SHELL_LIB) stop_server {hostname}
endif
"#).unwrap();
        }

        tag(res, "teardown every separate server targets");
        for server in checked.db.server().rows_iter() {
            let hostname = checked.db.server().c_hostname(server);
            *res += ".PHONY: teardown-server_";
            *res += hostname;
            *res += "\n";
            *res += "teardown-server_";
            *res += hostname;
            *res += ": stop-server_";
            *res += hostname;
            for server_disk in checked.db.server().c_children_server_disk(server) {
                let disk_id = checked.db.server_disk().c_disk_id(*server_disk);
                write!(res, r#"
	rm -f servers/disks/{hostname}_{disk_id}.qcow"#).unwrap();
            }
            *res += "\n";
        }
    }

    tag(res, "wait ready separate server targets");
    for server in checked.db.server().rows_iter() {
        let ip = checked.db.network_interface().c_if_ip(checked.db.server().c_ssh_interface(server));
        let hostname = checked.db.server().c_hostname(server);
        *res += ".PHONY: wait-ready_";
        *res += hostname;
        *res += "\n";
        *res += "wait-ready_";
        *res += hostname;
        *res += ":";
        write!(res, r#"
	$(LOAD_SHELL_LIB) ensure_server_ready {ip}
"#).unwrap();
        *res += "# marker\n";
        write!(res, "markers/server-ready/{hostname}:\n").unwrap();
        write!(res, "\t$(MAKE) wait-ready_{hostname}\n").unwrap();
        write!(res, "\tmkdir -p markers/server-ready && touch markers/server-ready/{hostname}\n").unwrap();
    }

    tag(res, "login to servers");
    let ssh_opts = "-o ServerAliveInterval=10 -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10";
    for server in checked.db.server().rows_iter() {
        let ssh_iface = checked.db.server().c_ssh_interface(server);
        let ip = checked.db.network_interface().c_if_ip(ssh_iface);
        let hostname = checked.db.server().c_hostname(server);
        *res += ".PHONY: login_";
        *res += hostname;
        *res += "\n";
        *res += "login_";
        *res += hostname;
        *res += ":";
        write!(res, r#"
	ssh {ssh_opts} -i servers/aux/root_ssh_key admin@{ip}
"#).unwrap();
    }
}

fn makefile_compile_environments_part(checked: &CheckedDB, res: &mut String) {
    tag(res, "compile environments");
    *res += ".PHONY: build-compile-environments\n";
    *res += "build-compile-environments:";
    for ce in checked.db.rust_compilation_environment().rows_iter() {
        let ce_name = checked.db.rust_compilation_environment().c_env_name(ce);
        write!(res, " comp-envs/{ce_name}/Cargo.lock").unwrap();
    }
    *res += "\n";
    *res += "\n";

    for ce in checked.db.rust_compilation_environment().rows_iter() {
        let ce_name = checked.db.rust_compilation_environment().c_env_name(ce);
        write!(res, r#"
comp-envs/{ce_name}/Cargo.lock: comp-envs/{ce_name}/Cargo.toml
	cd comp-envs/{ce_name} && cargo generate-lockfile
"#).unwrap();
    }
}

fn makefile_apps_part(checked: &CheckedDB, res: &mut String) {
    tag(res, "build all apps target");
    *res += ".PHONY: build-all-apps\n";
    *res += "build-all-apps: build-compile-environments";
    // there can be no name clashes in frontend and backend apps because we check with materialized view
    for app in checked.db.backend_application().rows_iter() {
        *res += " build-app_";
        *res += checked.db.backend_application().c_application_name(app);
    }
    for app in checked.db.frontend_application().rows_iter() {
        *res += " build-app_";
        *res += checked.db.frontend_application().c_application_name(app);
    }
    *res += "\n";

    tag(res, "build all backend apps");
    for app in checked.db.backend_application().rows_iter() {
        let comp_env_name = checked.db.rust_compilation_environment().c_env_name(
            checked.db.backend_application().c_build_environment(app)
        );
        let app_name = checked.db.backend_application().c_application_name(app);
        // TODO: flake.nix just refer to one Cargo.lock ?
        write!(res, "apps/{app_name}/Cargo.lock: comp-envs/{comp_env_name}/Cargo.lock
	cd apps/{app_name} && cp ../../comp-envs/{comp_env_name}/Cargo.lock Cargo.lock
").unwrap();
        *res += ".PHONY: build-app_";
        *res += app_name;
        *res += "\n";
        *res += "build-app_";
        *res += app_name;
        *res += ":";
        write!(res, r#" apps/{app_name}/Cargo.lock
	cd apps/{app_name} && nix build
"#).unwrap();
    }
    tag(res, "build all frontend apps");
    for app in checked.db.frontend_application().rows_iter() {
        let comp_env_name = checked.db.rust_compilation_environment().c_env_name(
            checked.db.frontend_application().c_build_environment(app)
        );
        let app_name = checked.db.frontend_application().c_application_name(app);
        write!(res, "apps/{app_name}/Cargo.lock: comp-envs/{comp_env_name}/Cargo.lock
	cd apps/{app_name} && cp ../../comp-envs/{comp_env_name}/Cargo.lock Cargo.lock
").unwrap();
        *res += ".PHONY: build-app_";
        *res += app_name;
        *res += "\n";
        *res += "build-app_";
        *res += app_name;
        *res += ":";
        write!(res, r#" apps/{app_name}/Cargo.lock
	cd apps/{app_name} && nix build
"#).unwrap();
    }
}

fn aws_images_part(checked: &CheckedDB) -> String {
    let mut res = String::new();
    let mut aws_regions: Vec<String> = Vec::new();
    for aws_dc in checked.projections.cloud_topologies.aws.dcs.values() {
        let region = &aws_dc.region;
        aws_regions.push(format!("test -f ec2-images/{region}.x86_64-linux.ami_id"));
    }
    let aws_regions_tests = aws_regions.join(" && ");
    res += ".PHONY: aws-images\n";
    res += "aws-images:";
    for ua in &checked.projections.used_architectures {
        write!(&mut res, " aws-image-{ua}").unwrap();
    }
    res += "\n";
    res += "\n";

    for ua in &checked.projections.used_architectures {
        write!(&mut res, r#"
# if ami image already in cache upload /dev/null
# to save on AWS upload costs and upload time
.PHONY: maybe-fake-aws-image-{ua}
maybe-fake-aws-image-{ua}:
	cd terraform/aws/image-{ua} && \
	  {aws_regions_tests} && \
	  ( echo /dev/null > /tmp/aws-image.txt ) && \
	  cp -u /tmp/aws-image.txt aws-image.txt \
	  || true

.PHONY: aws-image-{ua}
aws-image-{ua}: maybe-fake-aws-image-{ua} terraform/aws/image-{ua}/aws-image.txt

terraform/aws/image-{ua}/result:
	cd terraform/aws/image-{ua} && nix build --override-input nixos-generators $(NIXOS_GENERATORS) ./flake.nix

terraform/aws/image-{ua}/aws-image.txt: terraform/aws/image-{ua}/result
	cd terraform/aws/image-{ua} && \
	  find -L $(MAKEFILE_DIRECTORY)/terraform/aws/image-{ua}/result -type f | grep -F '.vhd' \
	   > aws-image.txt
"#).unwrap()
    }

    res

}

fn google_cloud_images_part(checked: &CheckedDB) -> String {
    let mut res = String::new();
    tag(&mut res, "build google cloud all architecture images");
    res += ".PHONY: gcloud-images\n";
    res += "gcloud-images:";
    for ua in &checked.projections.used_architectures {
        res += " terraform/gcloud/image-";
        res += ua;
        res += "/gcloud-image.txt";
    }
    res += "\n";
    res += "\n";

    for ua in &checked.projections.used_architectures {
        write!(&mut res, r#"
.PHONY: gcloud-image-{ua}
gcloud-image-{ua}: terraform/gcloud/image-{ua}/gcloud-image.txt

terraform/gcloud/image-{ua}/result:
	cd terraform/gcloud/image-{ua} && nix build --override-input nixos-generators $(NIXOS_GENERATORS) ./flake.nix

terraform/gcloud/image-{ua}/gcloud-image.txt: terraform/gcloud/image-{ua}/result
	cd terraform/gcloud/image-{ua} && \
	  find -L $(MAKEFILE_DIRECTORY)/terraform/gcloud/image-{ua}/result -type f | grep -F '.tar.gz' \
	   > gcloud-image.txt
"#).unwrap();
    }

    res
}

fn makefile_integration_tests_target(checked: &CheckedDB, res: &mut String) {
    write!(res, r#"
.PHONY: integration-tests
integration-tests: build-integration-tests
	$(MAKE) -B integration-tests/grafana-instances-admin-passwords.txt
	cd integration-tests && \
	  ADMIN_PANEL_PASSWORD=$$( $(EPL_EXECUTABLE) get-secret --output-directory .. --key admin_panel_password ) \"#).unwrap();

    let mut regions: BTreeMap<TableRowPointerRegion, BTreeSet<TableRowPointerGrafana>> = BTreeMap::new();
    for g in checked.db.grafana().rows_iter() {
        let grafana_name = checked.db.grafana().c_deployment_name(g);
        let grafana_env_name = grafana_name.to_case(convert_case::Case::ScreamingSnake);
    write!(res, r#"
	  GRAFANA_{grafana_env_name}_ADMIN_PASSWORD=$$( cat grafana-instances-admin-passwords.txt | grep -E '^{grafana_name}' | awk '{{print $$2}}' ) \"#).unwrap();
        let e = regions.entry(checked.db.grafana().c_region(g)).or_default();
        e.insert(g);
    }

    write!(res, r#"
	  timeout 60s cargo test

integration-tests/grafana-instances-admin-passwords.txt:
	rm -f integration-tests/grafana-instances-admin-passwords.txt
	touch integration-tests/grafana-instances-admin-passwords.txt
	chmod 600 integration-tests/grafana-instances-admin-passwords.txt"#).unwrap();
    for (region, graf_instances) in regions {
        let region_name = checked.db.region().c_region_name(region);
        let server = first_first_region_vault_server(&checked.db, region);
        if let Some(server) = server {
            let network_iface = checked.projections.consul_network_iface.value(server);
            let ip = checked.db.network_interface().c_if_ip(*network_iface);
            let mut all_grafana_instances = String::new();
            for graf_instance in graf_instances {
                all_grafana_instances += " ";
                all_grafana_instances += checked.db.grafana().c_deployment_name(graf_instance);
            }
            write!(res, r#"
	$(LOAD_SHELL_LIB) extract_grafana_admin_keys_from_vault \
	  {region_name} $(EPL_EXECUTABLE) {ip} {all_grafana_instances} \
	  >> ../integration-tests/grafana-instances-admin-passwords.txt
"#).unwrap();
        }
    }
}
