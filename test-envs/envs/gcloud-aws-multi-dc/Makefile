
#############################################
# Core Makefile part
#############################################

EPL_ENV_NAME = gcloud-aws-multi-dc
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
	$(RUN_AS_NON_ROOT) $(MAKE) gcloud-images
	$(RUN_AS_NON_ROOT) $(MAKE) aws-images
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
	$(RUN_AS_NON_ROOT) $(MAKE) terraform-provision-all-clouds
	$(RUN_AS_NON_ROOT) $(MAKE) compile-project # public ips might be updated for servers, recompile project
	$(RUN_AS_NON_ROOT) $(MAKE) wait-ready-public-servers -j $(L1_PROVISIONING_JOBS) # wait for servers ready before running preconditions
	$(RUN_AS_NON_ROOT) $(MAKE) public-servers-preconditions -j $(L1_PROVISIONING_JOBS) # preconditions for public nodes
	$(RUN_AS_NON_ROOT) $(MAKE) ci-bootstrap-l1-public-nodes -j $(L1_PROVISIONING_JOBS) # provision public ip servers to establish wireguard
	$(MAKE) all-dcs-start-vpn
	$(RUN_AS_NON_ROOT) $(MAKE) aws-private-ips-bootstrap-internet
	$(RUN_AS_NON_ROOT) $(MAKE) wait-ready-all-servers -j $(L1_PROVISIONING_JOBS)
	$(RUN_AS_NON_ROOT) $(MAKE) all-servers-preconditions -j $(L1_PROVISIONING_JOBS)
endif

# target is tweaked for being run all the time so that targets are skipped
.PHONY: ci-full-provision-pre-l1
ci-full-provision-pre-l1:
ifneq ($(shell id -u), 0)
	$(error "You are not root, full provision routine requires sudo")
else
	$(RUN_AS_NON_ROOT) $(MAKE) ci-terraform-provision-all-clouds
	$(RUN_AS_NON_ROOT) $(MAKE) compile-project # public ips might be updated for servers, recompile project
	$(RUN_AS_NON_ROOT) $(MAKE) ci-wait-ready-public-servers -j $(L1_PROVISIONING_JOBS) # wait for servers ready before running preconditions
	$(RUN_AS_NON_ROOT) $(MAKE) ci-public-servers-preconditions -j $(L1_PROVISIONING_JOBS) # preconditions for public nodes
	$(RUN_AS_NON_ROOT) $(MAKE) ci-bootstrap-l1-public-nodes -j $(L1_PROVISIONING_JOBS) # provision public ip servers to establish wireguard
	$(MAKE) all-dcs-start-vpn
	$(RUN_AS_NON_ROOT) $(MAKE) aws-private-ips-bootstrap-internet
	$(RUN_AS_NON_ROOT) $(MAKE) ci-wait-ready-all-servers -j $(L1_PROVISIONING_JOBS)
	$(RUN_AS_NON_ROOT) $(MAKE) ci-all-servers-preconditions -j $(L1_PROVISIONING_JOBS)
endif

.PHONY: full-provision-pre-l2
full-provision-pre-l2:
	# initial l1 will fail because consul doesn't work yet
	$(RUN_AS_NON_ROOT) $(MAKE) l1-provision-with-wait -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID) -e L1_RESTART_CONSUL_POST_SECRETS=true -e L1_PROVISIONING_TOLERATE_REBUILD_FAIL=true
	# if wireguard is used run l1-provision second time
	$(RUN_AS_NON_ROOT) $(MAKE) l1-provision-with-wait -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID_2)
	$(RUN_AS_NON_ROOT) $(MAKE) wait-cross-dc-ping_all-regions
	$(RUN_AS_NON_ROOT) $(MAKE) restart-consul-masters_all-regions
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
	# if wireguard is used run l1-provision second time
	$(RUN_AS_NON_ROOT) $(MAKE) ci-l1-provision -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID_2)
	$(RUN_AS_NON_ROOT) $(MAKE) wait-cross-dc-ping_all-regions
	$(RUN_AS_NON_ROOT) $(MAKE) restart-consul-masters_all-regions
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



#############################################
# build google cloud all architecture images
#############################################
.PHONY: gcloud-images
gcloud-images: terraform/gcloud/image-x86_64/gcloud-image.txt


.PHONY: gcloud-image-x86_64
gcloud-image-x86_64: terraform/gcloud/image-x86_64/gcloud-image.txt

terraform/gcloud/image-x86_64/result:
	cd terraform/gcloud/image-x86_64 && nix build --override-input nixos-generators $(NIXOS_GENERATORS) ./flake.nix

terraform/gcloud/image-x86_64/gcloud-image.txt: terraform/gcloud/image-x86_64/result
	cd terraform/gcloud/image-x86_64 && \
	  find -L $(MAKEFILE_DIRECTORY)/terraform/gcloud/image-x86_64/result -type f | grep -F '.tar.gz' \
	   > gcloud-image.txt

.PHONY: aws-images
aws-images: aws-image-x86_64


# if ami image already in cache upload /dev/null
# to save on AWS upload costs and upload time
.PHONY: maybe-fake-aws-image-x86_64
maybe-fake-aws-image-x86_64:
	cd terraform/aws/image-x86_64 && \
	  test -f ec2-images/us-west-2.x86_64-linux.ami_id && \
	  ( echo /dev/null > /tmp/aws-image.txt ) && \
	  cp -u /tmp/aws-image.txt aws-image.txt \
	  || true

.PHONY: aws-image-x86_64
aws-image-x86_64: maybe-fake-aws-image-x86_64 terraform/aws/image-x86_64/aws-image.txt

terraform/aws/image-x86_64/result:
	cd terraform/aws/image-x86_64 && nix build --override-input nixos-generators $(NIXOS_GENERATORS) ./flake.nix

terraform/aws/image-x86_64/aws-image.txt: terraform/aws/image-x86_64/result
	cd terraform/aws/image-x86_64 && \
	  find -L $(MAKEFILE_DIRECTORY)/terraform/aws/image-x86_64/result -type f | grep -F '.vhd' \
	   > aws-image.txt



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
l1-provision: l1-provision_server-a l1-provision_server-b l1-provision_server-c l1-provision_server-d l1-provision_server-e l1-provision_server-f l1-provision_server-g l1-provision_server-h l1-provision_server-i

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
	  admin@34.102.96.253 "gunzip | sudo sh"
.PHONY: l1-provision_server-b
l1-provision_server-b:
	$(LOAD_SHELL_LIB) cat $(or $(CUSTOM_SCRIPT),../l1-provisioning/server-b/provision.sh) | \
	  sed 's/L1_EPL_PROVISIONING_ID/$(L1_PROVISIONING_ID)/g' | \
	  sed 's/L1_PROVISIONING_TOLERATE_REBUILD_FAIL/$(L1_PROVISIONING_TOLERATE_REBUILD_FAIL)/g' | \
	  sed 's/L1_RESTART_CONSUL_POST_SECRETS/$(L1_RESTART_CONSUL_POST_SECRETS)/g' | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@34.94.176.250 "gunzip | sudo sh"
.PHONY: l1-provision_server-c
l1-provision_server-c:
	$(LOAD_SHELL_LIB) cat $(or $(CUSTOM_SCRIPT),../l1-provisioning/server-c/provision.sh) | \
	  sed 's/L1_EPL_PROVISIONING_ID/$(L1_PROVISIONING_ID)/g' | \
	  sed 's/L1_PROVISIONING_TOLERATE_REBUILD_FAIL/$(L1_PROVISIONING_TOLERATE_REBUILD_FAIL)/g' | \
	  sed 's/L1_RESTART_CONSUL_POST_SECRETS/$(L1_RESTART_CONSUL_POST_SECRETS)/g' | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@34.102.72.86 "gunzip | sudo sh"
.PHONY: l1-provision_server-d
l1-provision_server-d:
	$(LOAD_SHELL_LIB) cat $(or $(CUSTOM_SCRIPT),../l1-provisioning/server-d/provision.sh) | \
	  sed 's/L1_EPL_PROVISIONING_ID/$(L1_PROVISIONING_ID)/g' | \
	  sed 's/L1_PROVISIONING_TOLERATE_REBUILD_FAIL/$(L1_PROVISIONING_TOLERATE_REBUILD_FAIL)/g' | \
	  sed 's/L1_RESTART_CONSUL_POST_SECRETS/$(L1_RESTART_CONSUL_POST_SECRETS)/g' | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@35.235.83.50 "gunzip | sudo sh"
.PHONY: l1-provision_server-e
l1-provision_server-e:
	$(LOAD_SHELL_LIB) cat $(or $(CUSTOM_SCRIPT),../l1-provisioning/server-e/provision.sh) | \
	  sed 's/L1_EPL_PROVISIONING_ID/$(L1_PROVISIONING_ID)/g' | \
	  sed 's/L1_PROVISIONING_TOLERATE_REBUILD_FAIL/$(L1_PROVISIONING_TOLERATE_REBUILD_FAIL)/g' | \
	  sed 's/L1_RESTART_CONSUL_POST_SECRETS/$(L1_RESTART_CONSUL_POST_SECRETS)/g' | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@34.212.47.215 "gunzip | sudo sh"
.PHONY: l1-provision_server-f
l1-provision_server-f:
	$(LOAD_SHELL_LIB) cat $(or $(CUSTOM_SCRIPT),../l1-provisioning/server-f/provision.sh) | \
	  sed 's/L1_EPL_PROVISIONING_ID/$(L1_PROVISIONING_ID)/g' | \
	  sed 's/L1_PROVISIONING_TOLERATE_REBUILD_FAIL/$(L1_PROVISIONING_TOLERATE_REBUILD_FAIL)/g' | \
	  sed 's/L1_RESTART_CONSUL_POST_SECRETS/$(L1_RESTART_CONSUL_POST_SECRETS)/g' | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@34.214.1.46 "gunzip | sudo sh"
.PHONY: l1-provision_server-g
l1-provision_server-g:
	$(LOAD_SHELL_LIB) cat $(or $(CUSTOM_SCRIPT),../l1-provisioning/server-g/provision.sh) | \
	  sed 's/L1_EPL_PROVISIONING_ID/$(L1_PROVISIONING_ID)/g' | \
	  sed 's/L1_PROVISIONING_TOLERATE_REBUILD_FAIL/$(L1_PROVISIONING_TOLERATE_REBUILD_FAIL)/g' | \
	  sed 's/L1_RESTART_CONSUL_POST_SECRETS/$(L1_RESTART_CONSUL_POST_SECRETS)/g' | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@10.17.0.12 "gunzip | sudo sh"
.PHONY: l1-provision_server-h
l1-provision_server-h:
	$(LOAD_SHELL_LIB) cat $(or $(CUSTOM_SCRIPT),../l1-provisioning/server-h/provision.sh) | \
	  sed 's/L1_EPL_PROVISIONING_ID/$(L1_PROVISIONING_ID)/g' | \
	  sed 's/L1_PROVISIONING_TOLERATE_REBUILD_FAIL/$(L1_PROVISIONING_TOLERATE_REBUILD_FAIL)/g' | \
	  sed 's/L1_RESTART_CONSUL_POST_SECRETS/$(L1_RESTART_CONSUL_POST_SECRETS)/g' | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@10.18.0.12 "gunzip | sudo sh"
.PHONY: l1-provision_server-i
l1-provision_server-i:
	$(LOAD_SHELL_LIB) cat $(or $(CUSTOM_SCRIPT),../l1-provisioning/server-i/provision.sh) | \
	  sed 's/L1_EPL_PROVISIONING_ID/$(L1_PROVISIONING_ID)/g' | \
	  sed 's/L1_PROVISIONING_TOLERATE_REBUILD_FAIL/$(L1_PROVISIONING_TOLERATE_REBUILD_FAIL)/g' | \
	  sed 's/L1_RESTART_CONSUL_POST_SECRETS/$(L1_RESTART_CONSUL_POST_SECRETS)/g' | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@10.19.0.12 "gunzip | sudo sh"


#############################################
# wait all servers l1 provision
#############################################
.PHONY: wait-l1-provision
wait-l1-provision: wait-l1-provision_server-a wait-l1-provision_server-b wait-l1-provision_server-c wait-l1-provision_server-d wait-l1-provision_server-e wait-l1-provision_server-f wait-l1-provision_server-g wait-l1-provision_server-h wait-l1-provision_server-i

#############################################
# wait separate servers l1 provision
#############################################
.PHONY: wait-l1-provision_server-a
wait-l1-provision_server-a: infra-state.sqlite
	$(LOAD_SHELL_LIB) wait_l1_provisioning_finished $(L1_PROVISIONING_ID) 34.102.96.253 server-a us-west
.PHONY: wait-l1-provision_server-b
wait-l1-provision_server-b: infra-state.sqlite
	$(LOAD_SHELL_LIB) wait_l1_provisioning_finished $(L1_PROVISIONING_ID) 34.94.176.250 server-b us-west
.PHONY: wait-l1-provision_server-c
wait-l1-provision_server-c: infra-state.sqlite
	$(LOAD_SHELL_LIB) wait_l1_provisioning_finished $(L1_PROVISIONING_ID) 34.102.72.86 server-c us-west
.PHONY: wait-l1-provision_server-d
wait-l1-provision_server-d: infra-state.sqlite
	$(LOAD_SHELL_LIB) wait_l1_provisioning_finished $(L1_PROVISIONING_ID) 35.235.83.50 server-d us-west
.PHONY: wait-l1-provision_server-e
wait-l1-provision_server-e: infra-state.sqlite
	$(LOAD_SHELL_LIB) wait_l1_provisioning_finished $(L1_PROVISIONING_ID) 34.212.47.215 server-e us-west
.PHONY: wait-l1-provision_server-f
wait-l1-provision_server-f: infra-state.sqlite
	$(LOAD_SHELL_LIB) wait_l1_provisioning_finished $(L1_PROVISIONING_ID) 34.214.1.46 server-f us-west
.PHONY: wait-l1-provision_server-g
wait-l1-provision_server-g: infra-state.sqlite
	$(LOAD_SHELL_LIB) wait_l1_provisioning_finished $(L1_PROVISIONING_ID) 10.17.0.12 server-g us-west
.PHONY: wait-l1-provision_server-h
wait-l1-provision_server-h: infra-state.sqlite
	$(LOAD_SHELL_LIB) wait_l1_provisioning_finished $(L1_PROVISIONING_ID) 10.18.0.12 server-h us-west
.PHONY: wait-l1-provision_server-i
wait-l1-provision_server-i: infra-state.sqlite
	$(LOAD_SHELL_LIB) wait_l1_provisioning_finished $(L1_PROVISIONING_ID) 10.19.0.12 server-i us-west


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
.PHONY: l1-provision-ww_server-e
l1-provision-ww_server-e:
	$(MAKE) l1-provision_server-e -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-e -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
# marker
markers/l1-bootstrapped/server-e:
	$(MAKE) l1-provision_server-e -e L1_PROVISIONING_TOLERATE_REBUILD_FAIL=true -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-e -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	mkdir -p markers/l1-bootstrapped && touch markers/l1-bootstrapped/server-e
.PHONY: l1-provision-ww_server-f
l1-provision-ww_server-f:
	$(MAKE) l1-provision_server-f -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-f -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
# marker
markers/l1-bootstrapped/server-f:
	$(MAKE) l1-provision_server-f -e L1_PROVISIONING_TOLERATE_REBUILD_FAIL=true -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-f -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	mkdir -p markers/l1-bootstrapped && touch markers/l1-bootstrapped/server-f
.PHONY: l1-provision-ww_server-g
l1-provision-ww_server-g:
	$(MAKE) l1-provision_server-g -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-g -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
# marker
markers/l1-bootstrapped/server-g:
	$(MAKE) l1-provision_server-g -e L1_PROVISIONING_TOLERATE_REBUILD_FAIL=true -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-g -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	mkdir -p markers/l1-bootstrapped && touch markers/l1-bootstrapped/server-g
.PHONY: l1-provision-ww_server-h
l1-provision-ww_server-h:
	$(MAKE) l1-provision_server-h -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-h -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
# marker
markers/l1-bootstrapped/server-h:
	$(MAKE) l1-provision_server-h -e L1_PROVISIONING_TOLERATE_REBUILD_FAIL=true -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-h -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	mkdir -p markers/l1-bootstrapped && touch markers/l1-bootstrapped/server-h
.PHONY: l1-provision-ww_server-i
l1-provision-ww_server-i:
	$(MAKE) l1-provision_server-i -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-i -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
# marker
markers/l1-bootstrapped/server-i:
	$(MAKE) l1-provision_server-i -e L1_PROVISIONING_TOLERATE_REBUILD_FAIL=true -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-i -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	mkdir -p markers/l1-bootstrapped && touch markers/l1-bootstrapped/server-i


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
	if [ -e markers/fast-l1-needed/us-west ]; then $(LOAD_SHELL_LIB) fast_l1_provisioning_for_region 34.102.96.253 us-west ../l1-fast/us-west.tar.gz $$( cat ../l1-fast/epl-prov-id ) $(EPL_EXECUTABLE); fi


#############################################
# public servers preconditions
#############################################
.PHONY: public-servers-preconditions
public-servers-preconditions: preconditions_server-a preconditions_server-b preconditions_server-c preconditions_server-d preconditions_server-e preconditions_server-f

#############################################
# ci public servers preconditions
#############################################
.PHONY: ci-public-servers-preconditions
ci-public-servers-preconditions:
	$(MAKE) -j $(L1_PROVISIONING_JOBS)  markers/server-preconditions/server-a markers/server-preconditions/server-b markers/server-preconditions/server-c markers/server-preconditions/server-d markers/server-preconditions/server-e markers/server-preconditions/server-f

#############################################
# private servers preconditions
#############################################
.PHONY: private-servers-preconditions
private-servers-preconditions: preconditions_server-g preconditions_server-h preconditions_server-i

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
	$(MAKE) -j $(L1_PROVISIONING_JOBS) markers/server-preconditions/server-a markers/server-preconditions/server-b markers/server-preconditions/server-c markers/server-preconditions/server-d markers/server-preconditions/server-e markers/server-preconditions/server-f markers/server-preconditions/server-g markers/server-preconditions/server-h markers/server-preconditions/server-i
#############################################
# separate servers preconditions
#############################################
.PHONY: preconditions_server-a
preconditions_server-a:
	$(LOAD_SHELL_LIB) cat ../l1-provisioning/server-a/preconditions.sh | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@34.102.96.253 "gunzip | sh"
	mkdir -p markers/server-preconditions && touch markers/server-preconditions/server-a
markers/server-preconditions/server-a: l1-provisioning/server-a/preconditions.sh
	$(MAKE) preconditions_server-a
.PHONY: preconditions_server-b
preconditions_server-b:
	$(LOAD_SHELL_LIB) cat ../l1-provisioning/server-b/preconditions.sh | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@34.94.176.250 "gunzip | sh"
	mkdir -p markers/server-preconditions && touch markers/server-preconditions/server-b
markers/server-preconditions/server-b: l1-provisioning/server-b/preconditions.sh
	$(MAKE) preconditions_server-b
.PHONY: preconditions_server-c
preconditions_server-c:
	$(LOAD_SHELL_LIB) cat ../l1-provisioning/server-c/preconditions.sh | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@34.102.72.86 "gunzip | sh"
	mkdir -p markers/server-preconditions && touch markers/server-preconditions/server-c
markers/server-preconditions/server-c: l1-provisioning/server-c/preconditions.sh
	$(MAKE) preconditions_server-c
.PHONY: preconditions_server-d
preconditions_server-d:
	$(LOAD_SHELL_LIB) cat ../l1-provisioning/server-d/preconditions.sh | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@35.235.83.50 "gunzip | sh"
	mkdir -p markers/server-preconditions && touch markers/server-preconditions/server-d
markers/server-preconditions/server-d: l1-provisioning/server-d/preconditions.sh
	$(MAKE) preconditions_server-d
.PHONY: preconditions_server-e
preconditions_server-e:
	$(LOAD_SHELL_LIB) cat ../l1-provisioning/server-e/preconditions.sh | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@34.212.47.215 "gunzip | sh"
	mkdir -p markers/server-preconditions && touch markers/server-preconditions/server-e
markers/server-preconditions/server-e: l1-provisioning/server-e/preconditions.sh
	$(MAKE) preconditions_server-e
.PHONY: preconditions_server-f
preconditions_server-f:
	$(LOAD_SHELL_LIB) cat ../l1-provisioning/server-f/preconditions.sh | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@34.214.1.46 "gunzip | sh"
	mkdir -p markers/server-preconditions && touch markers/server-preconditions/server-f
markers/server-preconditions/server-f: l1-provisioning/server-f/preconditions.sh
	$(MAKE) preconditions_server-f
.PHONY: preconditions_server-g
preconditions_server-g:
	$(LOAD_SHELL_LIB) cat ../l1-provisioning/server-g/preconditions.sh | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@10.17.0.12 "gunzip | sh"
	mkdir -p markers/server-preconditions && touch markers/server-preconditions/server-g
markers/server-preconditions/server-g: l1-provisioning/server-g/preconditions.sh
	$(MAKE) preconditions_server-g
.PHONY: preconditions_server-h
preconditions_server-h:
	$(LOAD_SHELL_LIB) cat ../l1-provisioning/server-h/preconditions.sh | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@10.18.0.12 "gunzip | sh"
	mkdir -p markers/server-preconditions && touch markers/server-preconditions/server-h
markers/server-preconditions/server-h: l1-provisioning/server-h/preconditions.sh
	$(MAKE) preconditions_server-h
.PHONY: preconditions_server-i
preconditions_server-i:
	$(LOAD_SHELL_LIB) cat ../l1-provisioning/server-i/preconditions.sh | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@10.19.0.12 "gunzip | sh"
	mkdir -p markers/server-preconditions && touch markers/server-preconditions/server-i
markers/server-preconditions/server-i: l1-provisioning/server-i/preconditions.sh
	$(MAKE) preconditions_server-i


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
	$(LOAD_SHELL_LIB) maybe_bootstrap_nomad 10.17.0.11 us-west $(EPL_EXECUTABLE)

# marker
markers/nomad-bootstrap/us-west:
	$(MAKE) nomad-bootstrap_region_us-west
	mkdir -p markers/nomad-bootstrap && touch markers/nomad-bootstrap/us-west
.PHONY: vault-init_region_us-west
vault-init_region_us-west: vault-init_region_us-west_server_server-b vault-init_region_us-west_server_server-c vault-init_region_us-west_server_server-f
.PHONY: vault-init_region_us-west_server_server-b
vault-init_region_us-west_server_server-b:
	$(LOAD_SHELL_LIB) \
	maybe_init_vault 10.17.0.11 https://server-b.us-west.epl-infra.net:8200 us-west $(EPL_EXECUTABLE)
.PHONY: vault-init_region_us-west_server_server-c
vault-init_region_us-west_server_server-c:
	$(LOAD_SHELL_LIB) \
	maybe_init_vault 10.18.0.10 https://server-c.us-west.epl-infra.net:8200 us-west $(EPL_EXECUTABLE)
.PHONY: vault-init_region_us-west_server_server-f
vault-init_region_us-west_server_server-f:
	$(LOAD_SHELL_LIB) \
	maybe_init_vault 10.19.0.11 https://server-f.us-west.epl-infra.net:8200 us-west $(EPL_EXECUTABLE)
	echo Vault init done

# marker
markers/vault-init/us-west:
	$(MAKE) vault-init_region_us-west
	mkdir -p markers/vault-init && touch markers/vault-init/us-west
.PHONY: vault-unseal_region_us-west
vault-unseal_region_us-west: vault-unseal_region_us-west_server_server-b vault-unseal_region_us-west_server_server-c vault-unseal_region_us-west_server_server-f
.PHONY: vault-unseal_region_us-west_server_server-b
vault-unseal_region_us-west_server_server-b:
	$(LOAD_SHELL_LIB) \
	maybe_unseal_vault 10.17.0.11 https://server-b.us-west.epl-infra.net:8200 us-west $(EPL_EXECUTABLE)
.PHONY: vault-unseal_region_us-west_server_server-c
vault-unseal_region_us-west_server_server-c:
	$(LOAD_SHELL_LIB) \
	maybe_unseal_vault 10.18.0.10 https://server-c.us-west.epl-infra.net:8200 us-west $(EPL_EXECUTABLE)
.PHONY: vault-unseal_region_us-west_server_server-f
vault-unseal_region_us-west_server_server-f:
	$(LOAD_SHELL_LIB) \
	maybe_unseal_vault 10.19.0.11 https://server-f.us-west.epl-infra.net:8200 us-west $(EPL_EXECUTABLE)
	echo Vault unseal done
.PHONY: ci-vault-unseal_all-regions
ci-vault-unseal_all-regions:
	$(MAKE) -j $(L1_PROVISIONING_JOBS) ci-vault-unseal_region_us-west
.PHONY: ci-vault-unseal_region_us-west
ci-vault-unseal_region_us-west: ci-vault-unseal_region_us-west_server_server-b ci-vault-unseal_region_us-west_server_server-c ci-vault-unseal_region_us-west_server_server-f
.PHONY: ci-vault-unseal_region_us-west_server_server-b
ci-vault-unseal_region_us-west_server_server-b:
	$(MAKE) -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$$( cat l1-fast/epl-prov-id ) nothing \
	 $$( echo " SELECT 'l1-provision-ww_' || hostname FROM servers_for_slow_l1_provision WHERE hostname = 'server-b' " | sqlite3 infra-state.sqlite )
	$(LOAD_SHELL_LIB) maybe_unseal_vault 34.94.176.250 https://server-b.us-west.epl-infra.net:8200 us-west $(EPL_EXECUTABLE) default
.PHONY: ci-vault-unseal_region_us-west_server_server-c
ci-vault-unseal_region_us-west_server_server-c:
	$(MAKE) -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$$( cat l1-fast/epl-prov-id ) nothing \
	 $$( echo " SELECT 'l1-provision-ww_' || hostname FROM servers_for_slow_l1_provision WHERE hostname = 'server-c' " | sqlite3 infra-state.sqlite )
	$(LOAD_SHELL_LIB) maybe_unseal_vault 34.102.72.86 https://server-c.us-west.epl-infra.net:8200 us-west $(EPL_EXECUTABLE) default
.PHONY: ci-vault-unseal_region_us-west_server_server-f
ci-vault-unseal_region_us-west_server_server-f:
	$(MAKE) -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$$( cat l1-fast/epl-prov-id ) nothing \
	 $$( echo " SELECT 'l1-provision-ww_' || hostname FROM servers_for_slow_l1_provision WHERE hostname = 'server-f' " | sqlite3 infra-state.sqlite )
	$(LOAD_SHELL_LIB) maybe_unseal_vault 34.214.1.46 https://server-f.us-west.epl-infra.net:8200 us-west $(EPL_EXECUTABLE) default
	echo Vault unseal done
.PHONY: nomad-policies_region_us-west
nomad-policies_region_us-west:
	$(LOAD_SHELL_LIB) nomad_policies_provision 10.17.0.11 us-west $(EPL_EXECUTABLE)

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
# wait for cross dc ping all regions
#############################################
.PHONY: wait-cross-dc-ping_all-regions
wait-cross-dc-ping_all-regions: wait-cross-dc-ping_region_us-west

#############################################
# wait for cross dc conection inside regions
#############################################
.PHONY: wait-cross-dc-ping_region_us-west
wait-cross-dc-ping_region_us-west:
	$(LOAD_SHELL_LIB) wait_until_ping_succeeds 34.102.96.253 10.17.0.10 10.18.0.10
	$(LOAD_SHELL_LIB) wait_until_ping_succeeds 34.102.96.253 10.17.0.10 10.19.0.10
	$(LOAD_SHELL_LIB) wait_until_ping_succeeds 34.102.72.86 10.18.0.10 10.17.0.10
	$(LOAD_SHELL_LIB) wait_until_ping_succeeds 34.102.72.86 10.18.0.10 10.19.0.10
	$(LOAD_SHELL_LIB) wait_until_ping_succeeds 34.212.47.215 10.19.0.10 10.17.0.10
	$(LOAD_SHELL_LIB) wait_until_ping_succeeds 34.212.47.215 10.19.0.10 10.18.0.10


#############################################
# restart consul masters for all regions
#############################################
restart-consul-masters_all-regions: restart-consul-masters_region_us-west

#############################################
# restart consul masters by region
#############################################
.PHONY: restart-consul-masters_region_us-west
restart-consul-masters_region_us-west:
	$(LOAD_SHELL_LIB) \
	ssh admin@10.17.0.10 -i aux/root_ssh_key -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 \
	  'consul members || sudo systemctl restart consul.service && sleep 1'

	$(LOAD_SHELL_LIB) \
	ssh admin@10.18.0.10 -i aux/root_ssh_key -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 \
	  'consul members || sudo systemctl restart consul.service && sleep 1'

	$(LOAD_SHELL_LIB) \
	ssh admin@10.19.0.10 -i aux/root_ssh_key -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 \
	  'consul members || sudo systemctl restart consul.service && sleep 1'


#############################################
# wait ready public servers
#############################################
.PHONY: wait-ready-public-servers
wait-ready-public-servers: wait-ready_server-a wait-ready_server-b wait-ready_server-c wait-ready_server-d wait-ready_server-e wait-ready_server-f

#############################################
# ci wait ready public servers
#############################################
.PHONY: ci-wait-ready-public-servers
ci-wait-ready-public-servers:
	$(MAKE) -j $(L1_PROVISIONING_JOBS) markers/server-ready/server-a markers/server-ready/server-b markers/server-ready/server-c markers/server-ready/server-d markers/server-ready/server-e markers/server-ready/server-f

#############################################
# wait ready all servers
#############################################
.PHONY: wait-ready-all-servers
wait-ready-all-servers: wait-ready_server-a wait-ready_server-b wait-ready_server-c wait-ready_server-d wait-ready_server-e wait-ready_server-f wait-ready_server-g wait-ready_server-h wait-ready_server-i

#############################################
# ci wait ready all servers with completion markers
#############################################
.PHONY: ci-wait-ready-all-servers
ci-wait-ready-all-servers:
	$(MAKE) -j $(L1_PROVISIONING_JOBS) markers/server-ready/server-a markers/server-ready/server-b markers/server-ready/server-c markers/server-ready/server-d markers/server-ready/server-e markers/server-ready/server-f markers/server-ready/server-g markers/server-ready/server-h markers/server-ready/server-i

#############################################
# wait ready separate server targets
#############################################
.PHONY: wait-ready_server-a
wait-ready_server-a:
	$(LOAD_SHELL_LIB) ensure_server_ready 34.102.96.253
# marker
markers/server-ready/server-a:
	$(MAKE) wait-ready_server-a
	mkdir -p markers/server-ready && touch markers/server-ready/server-a
.PHONY: wait-ready_server-b
wait-ready_server-b:
	$(LOAD_SHELL_LIB) ensure_server_ready 34.94.176.250
# marker
markers/server-ready/server-b:
	$(MAKE) wait-ready_server-b
	mkdir -p markers/server-ready && touch markers/server-ready/server-b
.PHONY: wait-ready_server-c
wait-ready_server-c:
	$(LOAD_SHELL_LIB) ensure_server_ready 34.102.72.86
# marker
markers/server-ready/server-c:
	$(MAKE) wait-ready_server-c
	mkdir -p markers/server-ready && touch markers/server-ready/server-c
.PHONY: wait-ready_server-d
wait-ready_server-d:
	$(LOAD_SHELL_LIB) ensure_server_ready 35.235.83.50
# marker
markers/server-ready/server-d:
	$(MAKE) wait-ready_server-d
	mkdir -p markers/server-ready && touch markers/server-ready/server-d
.PHONY: wait-ready_server-e
wait-ready_server-e:
	$(LOAD_SHELL_LIB) ensure_server_ready 34.212.47.215
# marker
markers/server-ready/server-e:
	$(MAKE) wait-ready_server-e
	mkdir -p markers/server-ready && touch markers/server-ready/server-e
.PHONY: wait-ready_server-f
wait-ready_server-f:
	$(LOAD_SHELL_LIB) ensure_server_ready 34.214.1.46
# marker
markers/server-ready/server-f:
	$(MAKE) wait-ready_server-f
	mkdir -p markers/server-ready && touch markers/server-ready/server-f
.PHONY: wait-ready_server-g
wait-ready_server-g:
	$(LOAD_SHELL_LIB) ensure_server_ready 10.17.0.12
# marker
markers/server-ready/server-g:
	$(MAKE) wait-ready_server-g
	mkdir -p markers/server-ready && touch markers/server-ready/server-g
.PHONY: wait-ready_server-h
wait-ready_server-h:
	$(LOAD_SHELL_LIB) ensure_server_ready 10.18.0.12
# marker
markers/server-ready/server-h:
	$(MAKE) wait-ready_server-h
	mkdir -p markers/server-ready && touch markers/server-ready/server-h
.PHONY: wait-ready_server-i
wait-ready_server-i:
	$(LOAD_SHELL_LIB) ensure_server_ready 10.19.0.12
# marker
markers/server-ready/server-i:
	$(MAKE) wait-ready_server-i
	mkdir -p markers/server-ready && touch markers/server-ready/server-i

#############################################
# login to servers
#############################################
.PHONY: login_server-a
login_server-a:
	ssh -o ServerAliveInterval=10 -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i servers/aux/root_ssh_key admin@34.102.96.253
.PHONY: login_server-b
login_server-b:
	ssh -o ServerAliveInterval=10 -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i servers/aux/root_ssh_key admin@34.94.176.250
.PHONY: login_server-c
login_server-c:
	ssh -o ServerAliveInterval=10 -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i servers/aux/root_ssh_key admin@34.102.72.86
.PHONY: login_server-d
login_server-d:
	ssh -o ServerAliveInterval=10 -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i servers/aux/root_ssh_key admin@35.235.83.50
.PHONY: login_server-e
login_server-e:
	ssh -o ServerAliveInterval=10 -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i servers/aux/root_ssh_key admin@34.212.47.215
.PHONY: login_server-f
login_server-f:
	ssh -o ServerAliveInterval=10 -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i servers/aux/root_ssh_key admin@34.214.1.46
.PHONY: login_server-g
login_server-g:
	ssh -o ServerAliveInterval=10 -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i servers/aux/root_ssh_key admin@10.17.0.12
.PHONY: login_server-h
login_server-h:
	ssh -o ServerAliveInterval=10 -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i servers/aux/root_ssh_key admin@10.18.0.12
.PHONY: login_server-i
login_server-i:
	ssh -o ServerAliveInterval=10 -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i servers/aux/root_ssh_key admin@10.19.0.12

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
# terraform
#############################################
.PHONY: terraform-provision-all-clouds
terraform-provision-all-clouds: terraform-provision-aws terraform-provision-gcloud
.PHONY: ci-terraform-provision-all-clouds
ci-terraform-provision-all-clouds:
	$(MAKE) -j $(L1_PROVISIONING_JOBS) terraform/aws/terraform.tfstate terraform/gcloud/terraform.tfstate
.PHONY: terraform-destroy-all-clouds
terraform-destroy-all-clouds: terraform-destroy-aws terraform-destroy-gcloud
.PHONY: terraform-provision-aws
terraform-provision-aws: aws-images
	cd terraform/aws && stat .terraform.lock.hcl || terraform init
	cd terraform/aws && terraform apply -auto-approve
	cd terraform/aws && cat terraform.tfstate | jq '{"network_interface":[ .resources[] | select(.type | contains("aws_instance")) | select(.instances[0].attributes.public_ip != "") | {"primary_key": (.name + "=>void"),"replacements":{"if_ip":.instances[0].attributes.public_ip}} ], "server": [.resources[] | select(.type | contains("aws_instance")) | select(.instances[0].attributes.ipv6_addresses[0]) | {"primary_key":.name,"replacements":{"public_ipv6_address":.instances[0].attributes.ipv6_addresses[0]}}]}' > replacements.json ; \
	$(EDENDB_EXECUTABLE) --replacements-file replacements.json ../../data/main.edl $(EPL_PROJECT_DIR)/edb-src/main.edl

terraform/aws/terraform.tfstate: terraform/aws/main.tf
	$(MAKE) terraform-provision-aws
.PHONY: terraform-destroy-aws
terraform-destroy-aws:
	cd terraform/aws && terraform destroy $(TF_DESTROY_FLAGS)
.PHONY: terraform-provision-gcloud
terraform-provision-gcloud: gcloud-images
	cd terraform/gcloud && stat .terraform.lock.hcl || terraform init
	cd terraform/gcloud && terraform apply -auto-approve
	cd terraform/gcloud && cat terraform.tfstate | jq '{"network_interface":[ .resources[] | select(.type | contains("google_compute_instance")) | select(.instances[0].attributes.network_interface[0].access_config[0].nat_ip != null) | { "primary_key": (.name + "=>void"), "replacements":{"if_ip":.instances[0].attributes.network_interface[0].access_config[0].nat_ip} } ]}' > replacements.json ; \
	$(EDENDB_EXECUTABLE) --replacements-file replacements.json ../../data/main.edl $(EPL_PROJECT_DIR)/edb-src/main.edl

terraform/gcloud/terraform.tfstate: terraform/gcloud/main.tf
	$(MAKE) terraform-provision-gcloud
.PHONY: terraform-destroy-gcloud
terraform-destroy-gcloud:
	cd terraform/gcloud && terraform destroy $(TF_DESTROY_FLAGS)
.PHONY: ci-bootstrap-l1-public-nodes
ci-bootstrap-l1-public-nodes:
	$(MAKE) -j $(L1_PROVISIONING_JOBS) markers/server-ready/server-a markers/server-ready/server-b markers/server-ready/server-c markers/server-ready/server-d markers/server-ready/server-e markers/server-ready/server-f
	$(MAKE) -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID) markers/l1-bootstrapped/server-a markers/l1-bootstrapped/server-b markers/l1-bootstrapped/server-c markers/l1-bootstrapped/server-d markers/l1-bootstrapped/server-e markers/l1-bootstrapped/server-f

.PHONY: l1-provision-public-servers
l1-provision-public-servers: l1-provision_server-a l1-provision_server-b l1-provision_server-c l1-provision_server-d l1-provision_server-e l1-provision_server-f
.PHONY: wait-l1-provision-public-servers
wait-l1-provision-public-servers: wait-l1-provision_server-a wait-l1-provision_server-b wait-l1-provision_server-c wait-l1-provision_server-d wait-l1-provision_server-e wait-l1-provision_server-f



#############################################
# aws public subnets internet bootstrap
#############################################
.PHONY: aws-private-ips-bootstrap-internet
aws-private-ips-bootstrap-internet:
	$(LOAD_SHELL_LIB) aws_bootstrap_private_node_internet server-i 10.19.0.12 10.19.0.10 1.1.1.1 10.19.0.0/16 10.19.0.1


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
build-all-apps: build-compile-environments

#############################################
# build all backend apps
#############################################

#############################################
# build all frontend apps
#############################################

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
	    ('server-a','us-west','839c00e1fed7cb22b438a4ab26c12940e3656f84c7a472384453c7db57b674ac'), \
	    ('server-b','us-west','5ce8126595d5bf3e9149315ce2a12ba0439467b9b7b454ad49344efa65d5c6a6'), \
	    ('server-c','us-west','5e955c4b2ac8d6fd351722a450ac79ca4a8f75b1b1f67da6fa8e16fc90f01751'), \
	    ('server-d','us-west','d346b991c20f86195d9408cc89af9c937be3a0fc15cc7ec8dfc1e12f87026405'), \
	    ('server-e','us-west','3e6acb5944dc851b375d362801b7824ca19273bfac47382cbeb954f88ac3f2af'), \
	    ('server-f','us-west','4a08b4edd4b012c367a154e5d04cc5cde22cc8ea4c945f937af97a5f54e91617'), \
	    ('server-g','us-west','e411f60f702d929dc8f71e760c5c64106f8ff0d550dfa92aefc9c23dea5b9300'), \
	    ('server-h','us-west','69c9ded741eb9fa22a145b25a75a62c9d72c5dde4693e0cac8b64e9a66f5d1a6'), \
	    ('server-i','us-west','6202d40e88c1979298f83fe319685e5d039fd9f42d38f31a1715a640c12e5d44'); \
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

