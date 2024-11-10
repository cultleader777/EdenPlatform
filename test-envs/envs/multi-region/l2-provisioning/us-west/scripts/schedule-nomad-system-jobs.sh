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
VAULT_TOKEN=$( cat /run/secdir/epl-job-tokens/epl-grafana-main-us-west ) nomad job run -detach nomad-jobs/epl_grafana-main-us-west.hcl
VAULT_TOKEN=$( cat /run/secdir/epl-job-tokens/epl-loki-main-us-west ) nomad job run -detach nomad-jobs/epl_loki-main-us-west.hcl
VAULT_TOKEN=$( cat /run/secdir/epl-job-tokens/epl-minio-main-us-west ) nomad job run -detach nomad-jobs/epl_minio-main-us-west.hcl
nomad job run -detach nomad-jobs/epl_mon-main-us-west.hcl
VAULT_TOKEN=$( cat /run/secdir/epl-job-tokens/epl-pg-testdb-us-west ) nomad job run -detach nomad-jobs/epl_pg-testdb-us-west.hcl
VAULT_TOKEN=$( cat /run/secdir/epl-job-tokens/epl-tempo-r1-tempo ) nomad job run -detach nomad-jobs/epl_tempo-r1-tempo.hcl
