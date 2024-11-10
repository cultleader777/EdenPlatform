#!/bin/sh

export NATS_URL=nats://epl-nats-main-nats.service.consul:4222

while ! nats account info
do
  echo Waiting for the NATS cluster to be up at $NATS_URL
  sleep 2
done
while ! nats stream ls
do
  echo Waiting for the NATS cluster list streams $NATS_URL
  sleep 2
done
STREAM_CFG=$( cat << EOF | base64 -w 0
{
  "config": {
    "subjects": ["some_test_stream.*"],
    "retention": "limits",
    "max_consumers": -1,
    "max_msgs_per_subject": -1,
    "max_msgs": -1,
    "max_bytes": -1,
    "max_age": 604800000000000,
    "max_msg_size": 1048576,
    "storage": "file",
    "discard": "old",
    "num_replicas": 3,
    "duplicate_window": 120000000000,
    "sealed": false,
    "deny_delete": false,
    "deny_purge": false,
    "allow_rollup_hdrs": true,
    "allow_direct": true,
    "mirror_direct": false
  }
}
EOF
)
while echo $STREAM_CFG | base64 -d | nats stream add 'some_test_stream' --subjects 'some_test_stream.*' --config=/dev/stdin 2>&1 | tee /dev/stderr | grep 'no suitable peers for placement'
do
  echo retriable stream adding error, retrying in 3 seconds
  sleep 3
done

ADD_SUCCEEDED="${PIPESTATUS[2]}"
if [ "$ADD_SUCCEEDED" -eq 0 ];
then
  while echo $STREAM_CFG | base64 -d | nats stream edit some_test_stream --config=/dev/stdin --force 2>&1 | tee /dev/stderr | grep 'no suitable peers for placement'
  do
    echo retriable stream editing error, retrying in 3 seconds
    sleep 3
  done
fi
STREAM_CFG=$( cat << EOF | base64 -w 0
{
  "config": {
    "subjects": ["some_output_stream.*"],
    "retention": "limits",
    "max_consumers": -1,
    "max_msgs_per_subject": -1,
    "max_msgs": -1,
    "max_bytes": -1,
    "max_age": 604800000000000,
    "max_msg_size": 1048576,
    "storage": "file",
    "discard": "old",
    "num_replicas": 3,
    "duplicate_window": 120000000000,
    "sealed": false,
    "deny_delete": false,
    "deny_purge": false,
    "allow_rollup_hdrs": true,
    "allow_direct": true,
    "mirror_direct": false
  }
}
EOF
)
while echo $STREAM_CFG | base64 -d | nats stream add 'some_output_stream' --subjects 'some_output_stream.*' --config=/dev/stdin 2>&1 | tee /dev/stderr | grep 'no suitable peers for placement'
do
  echo retriable stream adding error, retrying in 3 seconds
  sleep 3
done

ADD_SUCCEEDED="${PIPESTATUS[2]}"
if [ "$ADD_SUCCEEDED" -eq 0 ];
then
  while echo $STREAM_CFG | base64 -d | nats stream edit some_output_stream --config=/dev/stdin --force 2>&1 | tee /dev/stderr | grep 'no suitable peers for placement'
  do
    echo retriable stream editing error, retrying in 3 seconds
    sleep 3
  done
fi
STREAM_CFG=$( cat << EOF | base64 -w 0
{
  "config": {
    "subjects": ["chdb_a_sink"],
    "retention": "limits",
    "max_consumers": -1,
    "max_msgs_per_subject": -1,
    "max_msgs": -1,
    "max_bytes": -1,
    "max_age": 604800000000000,
    "max_msg_size": 1048576,
    "storage": "file",
    "discard": "old",
    "num_replicas": 3,
    "duplicate_window": 120000000000,
    "sealed": false,
    "deny_delete": false,
    "deny_purge": false,
    "allow_rollup_hdrs": true,
    "allow_direct": true,
    "mirror_direct": false
  }
}
EOF
)
while echo $STREAM_CFG | base64 -d | nats stream add 'chdb_a_sink' --subjects 'chdb_a_sink' --config=/dev/stdin 2>&1 | tee /dev/stderr | grep 'no suitable peers for placement'
do
  echo retriable stream adding error, retrying in 3 seconds
  sleep 3
done

ADD_SUCCEEDED="${PIPESTATUS[2]}"
if [ "$ADD_SUCCEEDED" -eq 0 ];
then
  while echo $STREAM_CFG | base64 -d | nats stream edit chdb_a_sink --config=/dev/stdin --force 2>&1 | tee /dev/stderr | grep 'no suitable peers for placement'
  do
    echo retriable stream editing error, retrying in 3 seconds
    sleep 3
  done
fi
nats consumer add chdb_a_sink ch_imp_stream_import --filter 'chdb_a_sink' --target ch_imp.testch.chdb_a.stream_import --deliver-group ch_imp_stream_import --ack none --deliver all --replay instant --defaults || true
