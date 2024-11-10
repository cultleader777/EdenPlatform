#!/bin/sh

set -e

PROVISIONING_TIME=$( date '+%Y-%m-%dT%H:%M:%S.%NZ' )
PROVISIONING_LOG_DIR=/var/log/epl-l2-prov/$PROVISIONING_TIME
find /var/log/epl-l2-prov/* -type d -ctime +7 -exec rm -rf {} \; || true
mkdir -p $PROVISIONING_LOG_DIR

# level 10
stage=10
jobs=( )
stage_pids=( )
jobs+=( provision-vault-secrets.sh )
scripts/provision-vault-secrets.sh 2>&1 | ts '[%Y-%m-%dT%H:%M:%.SZ] 010 provision-vault-secrets.sh      |' | tee $PROVISIONING_LOG_DIR/010_provision-vault-secrets.sh.log & stage_pids+=( $! )
jobs+=( provision-consul-resources.sh )
scripts/provision-consul-resources.sh 2>&1 | ts '[%Y-%m-%dT%H:%M:%.SZ] 010 provision-consul-resources.sh   |' | tee $PROVISIONING_LOG_DIR/010_provision-consul-resources.sh.log & stage_pids+=( $! )

job_idx=0
for pid in "${stage_pids[@]}"; do
  job_idx=$((job_idx + 1))
  if ! wait "$pid";
  then
    echo Job ${jobs[job_idx]} in stage ${stage} failed, exiting
    exit 7
  fi
done

# level 20
stage=20
jobs=( )
stage_pids=( )
jobs+=( provision-nomad-namespaces.sh )
scripts/provision-nomad-namespaces.sh 2>&1 | ts '[%Y-%m-%dT%H:%M:%.SZ] 020 provision-nomad-namespaces.sh   |' | tee $PROVISIONING_LOG_DIR/020_provision-nomad-namespaces.sh.log & stage_pids+=( $! )
jobs+=( schedule-nomad-system-jobs.sh )
scripts/schedule-nomad-system-jobs.sh 2>&1 | ts '[%Y-%m-%dT%H:%M:%.SZ] 020 schedule-nomad-system-jobs.sh   |' | tee $PROVISIONING_LOG_DIR/020_schedule-nomad-system-jobs.sh.log & stage_pids+=( $! )

job_idx=0
for pid in "${stage_pids[@]}"; do
  job_idx=$((job_idx + 1))
  if ! wait "$pid";
  then
    echo Job ${jobs[job_idx]} in stage ${stage} failed, exiting
    exit 7
  fi
done

# level 30
stage=30
jobs=( )
stage_pids=( )
jobs+=( provision-pg-instances.sh )
scripts/provision-pg-instances.sh 2>&1 | ts '[%Y-%m-%dT%H:%M:%.SZ] 030 provision-pg-instances.sh       |' | tee $PROVISIONING_LOG_DIR/030_provision-pg-instances.sh.log & stage_pids+=( $! )
jobs+=( provision-ch-instances.sh )
scripts/provision-ch-instances.sh 2>&1 | ts '[%Y-%m-%dT%H:%M:%.SZ] 030 provision-ch-instances.sh       |' | tee $PROVISIONING_LOG_DIR/030_provision-ch-instances.sh.log & stage_pids+=( $! )
jobs+=( provision-nats-resources.sh )
scripts/provision-nats-resources.sh 2>&1 | ts '[%Y-%m-%dT%H:%M:%.SZ] 030 provision-nats-resources.sh     |' | tee $PROVISIONING_LOG_DIR/030_provision-nats-resources.sh.log & stage_pids+=( $! )

job_idx=0
for pid in "${stage_pids[@]}"; do
  job_idx=$((job_idx + 1))
  if ! wait "$pid";
  then
    echo Job ${jobs[job_idx]} in stage ${stage} failed, exiting
    exit 7
  fi
done

# level 40
stage=40
jobs=( )
stage_pids=( )
jobs+=( build-epl-apps.sh )
scripts/build-epl-apps.sh 2>&1 | ts '[%Y-%m-%dT%H:%M:%.SZ] 040 build-epl-apps.sh               |' | tee $PROVISIONING_LOG_DIR/040_build-epl-apps.sh.log & stage_pids+=( $! )

job_idx=0
for pid in "${stage_pids[@]}"; do
  job_idx=$((job_idx + 1))
  if ! wait "$pid";
  then
    echo Job ${jobs[job_idx]} in stage ${stage} failed, exiting
    exit 7
  fi
done

# level 50
stage=50
jobs=( )
stage_pids=( )
jobs+=( dr-push-epl-apps.sh )
scripts/dr-push-epl-apps.sh 2>&1 | ts '[%Y-%m-%dT%H:%M:%.SZ] 050 dr-push-epl-apps.sh             |' | tee $PROVISIONING_LOG_DIR/050_dr-push-epl-apps.sh.log & stage_pids+=( $! )

job_idx=0
for pid in "${stage_pids[@]}"; do
  job_idx=$((job_idx + 1))
  if ! wait "$pid";
  then
    echo Job ${jobs[job_idx]} in stage ${stage} failed, exiting
    exit 7
  fi
done

# level 60
stage=60
jobs=( )
stage_pids=( )
jobs+=( schedule-nomad-app-jobs.sh )
scripts/schedule-nomad-app-jobs.sh 2>&1 | ts '[%Y-%m-%dT%H:%M:%.SZ] 060 schedule-nomad-app-jobs.sh      |' | tee $PROVISIONING_LOG_DIR/060_schedule-nomad-app-jobs.sh.log & stage_pids+=( $! )

job_idx=0
for pid in "${stage_pids[@]}"; do
  job_idx=$((job_idx + 1))
  if ! wait "$pid";
  then
    echo Job ${jobs[job_idx]} in stage ${stage} failed, exiting
    exit 7
  fi
done

# level 70
stage=70
jobs=( )
stage_pids=( )
jobs+=( provision-grafana-dashboards.sh )
scripts/provision-grafana-dashboards.sh 2>&1 | ts '[%Y-%m-%dT%H:%M:%.SZ] 070 provision-grafana-dashboards.sh |' | tee $PROVISIONING_LOG_DIR/070_provision-grafana-dashboards.sh.log & stage_pids+=( $! )

job_idx=0
for pid in "${stage_pids[@]}"; do
  job_idx=$((job_idx + 1))
  if ! wait "$pid";
  then
    echo Job ${jobs[job_idx]} in stage ${stage} failed, exiting
    exit 7
  fi
done

# create log summary
cat $PROVISIONING_LOG_DIR/010_provision-vault-secrets.sh.log $PROVISIONING_LOG_DIR/010_provision-consul-resources.sh.log $PROVISIONING_LOG_DIR/020_provision-nomad-namespaces.sh.log $PROVISIONING_LOG_DIR/020_schedule-nomad-system-jobs.sh.log $PROVISIONING_LOG_DIR/030_provision-pg-instances.sh.log $PROVISIONING_LOG_DIR/030_provision-ch-instances.sh.log $PROVISIONING_LOG_DIR/030_provision-nats-resources.sh.log $PROVISIONING_LOG_DIR/040_build-epl-apps.sh.log $PROVISIONING_LOG_DIR/050_dr-push-epl-apps.sh.log $PROVISIONING_LOG_DIR/060_schedule-nomad-app-jobs.sh.log $PROVISIONING_LOG_DIR/070_provision-grafana-dashboards.sh.log | sort > $PROVISIONING_LOG_DIR/_combined.log
