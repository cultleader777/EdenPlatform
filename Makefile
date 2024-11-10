
.PHONY: build
build:
	cargo build

.PHONY: test
test:
	cargo test

.PHONY: compile_all_environments
compile_all_environments:
	cd test-envs && ./compile-all-envs

# x86 families listed here https://cloud.google.com/compute/docs/machine-resource#x86
# do the same for arm once arm support is implemented
# derive this list with following command
.PHONY: refresh_gcloud_instance_types
refresh_gcloud_instance_types:
	gcloud compute machine-types list | \
	    tail -n +2 | \
		awk '{print $$1,$$3,$$4}' | \
		sort | \
		uniq \
		> src/static_analysis/cloud/gcloud-instance-types.txt
	cargo run dump --what gcloud-instance-types > edb-src/cloud_specific/gcloud.edl

.PHONY: refresh_aws_instance_types
refresh_aws_instance_types:
	aws ec2 describe-instance-types | jq > \
		src/static_analysis/cloud/aws-instance-types-orig.json
	cat src/static_analysis/cloud/aws-instance-types-orig.json | \
		jq -r '.InstanceTypes[] | { instance_type: .InstanceType, arch: .ProcessorInfo.SupportedArchitectures[-1], hypervisor: .Hypervisor, bare_metal: .BareMetal, cores: .VCpuInfo.DefaultVCpus, memory_mb: .MemoryInfo.SizeInMiB, maximum_iops: .EbsInfo.EbsOptimizedInfo.MaximumIops }' | \
		jq -s '.' | \
		jq 'sort_by(.instance_type)' \
		> src/static_analysis/cloud/aws-instance-types.json
	cargo run dump --what aws-instance-types > edb-src/cloud_specific/aws.edl

.PHONY: compress_l1_sig_checker
compress_l1_sig_checker:
	cd misc/l1-sig-checker && nix-build default.nix
	cd misc/l1-sig-checker && \
	find . -type f | grep -v _build |\
	  grep -E '/(default.nix|main.ml|dune-project|dune)$$' |\
	  tar -cvf l1-sig-checker.tar -T - && \
	  gzip -9 -f l1-sig-checker.tar
	cat misc/l1-sig-checker/l1-sig-checker.tar.gz | base64 -w0 > src/codegen/l1_provisioning/sigchecker.b64
