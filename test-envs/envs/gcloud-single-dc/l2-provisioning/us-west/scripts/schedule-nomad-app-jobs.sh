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

