#!/bin/sh

function upload_plan_with_retry() {
  KV_PATH=$1
  KV_FILE=$2
  RAW_FILE=$( echo $KV_FILE | tr -d '@' )

  # if server not bootstrapped yet file will be skipped
  if [ ! -f "$RAW_FILE" ]
  then
    echo "File $RAW_FILE doesn't exist, skipping"
    return 0
  fi

  for I in $(seq 1 3)
  do
    if consul kv put $KV_PATH $KV_FILE
    then
      break
    fi
    echo Failed consul plan upload, trying again in 3 seconds...
    sleep 3
  done
}

UPLOAD_START=$( date +%s%N )
upload_plan_with_retry epl-l1-plans/server-a @plan_server-a.bin
upload_plan_with_retry epl-l1-plans/server-b @plan_server-b.bin
upload_plan_with_retry epl-l1-plans/server-c @plan_server-c.bin
upload_plan_with_retry epl-l1-plans/server-d @plan_server-d.bin
UPLOAD_END=$( date +%s%N )
UPLOAD_MS=$(( ( UPLOAD_END - UPLOAD_START ) / 1000000 ))
echo Upload took ${UPLOAD_MS}ms
