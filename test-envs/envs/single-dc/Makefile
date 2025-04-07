
#############################################
# Core Makefile part
#############################################

EPL_ENV_NAME = single-dc
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
DOCKER_CACHE_CONTAINER_NAME = epl_docker_image_cache
EPL_PROJECT_DIR ?= $(realpath ../../..)
EPL_EXECUTABLE ?= $(realpath $(EPL_PROJECT_DIR)/target/debug/epl)
# TODO: build our own edendb and use that
EDENDB_EXECUTABLE ?= edendb
DOCKER_CACHE_DIR ?= ../../docker-cache
# DOCKER_CACHE_IMAGE ?= registry:2.8.1
DOCKER_CACHE_IMAGE ?= registry@sha256:cc6393207bf9d3e032c4d9277834c1695117532c9f7e8c64e7b7adcda3a85f39

DOCKER_CACHE_CONFIG = $(DOCKER_CACHE_DIR)/config.yml

LOAD_SHELL_LIB=cd servers; source ./library.sh;

RUN_AS_NON_ROOT = $(shell [ "$$(id -u)" -eq 0 ] && echo 'sudo -E -u $$SUDO_USER' )

ifeq ($(origin EPL_SHELL),undefined)
    $(error this makefile should only be run inside eden platform nix shell)
endif

.PHONY: build-env-projects
build-env-projects:
	$(RUN_AS_NON_ROOT) $(MAKE) build-all-apps
	$(RUN_AS_NON_ROOT) $(MAKE) build-integration-tests

markers/all-regions.txt:
	mkdir -p markers
	echo 'us-west' > markers/all-regions.txt.tmp
	cmp --silent markers/all-regions.txt.tmp markers/all-regions.txt || mv -f markers/all-regions.txt.tmp markers/all-regions.txt

.PHONY: full-provision-pre-l1
full-provision-pre-l1:
ifneq ($(shell id -u), 0)
	$(error "You are not root, full provision routine requires sudo")
else
	$(MAKE) run-all-servers
	$(RUN_AS_NON_ROOT) $(MAKE) wait-ready-all-servers -j $(L1_PROVISIONING_JOBS)
	$(RUN_AS_NON_ROOT) $(MAKE) all-servers-preconditions -j $(L1_PROVISIONING_JOBS)
endif

# target is tweaked for being run all the time so that targets are skipped
.PHONY: ci-full-provision-pre-l1
ci-full-provision-pre-l1:
ifneq ($(shell id -u), 0)
	$(error "You are not root, full provision routine requires sudo")
else
	$(MAKE) run-all-servers
	$(RUN_AS_NON_ROOT) $(MAKE) ci-wait-ready-all-servers -j $(L1_PROVISIONING_JOBS)
	$(RUN_AS_NON_ROOT) $(MAKE) ci-all-servers-preconditions -j $(L1_PROVISIONING_JOBS)
endif

.PHONY: full-provision-pre-l2
full-provision-pre-l2:
	# initial l1 will fail because consul doesn't work yet
	$(RUN_AS_NON_ROOT) $(MAKE) l1-provision-with-wait -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID) -e L1_RESTART_CONSUL_POST_SECRETS=true -e L1_PROVISIONING_TOLERATE_REBUILD_FAIL=true
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
	$(RUN_AS_NON_ROOT) $(MAKE) l1-provision-with-wait -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID_4)

markers/consul-all-regions-bootstrapped: markers/all-regions.txt
	# initial l1 will fail because consul doesn't work yet
	$(RUN_AS_NON_ROOT) $(MAKE) ci-l1-provision -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID) -e L1_RESTART_CONSUL_POST_SECRETS=true -e L1_PROVISIONING_TOLERATE_REBUILD_FAIL=true
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
	$(RUN_AS_NON_ROOT) $(MAKE) ci-l1-provision -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID_4)
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
	$(RUN_AS_NON_ROOT) $(MAKE) compile-project
	$(RUN_AS_NON_ROOT) $(MAKE) run-nix-serve
	$(RUN_AS_NON_ROOT) $(MAKE) run-docker-cache
	# cache builds by building in our machine
	$(RUN_AS_NON_ROOT) $(MAKE) build-remote-vm-configs
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
compile-project: epl-executable
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


.PHONY: vm-images
vm-images: servers/vm-template-x86_64.txt


servers/vm-template-x86_64.txt: servers/vm-template-x86_64.nix
	cd servers && \
	  $(RUN_AS_NON_ROOT) $(NIXOS_GENERATE) \
	  -I nixpkgs=channel:nixos-23.11 \
	  -f qcow-zfs --system x86_64-linux \
	  -c vm-template-x86_64.nix > vm-template-x86_64.tmp && \
	  mv -f vm-template-x86_64.tmp vm-template-x86_64.txt


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
	-ps aux | grep nix-serve.psgi | grep -v grep | awk '{print $$2}' | xargs kill

.PHONY: run-nix-serve
run-nix-serve:
	$(LOAD_SHELL_LIB) maybe_run_nix_serve

.PHONY: run-docker-cache
run-docker-cache: $(DOCKER_CACHE_CONFIG)
	docker ps --format='{{.Names}}' | grep $(DOCKER_CACHE_CONTAINER_NAME) || \
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





.PHONY: epl-executable
epl-executable:
	cd $(EPL_PROJECT_DIR) && cargo build


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
	$(RUN_AS_NON_ROOT) $(MAKE) compile-project -e EPL_MAIN_COMPILATION=true
	$(RUN_AS_NON_ROOT) $(MAKE) run-nix-serve
	$(RUN_AS_NON_ROOT) $(MAKE) run-docker-cache
	# cache builds by building in our machine
	$(RUN_AS_NON_ROOT) $(MAKE) build-remote-vm-configs
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

#############################################
# all servers l1 provision
#############################################
.PHONY: l1-provision
l1-provision: l1-provision_server-a l1-provision_server-b l1-provision_server-c l1-provision_server-d

#############################################
# separate servers l1 provision
#############################################
.PHONY: l1-provision_server-a
l1-provision_server-a:
	$(LOAD_SHELL_LIB) cat $(or $(CUSTOM_SCRIPT),../l1-provisioning/server-a/provision.sh) | \
	  sed 's/L1_EPL_PROVISIONING_ID/$(L1_PROVISIONING_ID)/g' | \
	  sed 's/L1_PROVISIONING_TOLERATE_REBUILD_FAIL/$(L1_PROVISIONING_TOLERATE_REBUILD_FAIL)/g' | \
	  sed 's/L1_RESTART_CONSUL_POST_SECRETS/$(L1_RESTART_CONSUL_POST_SECRETS)/g' | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@10.17.0.10 "gunzip | sudo sh"
.PHONY: l1-provision_server-b
l1-provision_server-b:
	$(LOAD_SHELL_LIB) cat $(or $(CUSTOM_SCRIPT),../l1-provisioning/server-b/provision.sh) | \
	  sed 's/L1_EPL_PROVISIONING_ID/$(L1_PROVISIONING_ID)/g' | \
	  sed 's/L1_PROVISIONING_TOLERATE_REBUILD_FAIL/$(L1_PROVISIONING_TOLERATE_REBUILD_FAIL)/g' | \
	  sed 's/L1_RESTART_CONSUL_POST_SECRETS/$(L1_RESTART_CONSUL_POST_SECRETS)/g' | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@10.17.0.11 "gunzip | sudo sh"
.PHONY: l1-provision_server-c
l1-provision_server-c:
	$(LOAD_SHELL_LIB) cat $(or $(CUSTOM_SCRIPT),../l1-provisioning/server-c/provision.sh) | \
	  sed 's/L1_EPL_PROVISIONING_ID/$(L1_PROVISIONING_ID)/g' | \
	  sed 's/L1_PROVISIONING_TOLERATE_REBUILD_FAIL/$(L1_PROVISIONING_TOLERATE_REBUILD_FAIL)/g' | \
	  sed 's/L1_RESTART_CONSUL_POST_SECRETS/$(L1_RESTART_CONSUL_POST_SECRETS)/g' | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@77.77.77.12 "gunzip | sudo sh"
.PHONY: l1-provision_server-d
l1-provision_server-d:
	$(LOAD_SHELL_LIB) cat $(or $(CUSTOM_SCRIPT),../l1-provisioning/server-d/provision.sh) | \
	  sed 's/L1_EPL_PROVISIONING_ID/$(L1_PROVISIONING_ID)/g' | \
	  sed 's/L1_PROVISIONING_TOLERATE_REBUILD_FAIL/$(L1_PROVISIONING_TOLERATE_REBUILD_FAIL)/g' | \
	  sed 's/L1_RESTART_CONSUL_POST_SECRETS/$(L1_RESTART_CONSUL_POST_SECRETS)/g' | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@77.77.77.13 "gunzip | sudo sh"


#############################################
# wait all servers l1 provision
#############################################
.PHONY: wait-l1-provision
wait-l1-provision: wait-l1-provision_server-a wait-l1-provision_server-b wait-l1-provision_server-c wait-l1-provision_server-d

#############################################
# wait separate servers l1 provision
#############################################
.PHONY: wait-l1-provision_server-a
wait-l1-provision_server-a: infra-state.sqlite
	$(LOAD_SHELL_LIB) wait_l1_provisioning_finished $(L1_PROVISIONING_ID) 10.17.0.10 server-a us-west
.PHONY: wait-l1-provision_server-b
wait-l1-provision_server-b: infra-state.sqlite
	$(LOAD_SHELL_LIB) wait_l1_provisioning_finished $(L1_PROVISIONING_ID) 10.17.0.11 server-b us-west
.PHONY: wait-l1-provision_server-c
wait-l1-provision_server-c: infra-state.sqlite
	$(LOAD_SHELL_LIB) wait_l1_provisioning_finished $(L1_PROVISIONING_ID) 77.77.77.12 server-c us-west
.PHONY: wait-l1-provision_server-d
wait-l1-provision_server-d: infra-state.sqlite
	$(LOAD_SHELL_LIB) wait_l1_provisioning_finished $(L1_PROVISIONING_ID) 77.77.77.13 server-d us-west


#############################################
# separate servers l1 provision with wait
#############################################
.PHONY: l1-provision-ww_server-a
l1-provision-ww_server-a:
	$(MAKE) l1-provision_server-a -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-a -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
# marker
markers/l1-bootstrapped/server-a:
	$(MAKE) l1-provision_server-a -e L1_PROVISIONING_TOLERATE_REBUILD_FAIL=true -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-a -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	mkdir -p markers/l1-bootstrapped && touch markers/l1-bootstrapped/server-a
.PHONY: l1-provision-ww_server-b
l1-provision-ww_server-b:
	$(MAKE) l1-provision_server-b -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-b -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
# marker
markers/l1-bootstrapped/server-b:
	$(MAKE) l1-provision_server-b -e L1_PROVISIONING_TOLERATE_REBUILD_FAIL=true -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-b -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	mkdir -p markers/l1-bootstrapped && touch markers/l1-bootstrapped/server-b
.PHONY: l1-provision-ww_server-c
l1-provision-ww_server-c:
	$(MAKE) l1-provision_server-c -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-c -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
# marker
markers/l1-bootstrapped/server-c:
	$(MAKE) l1-provision_server-c -e L1_PROVISIONING_TOLERATE_REBUILD_FAIL=true -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-c -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	mkdir -p markers/l1-bootstrapped && touch markers/l1-bootstrapped/server-c
.PHONY: l1-provision-ww_server-d
l1-provision-ww_server-d:
	$(MAKE) l1-provision_server-d -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-d -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
# marker
markers/l1-bootstrapped/server-d:
	$(MAKE) l1-provision_server-d -e L1_PROVISIONING_TOLERATE_REBUILD_FAIL=true -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-d -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	mkdir -p markers/l1-bootstrapped && touch markers/l1-bootstrapped/server-d


#############################################
# all servers fast l1 provision
#############################################
.PHONY: fast-l1-provision
fast-l1-provision: fast-l1-provision_us-west

#############################################
# separate regions fast l1 provision
#############################################
.PHONY: fast-l1-provision_us-west
fast-l1-provision_us-west:
	if [ -e markers/fast-l1-needed/us-west ]; then cd l1-fast && tar -zcvf us-west.tar.gz us-west/consul-push.sh \
	 $$( echo " SELECT 'us-west/plan_' || hostname || '.bin' FROM servers_for_fast_l1_provision WHERE region = 'us-west' " | sqlite3 ../infra-state.sqlite ); fi
	if [ -e markers/fast-l1-needed/us-west ]; then $(LOAD_SHELL_LIB) fast_l1_provisioning_for_region 10.17.0.10 us-west ../l1-fast/us-west.tar.gz $$( cat ../l1-fast/epl-prov-id ) $(EPL_EXECUTABLE); fi


#############################################
# public servers preconditions
#############################################
.PHONY: public-servers-preconditions
public-servers-preconditions: preconditions_server-c preconditions_server-d

#############################################
# ci public servers preconditions
#############################################
.PHONY: ci-public-servers-preconditions
ci-public-servers-preconditions:
	$(MAKE) -j $(L1_PROVISIONING_JOBS)  markers/server-preconditions/server-c markers/server-preconditions/server-d

#############################################
# private servers preconditions
#############################################
.PHONY: private-servers-preconditions
private-servers-preconditions: preconditions_server-a preconditions_server-b

#############################################
# all servers preconditions
#############################################
.PHONY: all-servers-preconditions
all-servers-preconditions: public-servers-preconditions private-servers-preconditions

#############################################
# ci all servers preconditions
#############################################
.PHONY: ci-all-servers-preconditions
ci-all-servers-preconditions:
	$(MAKE) -j $(L1_PROVISIONING_JOBS) markers/server-preconditions/server-a markers/server-preconditions/server-b markers/server-preconditions/server-c markers/server-preconditions/server-d
#############################################
# separate servers preconditions
#############################################
.PHONY: preconditions_server-a
preconditions_server-a:
	$(LOAD_SHELL_LIB) cat ../l1-provisioning/server-a/preconditions.sh | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@10.17.0.10 "gunzip | sh"
	mkdir -p markers/server-preconditions && touch markers/server-preconditions/server-a
markers/server-preconditions/server-a: l1-provisioning/server-a/preconditions.sh
	$(MAKE) preconditions_server-a
.PHONY: preconditions_server-b
preconditions_server-b:
	$(LOAD_SHELL_LIB) cat ../l1-provisioning/server-b/preconditions.sh | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@10.17.0.11 "gunzip | sh"
	mkdir -p markers/server-preconditions && touch markers/server-preconditions/server-b
markers/server-preconditions/server-b: l1-provisioning/server-b/preconditions.sh
	$(MAKE) preconditions_server-b
.PHONY: preconditions_server-c
preconditions_server-c:
	$(LOAD_SHELL_LIB) cat ../l1-provisioning/server-c/preconditions.sh | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@77.77.77.12 "gunzip | sh"
	mkdir -p markers/server-preconditions && touch markers/server-preconditions/server-c
markers/server-preconditions/server-c: l1-provisioning/server-c/preconditions.sh
	$(MAKE) preconditions_server-c
.PHONY: preconditions_server-d
preconditions_server-d:
	$(LOAD_SHELL_LIB) cat ../l1-provisioning/server-d/preconditions.sh | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@77.77.77.13 "gunzip | sh"
	mkdir -p markers/server-preconditions && touch markers/server-preconditions/server-d
markers/server-preconditions/server-d: l1-provisioning/server-d/preconditions.sh
	$(MAKE) preconditions_server-d


#############################################
# all regions provisioning targets
#############################################
.PHONY: consul-bootstrap_all-regions
consul-bootstrap_all-regions:
	$(MAKE) markers/consul-bootstrap/us-west
.PHONY: nomad-bootstrap_all-regions
nomad-bootstrap_all-regions:
	$(MAKE) markers/nomad-bootstrap/us-west
.PHONY: vault-init_all-regions
vault-init_all-regions:
	$(MAKE) markers/vault-init/us-west
.PHONY: vault-unseal_all-regions
vault-unseal_all-regions: vault-unseal_region_us-west
.PHONY: nomad-policies_all-regions
nomad-policies_all-regions:
	$(MAKE) markers/nomad-policies/us-west
.PHONY: vault-policies_all-regions
vault-policies_all-regions:
	$(MAKE) markers/vault-policies/us-west

#############################################
# l2 provisioning all regions target
#############################################
.PHONY: l2-provision
l2-provision: l2-provision_us-west
	mkdir -p markers && touch markers/l2-provisioning-done

#############################################
# separate regions provisioning targets
#############################################
.PHONY: consul-bootstrap_region_us-west
consul-bootstrap_region_us-west:
	$(LOAD_SHELL_LIB) \
	ssh admin@10.17.0.10 -i aux/root_ssh_key -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 \
	  'epl-consul-bootstrap; epl-consul-vrrp-acl; epl-nomad-consul-acl-bootstrap; epl-vault-consul-acl-bootstrap'

# marker
markers/consul-bootstrap/us-west:
	$(MAKE) consul-bootstrap_region_us-west
	mkdir -p markers/consul-bootstrap && touch markers/consul-bootstrap/us-west
.PHONY: nomad-bootstrap_region_us-west
nomad-bootstrap_region_us-west:
	$(LOAD_SHELL_LIB) maybe_bootstrap_nomad 10.17.0.10 us-west $(EPL_EXECUTABLE)

# marker
markers/nomad-bootstrap/us-west:
	$(MAKE) nomad-bootstrap_region_us-west
	mkdir -p markers/nomad-bootstrap && touch markers/nomad-bootstrap/us-west
.PHONY: vault-init_region_us-west
vault-init_region_us-west: vault-init_region_us-west_server_server-b
.PHONY: vault-init_region_us-west_server_server-b
vault-init_region_us-west_server_server-b:
	$(LOAD_SHELL_LIB) \
	maybe_init_vault 10.17.0.11 https://server-b.us-west.epl-infra.net:8200 us-west $(EPL_EXECUTABLE)
	echo Vault init done

# marker
markers/vault-init/us-west:
	$(MAKE) vault-init_region_us-west
	mkdir -p markers/vault-init && touch markers/vault-init/us-west
.PHONY: vault-unseal_region_us-west
vault-unseal_region_us-west: vault-unseal_region_us-west_server_server-b vault-unseal_region_us-west_server_server-c vault-unseal_region_us-west_server_server-d
.PHONY: vault-unseal_region_us-west_server_server-b
vault-unseal_region_us-west_server_server-b:
	$(LOAD_SHELL_LIB) \
	maybe_unseal_vault 10.17.0.11 https://server-b.us-west.epl-infra.net:8200 us-west $(EPL_EXECUTABLE)
.PHONY: vault-unseal_region_us-west_server_server-c
vault-unseal_region_us-west_server_server-c:
	$(LOAD_SHELL_LIB) \
	maybe_unseal_vault 10.17.0.12 https://server-c.us-west.epl-infra.net:8200 us-west $(EPL_EXECUTABLE)
.PHONY: vault-unseal_region_us-west_server_server-d
vault-unseal_region_us-west_server_server-d:
	$(LOAD_SHELL_LIB) \
	maybe_unseal_vault 10.17.0.13 https://server-d.us-west.epl-infra.net:8200 us-west $(EPL_EXECUTABLE)
	echo Vault unseal done
.PHONY: ci-vault-unseal_all-regions
ci-vault-unseal_all-regions:
	$(MAKE) -j $(L1_PROVISIONING_JOBS) ci-vault-unseal_region_us-west
.PHONY: ci-vault-unseal_region_us-west
ci-vault-unseal_region_us-west: ci-vault-unseal_region_us-west_server_server-b ci-vault-unseal_region_us-west_server_server-c ci-vault-unseal_region_us-west_server_server-d
.PHONY: ci-vault-unseal_region_us-west_server_server-b
ci-vault-unseal_region_us-west_server_server-b:
	$(MAKE) -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$$( cat l1-fast/epl-prov-id ) nothing \
	 $$( echo " SELECT 'l1-provision-ww_' || hostname FROM servers_for_slow_l1_provision WHERE hostname = 'server-b' " | sqlite3 infra-state.sqlite )
	$(LOAD_SHELL_LIB) maybe_unseal_vault 10.17.0.11 https://server-b.us-west.epl-infra.net:8200 us-west $(EPL_EXECUTABLE) default
.PHONY: ci-vault-unseal_region_us-west_server_server-c
ci-vault-unseal_region_us-west_server_server-c:
	$(MAKE) -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$$( cat l1-fast/epl-prov-id ) nothing \
	 $$( echo " SELECT 'l1-provision-ww_' || hostname FROM servers_for_slow_l1_provision WHERE hostname = 'server-c' " | sqlite3 infra-state.sqlite )
	$(LOAD_SHELL_LIB) maybe_unseal_vault 77.77.77.12 https://server-c.us-west.epl-infra.net:8200 us-west $(EPL_EXECUTABLE) default
.PHONY: ci-vault-unseal_region_us-west_server_server-d
ci-vault-unseal_region_us-west_server_server-d:
	$(MAKE) -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$$( cat l1-fast/epl-prov-id ) nothing \
	 $$( echo " SELECT 'l1-provision-ww_' || hostname FROM servers_for_slow_l1_provision WHERE hostname = 'server-d' " | sqlite3 infra-state.sqlite )
	$(LOAD_SHELL_LIB) maybe_unseal_vault 77.77.77.13 https://server-d.us-west.epl-infra.net:8200 us-west $(EPL_EXECUTABLE) default
	echo Vault unseal done
.PHONY: nomad-policies_region_us-west
nomad-policies_region_us-west:
	$(LOAD_SHELL_LIB) nomad_policies_provision 10.17.0.10 us-west $(EPL_EXECUTABLE)

# marker
markers/nomad-policies/us-west:
	$(MAKE) nomad-policies_region_us-west
	mkdir -p markers/nomad-policies && touch markers/nomad-policies/us-west
.PHONY: vault-policies_region_us-west
vault-policies_region_us-west:
	$(LOAD_SHELL_LIB) vault_nomad_policies_provision 10.17.0.11 us-west $(EPL_EXECUTABLE)
	$(LOAD_SHELL_LIB) vault_acme_policies_provision 10.17.0.11 us-west $(EPL_EXECUTABLE)

# marker
markers/vault-policies/us-west:
	$(MAKE) vault-policies_region_us-west
	mkdir -p markers/vault-policies && touch markers/vault-policies/us-west
.PHONY: l2-provision_us-west
l2-provision_us-west:
	$(LOAD_SHELL_LIB) EPL_EXECUTABLE=$(EPL_EXECUTABLE) ./provision us-west 10.17.0.10

#############################################
# up all networks
#############################################
.PHONY: up-vm-networks
up-vm-networks: ensure-bridge-nf-tables-set up-net_virbr7 up-net_virbr8 up-libvirt-nat-disable-rule

#############################################
# down all networks
#############################################
.PHONY: down-vm-networks
down-vm-networks: down-libvirt-nat-disable-rule  down-net_virbr7 down-net_virbr8

#############################################
# up every network
#############################################
.PHONY: up-net_virbr7
up-net_virbr7:
ifneq ($(shell id -u), 0)
	$(error "You are not root, to create vm networks root is required")
else
	virsh net-list --all | grep virbr7 || \
	  virsh net-define servers/networks/virbr7.xml
	virsh net-list | grep virbr7 || \
	  virsh net-start virbr7
endif
.PHONY: up-net_virbr8
up-net_virbr8:
ifneq ($(shell id -u), 0)
	$(error "You are not root, to create vm networks root is required")
else
	virsh net-list --all | grep virbr8 || \
	  virsh net-define servers/networks/virbr8.xml
	virsh net-list | grep virbr8 || \
	  virsh net-start virbr8
endif

#############################################
# down every network
#############################################
.PHONY: down-net_virbr7
down-net_virbr7:
ifneq ($(shell id -u), 0)
	$(error "You are not root, to remove vm networks root is required")
else
	virsh net-list | grep virbr7 && \
	  virsh net-destroy virbr7 || true
	virsh net-list --all | grep virbr7 && \
	  virsh net-undefine virbr7 || true
endif
.PHONY: down-net_virbr8
down-net_virbr8:
ifneq ($(shell id -u), 0)
	$(error "You are not root, to remove vm networks root is required")
else
	virsh net-list | grep virbr8 && \
	  virsh net-destroy virbr8 || true
	virsh net-list --all | grep virbr8 && \
	  virsh net-undefine virbr8 || true
endif

#############################################
# wait ready public servers
#############################################
.PHONY: wait-ready-public-servers
wait-ready-public-servers: wait-ready_server-c wait-ready_server-d

#############################################
# ci wait ready public servers
#############################################
.PHONY: ci-wait-ready-public-servers
ci-wait-ready-public-servers:
	$(MAKE) -j $(L1_PROVISIONING_JOBS) markers/server-ready/server-c markers/server-ready/server-d

#############################################
# wait ready all servers
#############################################
.PHONY: wait-ready-all-servers
wait-ready-all-servers: wait-ready_server-a wait-ready_server-b wait-ready_server-c wait-ready_server-d

#############################################
# ci wait ready all servers with completion markers
#############################################
.PHONY: ci-wait-ready-all-servers
ci-wait-ready-all-servers:
	$(MAKE) -j $(L1_PROVISIONING_JOBS) markers/server-ready/server-a markers/server-ready/server-b markers/server-ready/server-c markers/server-ready/server-d

#############################################
# run all servers
#############################################
.PHONY: run-all-servers
run-all-servers: up-vm-networks run-server_server-a run-server_server-b run-server_server-c run-server_server-d

#############################################
# stop all servers
#############################################
.PHONY: stop-all-servers
stop-all-servers: stop-server_server-a stop-server_server-b stop-server_server-c stop-server_server-d

#############################################
# create all vm disks
#############################################
.PHONY: create-vm-disks
create-vm-disks: servers/disks/server-a_vda.qcow servers/disks/server-a_vdb.qcow servers/disks/server-a_vdc.qcow servers/disks/server-a_vdd.qcow servers/disks/server-a_vde.qcow servers/disks/server-a_vdf.qcow servers/disks/server-a_vdg.qcow servers/disks/server-a_vdh.qcow servers/disks/server-a_vdi.qcow servers/disks/server-a_vdj.qcow servers/disks/server-b_vda.qcow servers/disks/server-b_vdb.qcow servers/disks/server-c_vda.qcow servers/disks/server-c_vdb.qcow servers/disks/server-d_vda.qcow servers/disks/server-d_vdb.qcow

#############################################
# create every server disk targets
#############################################
servers/disks/server-a_vda.qcow: servers/vm-template-x86_64.txt
	$(LOAD_SHELL_LIB) prepare_disk_img disks/server-a_vda.qcow 21474836480 vm-template-x86_64.txt
servers/disks/server-a_vdb.qcow:
	$(LOAD_SHELL_LIB) prepare_disk_img disks/server-a_vdb.qcow 21474836480
servers/disks/server-a_vdc.qcow:
	$(LOAD_SHELL_LIB) prepare_disk_img disks/server-a_vdc.qcow 21474836480
servers/disks/server-a_vdd.qcow:
	$(LOAD_SHELL_LIB) prepare_disk_img disks/server-a_vdd.qcow 21474836480
servers/disks/server-a_vde.qcow:
	$(LOAD_SHELL_LIB) prepare_disk_img disks/server-a_vde.qcow 21474836480
servers/disks/server-a_vdf.qcow:
	$(LOAD_SHELL_LIB) prepare_disk_img disks/server-a_vdf.qcow 21474836480
servers/disks/server-a_vdg.qcow:
	$(LOAD_SHELL_LIB) prepare_disk_img disks/server-a_vdg.qcow 21474836480
servers/disks/server-a_vdh.qcow:
	$(LOAD_SHELL_LIB) prepare_disk_img disks/server-a_vdh.qcow 21474836480
servers/disks/server-a_vdi.qcow:
	$(LOAD_SHELL_LIB) prepare_disk_img disks/server-a_vdi.qcow 21474836480
servers/disks/server-a_vdj.qcow:
	$(LOAD_SHELL_LIB) prepare_disk_img disks/server-a_vdj.qcow 21474836480
servers/disks/server-b_vda.qcow: servers/vm-template-x86_64.txt
	$(LOAD_SHELL_LIB) prepare_disk_img disks/server-b_vda.qcow 21474836480 vm-template-x86_64.txt
servers/disks/server-b_vdb.qcow:
	$(LOAD_SHELL_LIB) prepare_disk_img disks/server-b_vdb.qcow 21474836480
servers/disks/server-c_vda.qcow: servers/vm-template-x86_64.txt
	$(LOAD_SHELL_LIB) prepare_disk_img disks/server-c_vda.qcow 21474836480 vm-template-x86_64.txt
servers/disks/server-c_vdb.qcow:
	$(LOAD_SHELL_LIB) prepare_disk_img disks/server-c_vdb.qcow 21474836480
servers/disks/server-d_vda.qcow: servers/vm-template-x86_64.txt
	$(LOAD_SHELL_LIB) prepare_disk_img disks/server-d_vda.qcow 21474836480 vm-template-x86_64.txt
servers/disks/server-d_vdb.qcow:
	$(LOAD_SHELL_LIB) prepare_disk_img disks/server-d_vdb.qcow 21474836480

#############################################
# run every server targets
#############################################
.PHONY: run-server_server-a
run-server_server-a: up-vm-networks | servers/disks/server-a_vda.qcow servers/disks/server-a_vdb.qcow servers/disks/server-a_vdc.qcow servers/disks/server-a_vdd.qcow servers/disks/server-a_vde.qcow servers/disks/server-a_vdf.qcow servers/disks/server-a_vdg.qcow servers/disks/server-a_vdh.qcow servers/disks/server-a_vdi.qcow servers/disks/server-a_vdj.qcow
ifneq ($(shell id -u), 0)
	$(error "You are not root, to create vms root is required")
else
	$(LOAD_SHELL_LIB) start_server server-a 8192 8 x86_64 \
	  --disk path=disks/server-a_vda.qcow,device=disk,serial=vda \
	  --disk path=disks/server-a_vdb.qcow,device=disk,serial=vdb \
	  --disk path=disks/server-a_vdc.qcow,device=disk,serial=vdc \
	  --disk path=disks/server-a_vdd.qcow,device=disk,serial=vdd \
	  --disk path=disks/server-a_vde.qcow,device=disk,serial=vde \
	  --disk path=disks/server-a_vdf.qcow,device=disk,serial=vdf \
	  --disk path=disks/server-a_vdg.qcow,device=disk,serial=vdg \
	  --disk path=disks/server-a_vdh.qcow,device=disk,serial=vdh \
	  --disk path=disks/server-a_vdi.qcow,device=disk,serial=vdi \
	  --disk path=disks/server-a_vdj.qcow,device=disk,serial=vdj \
	  --network network=virbr7,mac=3a:db:f2:1f:a0:e9 \
	&& true
endif
.PHONY: run-server_server-b
run-server_server-b: up-vm-networks | servers/disks/server-b_vda.qcow servers/disks/server-b_vdb.qcow
ifneq ($(shell id -u), 0)
	$(error "You are not root, to create vms root is required")
else
	$(LOAD_SHELL_LIB) start_server server-b 8192 2 x86_64 \
	  --disk path=disks/server-b_vda.qcow,device=disk,serial=vda \
	  --disk path=disks/server-b_vdb.qcow,device=disk,serial=vdb \
	  --network network=virbr7,mac=ce:1e:d6:b7:33:de \
	&& true
endif
.PHONY: run-server_server-c
run-server_server-c: up-vm-networks | servers/disks/server-c_vda.qcow servers/disks/server-c_vdb.qcow
ifneq ($(shell id -u), 0)
	$(error "You are not root, to create vms root is required")
else
	$(LOAD_SHELL_LIB) start_server server-c 8192 2 x86_64 \
	  --disk path=disks/server-c_vda.qcow,device=disk,serial=vda \
	  --disk path=disks/server-c_vdb.qcow,device=disk,serial=vdb \
	  --network network=virbr7,mac=3e:9a:22:a5:c7:16 \
	  --network network=virbr8,mac=fe:71:26:9c:52:b6 \
	&& true
endif
.PHONY: run-server_server-d
run-server_server-d: up-vm-networks | servers/disks/server-d_vda.qcow servers/disks/server-d_vdb.qcow
ifneq ($(shell id -u), 0)
	$(error "You are not root, to create vms root is required")
else
	$(LOAD_SHELL_LIB) start_server server-d 8192 2 x86_64 \
	  --disk path=disks/server-d_vda.qcow,device=disk,serial=vda \
	  --disk path=disks/server-d_vdb.qcow,device=disk,serial=vdb \
	  --network network=virbr7,mac=9a:3b:c0:77:9f:fc \
	  --network network=virbr8,mac=4a:08:02:8b:11:11 \
	&& true
endif

#############################################
# stop every separate server targets
#############################################
.PHONY: stop-server_server-a
stop-server_server-a:
ifneq ($(shell id -u), 0)
	$(error "You are not root, to stop vms root is required")
else
	$(LOAD_SHELL_LIB) stop_server server-a
endif
.PHONY: stop-server_server-b
stop-server_server-b:
ifneq ($(shell id -u), 0)
	$(error "You are not root, to stop vms root is required")
else
	$(LOAD_SHELL_LIB) stop_server server-b
endif
.PHONY: stop-server_server-c
stop-server_server-c:
ifneq ($(shell id -u), 0)
	$(error "You are not root, to stop vms root is required")
else
	$(LOAD_SHELL_LIB) stop_server server-c
endif
.PHONY: stop-server_server-d
stop-server_server-d:
ifneq ($(shell id -u), 0)
	$(error "You are not root, to stop vms root is required")
else
	$(LOAD_SHELL_LIB) stop_server server-d
endif

#############################################
# teardown every separate server targets
#############################################
.PHONY: teardown-server_server-a
teardown-server_server-a: stop-server_server-a
	rm -f servers/disks/server-a_vda.qcow
	rm -f servers/disks/server-a_vdb.qcow
	rm -f servers/disks/server-a_vdc.qcow
	rm -f servers/disks/server-a_vdd.qcow
	rm -f servers/disks/server-a_vde.qcow
	rm -f servers/disks/server-a_vdf.qcow
	rm -f servers/disks/server-a_vdg.qcow
	rm -f servers/disks/server-a_vdh.qcow
	rm -f servers/disks/server-a_vdi.qcow
	rm -f servers/disks/server-a_vdj.qcow
.PHONY: teardown-server_server-b
teardown-server_server-b: stop-server_server-b
	rm -f servers/disks/server-b_vda.qcow
	rm -f servers/disks/server-b_vdb.qcow
.PHONY: teardown-server_server-c
teardown-server_server-c: stop-server_server-c
	rm -f servers/disks/server-c_vda.qcow
	rm -f servers/disks/server-c_vdb.qcow
.PHONY: teardown-server_server-d
teardown-server_server-d: stop-server_server-d
	rm -f servers/disks/server-d_vda.qcow
	rm -f servers/disks/server-d_vdb.qcow

#############################################
# wait ready separate server targets
#############################################
.PHONY: wait-ready_server-a
wait-ready_server-a:
	$(LOAD_SHELL_LIB) ensure_server_ready 10.17.0.10
# marker
markers/server-ready/server-a:
	$(MAKE) wait-ready_server-a
	mkdir -p markers/server-ready && touch markers/server-ready/server-a
.PHONY: wait-ready_server-b
wait-ready_server-b:
	$(LOAD_SHELL_LIB) ensure_server_ready 10.17.0.11
# marker
markers/server-ready/server-b:
	$(MAKE) wait-ready_server-b
	mkdir -p markers/server-ready && touch markers/server-ready/server-b
.PHONY: wait-ready_server-c
wait-ready_server-c:
	$(LOAD_SHELL_LIB) ensure_server_ready 77.77.77.12
# marker
markers/server-ready/server-c:
	$(MAKE) wait-ready_server-c
	mkdir -p markers/server-ready && touch markers/server-ready/server-c
.PHONY: wait-ready_server-d
wait-ready_server-d:
	$(LOAD_SHELL_LIB) ensure_server_ready 77.77.77.13
# marker
markers/server-ready/server-d:
	$(MAKE) wait-ready_server-d
	mkdir -p markers/server-ready && touch markers/server-ready/server-d

#############################################
# login to servers
#############################################
.PHONY: login_server-a
login_server-a:
	ssh -o ServerAliveInterval=10 -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i servers/aux/root_ssh_key admin@10.17.0.10
.PHONY: login_server-b
login_server-b:
	ssh -o ServerAliveInterval=10 -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i servers/aux/root_ssh_key admin@10.17.0.11
.PHONY: login_server-c
login_server-c:
	ssh -o ServerAliveInterval=10 -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i servers/aux/root_ssh_key admin@77.77.77.12
.PHONY: login_server-d
login_server-d:
	ssh -o ServerAliveInterval=10 -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i servers/aux/root_ssh_key admin@77.77.77.13

#############################################
# vpn
#############################################
.PHONY: all-dcs-start-vpn
all-dcs-start-vpn:

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
.PHONY: all-dcs-stop-vpn
all-dcs-stop-vpn:

ifneq ($(shell id -u), 0)
	$(error "You are not root, to create vm networks root is required")
else
	-ip route del 10.0.0.0/8 dev wg7
	-ip link del dev wg7

endif


#############################################
# compile environments
#############################################
.PHONY: build-compile-environments
build-compile-environments: comp-envs/default_backend/Cargo.lock comp-envs/default_frontend/Cargo.lock


comp-envs/default_backend/Cargo.lock: comp-envs/default_backend/Cargo.toml
	cd comp-envs/default_backend && cargo generate-lockfile

comp-envs/default_frontend/Cargo.lock: comp-envs/default_frontend/Cargo.toml
	cd comp-envs/default_frontend && cargo generate-lockfile

#############################################
# build all apps target
#############################################
.PHONY: build-all-apps
build-all-apps: build-compile-environments build-app_hello-world build-app_frontend-test build-app_frontend-other

#############################################
# build all backend apps
#############################################
apps/hello-world/Cargo.lock: comp-envs/default_backend/Cargo.lock
	cd apps/hello-world && cp ../../comp-envs/default_backend/Cargo.lock Cargo.lock
.PHONY: build-app_hello-world
build-app_hello-world: apps/hello-world/Cargo.lock
	cd apps/hello-world && nix build

#############################################
# build all frontend apps
#############################################
apps/frontend-test/Cargo.lock: comp-envs/default_frontend/Cargo.lock
	cd apps/frontend-test && cp ../../comp-envs/default_frontend/Cargo.lock Cargo.lock
.PHONY: build-app_frontend-test
build-app_frontend-test: apps/frontend-test/Cargo.lock
	cd apps/frontend-test && nix build
apps/frontend-other/Cargo.lock: comp-envs/default_frontend/Cargo.lock
	cd apps/frontend-other && cp ../../comp-envs/default_frontend/Cargo.lock Cargo.lock
.PHONY: build-app_frontend-other
build-app_frontend-other: apps/frontend-other/Cargo.lock
	cd apps/frontend-other && nix build

.PHONY: integration-tests
integration-tests: build-integration-tests
	$(MAKE) -B integration-tests/grafana-instances-admin-passwords.txt
	cd integration-tests && \
	  ADMIN_PANEL_PASSWORD=$$( $(EPL_EXECUTABLE) get-secret --output-directory .. --key admin_panel_password ) \
	  GRAFANA_MAIN_ADMIN_PASSWORD=$$( cat grafana-instances-admin-passwords.txt | grep -E '^main' | awk '{print $$2}' ) \
	  timeout 60s cargo test

integration-tests/grafana-instances-admin-passwords.txt:
	rm -f integration-tests/grafana-instances-admin-passwords.txt
	touch integration-tests/grafana-instances-admin-passwords.txt
	chmod 600 integration-tests/grafana-instances-admin-passwords.txt
	$(LOAD_SHELL_LIB) extract_grafana_admin_keys_from_vault \
	  us-west $(EPL_EXECUTABLE) 10.17.0.11  main \
	  >> ../integration-tests/grafana-instances-admin-passwords.txt

#############################################
# all servers refresh in infra db
#############################################
.PHONY: refresh-static-server-infra-state
refresh-static-server-infra-state: infra-state.sqlite
	echo " \
	  DELETE FROM servers; \
	  INSERT INTO servers(hostname, region, expected_l1_hash) \
	  VALUES \
	    ('server-a','us-west','858f23096dd1a85cfa749d2a9c34ae9ac02728476c12b6850a94e11953ad88de'), \
	    ('server-b','us-west','de124af8af91a682f3575718e0dcb58507a06254cfe5a956ceb2c9ca95654db2'), \
	    ('server-c','us-west','348001e0627ff25e4a07dbf5b19756771cb81a0a30dcdf04b1e1bd9365cdd17c'), \
	    ('server-d','us-west','8492b04b746f784006eaf90878b416d3df582957244859b54c94678373bf44dd'); \
	" | sqlite3 infra-state.sqlite


#############################################
# refresh all prometheus clusters metrics
#############################################
.PHONY: scrape-planned-metrics
scrape-planned-metrics:
	# only scrape metrics if l2 provisioning finished
	test markers/l2-provisioning-done && $(EPL_EXECUTABLE) scrape-prometheus-metrics prometheus/metric_scrape_plan.yml prometheus/scraped_metrics.sqlite || true

.PHONY: refresh-metrics-db
refresh-metrics-db:
	$(EPL_EXECUTABLE) refresh-prometheus-metrics --prometheus-url default,http://10.17.0.10:9090 > metrics_db.yml.tmp
	mv -f metrics_db.yml.tmp metrics_db.yml

