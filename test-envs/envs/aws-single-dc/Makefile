
#############################################
# Core Makefile part
#############################################

EPL_ENV_NAME = aws-single-dc
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
	$(RUN_AS_NON_ROOT) $(MAKE) aws-images
	$(RUN_AS_NON_ROOT) $(MAKE) build-all-apps
	$(RUN_AS_NON_ROOT) $(MAKE) build-integration-tests

.PHONY: full-provision-pre-l1
full-provision-pre-l1:
ifneq ($(shell id -u), 0)
	$(error "You are not root, full provision routine requires sudo")
else
	$(RUN_AS_NON_ROOT) $(MAKE) terraform-provision-all-clouds
	$(RUN_AS_NON_ROOT) $(MAKE) compile-project # public ips updated for servers, recompile project
	$(RUN_AS_NON_ROOT) $(MAKE) wait-ready-public-servers -j $(L1_PROVISIONING_JOBS) # wait for servers ready before running preconditions
	$(RUN_AS_NON_ROOT) $(MAKE) public-servers-preconditions -j $(L1_PROVISIONING_JOBS) # preconditions for public nodes
	$(RUN_AS_NON_ROOT) $(MAKE) bootstrap-l1-public-nodes -j $(L1_PROVISIONING_JOBS) # provision public ip servers to establish wireguard
	$(MAKE) all-dcs-start-vpn
	$(RUN_AS_NON_ROOT) $(MAKE) aws-private-ips-bootstrap-internet
	$(RUN_AS_NON_ROOT) $(MAKE) wait-ready-all-servers -j $(L1_PROVISIONING_JOBS)
	$(RUN_AS_NON_ROOT) $(MAKE) all-servers-preconditions -j $(L1_PROVISIONING_JOBS)
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



.PHONY: aws-images
aws-images: aws-image-x86_64


# if ami image already in cache upload /dev/null
# to save on AWS upload costs and upload time
.PHONY: maybe-fake-aws-image-x86_64
maybe-fake-aws-image-x86_64:
	cd terraform/aws/image-x86_64 && \
	  test -f ec2-images/us-west-1.x86_64-linux.ami_id && \
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
epl-executable: $(EPL_EXECUTABLE)


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

.PHONY: ci-l1-provision
ci-l1-provision:
	$(MAKE) compile-project
	$(MAKE) refresh-server-infra-state
	# infra-state.sqlite added if empty servers returned from sqlite
	$(MAKE) -j $(L1_PROVISIONING_JOBS) infra-state.sqlite \
	  -e L1_PROVISIONING_ID=$$( cat l1-fast/epl-prov-id ) \
	  $$( echo " \
	        SELECT 'l1-provision-ww_' || hostname \
	        FROM servers \
	        WHERE hostname \
	        NOT IN ( \
	          SELECT hostname \
	          FROM bootstrapped_servers \
	        ); \
	      " | sqlite3 infra-state.sqlite )
	$(MAKE) fast-l1-provision

infra-state.sqlite:
	$(LOAD_SHELL_LIB) init_infra_state_db

$(EPL_EXECUTABLE):
	cd $(EPL_PROJECT_DIR) && cargo build

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
	  admin@77.77.77.10 "gunzip | sudo sh"
.PHONY: l1-provision_server-d
l1-provision_server-d:
	$(LOAD_SHELL_LIB) cat $(or $(CUSTOM_SCRIPT),../l1-provisioning/server-d/provision.sh) | \
	  sed 's/L1_EPL_PROVISIONING_ID/$(L1_PROVISIONING_ID)/g' | \
	  sed 's/L1_PROVISIONING_TOLERATE_REBUILD_FAIL/$(L1_PROVISIONING_TOLERATE_REBUILD_FAIL)/g' | \
	  sed 's/L1_RESTART_CONSUL_POST_SECRETS/$(L1_RESTART_CONSUL_POST_SECRETS)/g' | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@77.77.77.11 "gunzip | sudo sh"


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
	$(LOAD_SHELL_LIB) wait_l1_provisioning_finished $(L1_PROVISIONING_ID) 10.17.0.10 server-a
.PHONY: wait-l1-provision_server-b
wait-l1-provision_server-b: infra-state.sqlite
	$(LOAD_SHELL_LIB) wait_l1_provisioning_finished $(L1_PROVISIONING_ID) 10.17.0.11 server-b
.PHONY: wait-l1-provision_server-c
wait-l1-provision_server-c: infra-state.sqlite
	$(LOAD_SHELL_LIB) wait_l1_provisioning_finished $(L1_PROVISIONING_ID) 77.77.77.10 server-c
.PHONY: wait-l1-provision_server-d
wait-l1-provision_server-d: infra-state.sqlite
	$(LOAD_SHELL_LIB) wait_l1_provisioning_finished $(L1_PROVISIONING_ID) 77.77.77.11 server-d


#############################################
# separate servers l1 provision with wait
#############################################
.PHONY: l1-provision-ww_server-a
l1-provision-ww_server-a:
	$(MAKE) l1-provision_server-a -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-a -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
.PHONY: l1-provision-ww_server-b
l1-provision-ww_server-b:
	$(MAKE) l1-provision_server-b -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-b -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
.PHONY: l1-provision-ww_server-c
l1-provision-ww_server-c:
	$(MAKE) l1-provision_server-c -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-c -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
.PHONY: l1-provision-ww_server-d
l1-provision-ww_server-d:
	$(MAKE) l1-provision_server-d -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision_server-d -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)


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
	cd l1-fast && tar -zcvf us-west.tar.gz us-west/
	$(LOAD_SHELL_LIB) fast_l1_provisioning_for_region 10.17.0.10 us-west ../l1-fast/us-west.tar.gz $$( cat ../l1-fast/epl-prov-id ) $(EPL_EXECUTABLE)


#############################################
# public servers preconditions
#############################################
.PHONY: public-servers-preconditions
public-servers-preconditions: preconditions_server-c preconditions_server-d

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
# separate servers preconditions
#############################################
.PHONY: preconditions_server-a
preconditions_server-a:
	$(LOAD_SHELL_LIB) cat ../l1-provisioning/server-a/preconditions.sh | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@10.17.0.10 "gunzip | sh"
.PHONY: preconditions_server-b
preconditions_server-b:
	$(LOAD_SHELL_LIB) cat ../l1-provisioning/server-b/preconditions.sh | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@10.17.0.11 "gunzip | sh"
.PHONY: preconditions_server-c
preconditions_server-c:
	$(LOAD_SHELL_LIB) cat ../l1-provisioning/server-c/preconditions.sh | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@77.77.77.10 "gunzip | sh"
.PHONY: preconditions_server-d
preconditions_server-d:
	$(LOAD_SHELL_LIB) cat ../l1-provisioning/server-d/preconditions.sh | \
	  gzip -9 | \
	  ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i aux/root_ssh_key \
	  admin@77.77.77.11 "gunzip | sh"


#############################################
# all regions provisioning targets
#############################################
.PHONY: consul-bootstrap_all-regions
consul-bootstrap_all-regions: consul-bootstrap_region_us-west
.PHONY: nomad-bootstrap_all-regions
nomad-bootstrap_all-regions: nomad-bootstrap_region_us-west
.PHONY: vault-init_all-regions
vault-init_all-regions: vault-init_region_us-west
.PHONY: vault-unseal_all-regions
vault-unseal_all-regions: vault-unseal_region_us-west
.PHONY: nomad-policies_all-regions
nomad-policies_all-regions: nomad-policies_region_us-west
.PHONY: vault-policies_all-regions
vault-policies_all-regions: vault-policies_region_us-west

#############################################
# l2 provisioning all regions target
#############################################
.PHONY: l2-provision
l2-provision: l2-provision_us-west

#############################################
# separate regions provisioning targets
#############################################
.PHONY: consul-bootstrap_region_us-west
consul-bootstrap_region_us-west:
	$(LOAD_SHELL_LIB) \
	ssh admin@10.17.0.10 -i aux/root_ssh_key -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 \
	  'epl-consul-bootstrap; epl-consul-vrrp-acl; epl-nomad-consul-acl-bootstrap; epl-vault-consul-acl-bootstrap'
.PHONY: nomad-bootstrap_region_us-west
nomad-bootstrap_region_us-west:
	$(LOAD_SHELL_LIB) maybe_bootstrap_nomad 10.17.0.10 us-west $(EPL_EXECUTABLE)
.PHONY: vault-init_region_us-west
vault-init_region_us-west: vault-init_region_us-west_server_server-b
.PHONY: vault-init_region_us-west_server_server-b
vault-init_region_us-west_server_server-b:
	$(LOAD_SHELL_LIB) \
	maybe_init_vault 10.17.0.11 https://server-b.us-west.epl-infra.net:8200 us-west $(EPL_EXECUTABLE)
	echo Vault init done
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
.PHONY: nomad-policies_region_us-west
nomad-policies_region_us-west:
	$(LOAD_SHELL_LIB) nomad_policies_provision 10.17.0.10 us-west $(EPL_EXECUTABLE)
.PHONY: vault-policies_region_us-west
vault-policies_region_us-west:
	$(LOAD_SHELL_LIB) vault_nomad_policies_provision 10.17.0.10 us-west $(EPL_EXECUTABLE)
	$(LOAD_SHELL_LIB) vault_acme_policies_provision 10.17.0.10 us-west $(EPL_EXECUTABLE)
.PHONY: l2-provision_us-west
l2-provision_us-west:
	$(LOAD_SHELL_LIB) EPL_EXECUTABLE=$(EPL_EXECUTABLE) ./provision us-west 10.17.0.10

#############################################
# wait ready public servers
#############################################
.PHONY: wait-ready-public-servers
wait-ready-public-servers: wait-ready_server-c wait-ready_server-d

#############################################
# wait ready all servers
#############################################
.PHONY: wait-ready-all-servers
wait-ready-all-servers: wait-ready_server-a wait-ready_server-b wait-ready_server-c wait-ready_server-d

#############################################
# wait ready separate server targets
#############################################
.PHONY: wait-ready_server-a
wait-ready_server-a:
	$(LOAD_SHELL_LIB) ensure_server_ready 10.17.0.10
.PHONY: wait-ready_server-b
wait-ready_server-b:
	$(LOAD_SHELL_LIB) ensure_server_ready 10.17.0.11
.PHONY: wait-ready_server-c
wait-ready_server-c:
	$(LOAD_SHELL_LIB) ensure_server_ready 77.77.77.10
.PHONY: wait-ready_server-d
wait-ready_server-d:
	$(LOAD_SHELL_LIB) ensure_server_ready 77.77.77.11

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
	ssh -o ServerAliveInterval=10 -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i servers/aux/root_ssh_key admin@77.77.77.10
.PHONY: login_server-d
login_server-d:
	ssh -o ServerAliveInterval=10 -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=false -o ConnectionAttempts=3 -o ConnectTimeout=10 -i servers/aux/root_ssh_key admin@77.77.77.11

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
terraform-provision-all-clouds: terraform-provision-aws
.PHONY: terraform-destroy-all-clouds
terraform-destroy-all-clouds: terraform-destroy-aws
.PHONY: terraform-provision-aws
terraform-provision-aws: aws-images
	cd terraform/aws && stat .terraform.lock.hcl || terraform init
	cd terraform/aws && terraform apply -auto-approve
	cd terraform/aws && cat terraform.tfstate | jq '{"network_interface":[ .resources[] | select(.type | contains("aws_instance")) | select(.instances[0].attributes.public_ip != "") | {"primary_key": (.name + "=>void"),"replacements":{"if_ip":.instances[0].attributes.public_ip}} ], "server": [.resources[] | select(.type | contains("aws_instance")) | select(.instances[0].attributes.ipv6_addresses[0]) | {"primary_key":.name,"replacements":{"public_ipv6_address":.instances[0].attributes.ipv6_addresses[0]}}]}' > replacements.json ; \
	$(EDENDB_EXECUTABLE) --replacements-file replacements.json ../../data/main.edl $(EPL_PROJECT_DIR)/edb-src/main.edl
.PHONY: terraform-destroy-aws
terraform-destroy-aws:
	cd terraform/aws && terraform destroy $(TF_DESTROY_FLAGS)
.PHONY: bootstrap-l1-public-nodes
bootstrap-l1-public-nodes: ensure-public-servers-ready
	$(MAKE) l1-provision-public-servers -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_TOLERATE_REBUILD_FAIL=true -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)
	$(MAKE) wait-l1-provision-public-servers -j $(L1_PROVISIONING_JOBS) -e L1_PROVISIONING_ID=$(L1_PROVISIONING_ID)

.PHONY: ensure-public-servers-ready
ensure-public-servers-ready: wait-ready_server-c wait-ready_server-d
.PHONY: l1-provision-public-servers
l1-provision-public-servers: l1-provision_server-c l1-provision_server-d
.PHONY: wait-l1-provision-public-servers
wait-l1-provision-public-servers: wait-l1-provision_server-c wait-l1-provision_server-d



#############################################
# aws public subnets internet bootstrap
#############################################
.PHONY: aws-private-ips-bootstrap-internet
aws-private-ips-bootstrap-internet:
	$(LOAD_SHELL_LIB) aws_bootstrap_private_node_internet 10.17.0.10 10.17.0.12 1.1.1.1 10.17.0.0/16 10.17.0.1
	$(LOAD_SHELL_LIB) aws_bootstrap_private_node_internet 10.17.0.11 10.17.0.12 1.1.1.1 10.17.0.0/16 10.17.0.1


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
.PHONY: refresh-server-infra-state
refresh-server-infra-state: infra-state.sqlite
	echo " \
	  DELETE FROM servers; \
	  INSERT INTO servers(hostname) \
	  VALUES \
	    ('server-a'), \
	    ('server-b'), \
	    ('server-c'), \
	    ('server-d'); \
	" | sqlite3 infra-state.sqlite


#############################################
# refresh all prometheus clusters metrics
#############################################
.PHONY: scrape-planned-metrics
scrape-planned-metrics:
	$(EPL_EXECUTABLE) scrape-prometheus-metrics prometheus/metric_scrape_plan.yml prometheus/scraped_metrics.sqlite

.PHONY: refresh-metrics-db
refresh-metrics-db:
	$(EPL_EXECUTABLE) refresh-prometheus-metrics --prometheus-url default,http://10.17.0.10:9090 > metrics_db.yml.tmp
	mv -f metrics_db.yml.tmp metrics_db.yml

