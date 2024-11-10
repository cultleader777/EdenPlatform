#!/bin/sh

# if token is passed in somehow, unset it
unset VAULT_TOKEN

# wait for nomad hosts to be available
while ! ping -c 1 nomad-servers.service.consul
do
  sleep 1
done

# wait for nomad rpc port to open
while ! nc -z nomad-servers.service.consul 4646
do
  sleep 1
done

VAULT_TOKEN=$( cat /run/secdir/epl-job-tokens/epl-docker-registry ) nomad job run -detach nomad-jobs/epl_docker-registry.hcl
nomad job run -detach nomad-jobs/epl_external-lb.hcl
VAULT_TOKEN=$( cat /run/secdir/epl-job-tokens/epl-loki-main-us-east ) nomad job run -detach nomad-jobs/epl_loki-main-us-east.hcl
VAULT_TOKEN=$( cat /run/secdir/epl-job-tokens/epl-minio-main-us-east ) nomad job run -detach nomad-jobs/epl_minio-main-us-east.hcl
nomad job run -detach nomad-jobs/epl_mon-main-us-east.hcl
VAULT_TOKEN=$( cat /run/secdir/epl-job-tokens/epl-tempo-r2-tempo ) nomad job run -detach nomad-jobs/epl_tempo-r2-tempo.hcl
