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

if ! grep '@@EPL_' nomad-jobs/apps_app-frontend-other.hcl
then
  nomad job run -detach nomad-jobs/apps_app-frontend-other.hcl
else
  echo Nomad job nomad-jobs/apps_app-frontend-other.hcl image build failed
fi
if ! grep '@@EPL_' nomad-jobs/apps_app-frontend-test.hcl
then
  nomad job run -detach nomad-jobs/apps_app-frontend-test.hcl
else
  echo Nomad job nomad-jobs/apps_app-frontend-test.hcl image build failed
fi
if ! grep '@@EPL_' nomad-jobs/apps_app-test-hello-world.hcl
then
  VAULT_TOKEN=$( cat /run/secdir/epl-job-tokens/epl-app-test-hello-world ) nomad job run -detach nomad-jobs/apps_app-test-hello-world.hcl
else
  echo Nomad job nomad-jobs/apps_app-test-hello-world.hcl image build failed
fi
nomad job run -detach nomad-jobs/apps_bb-moonbeam-dev.hcl
