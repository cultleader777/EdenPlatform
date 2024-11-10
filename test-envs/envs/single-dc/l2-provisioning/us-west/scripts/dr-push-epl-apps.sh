set -e
IMAGE_PREFIX=epl-docker-registry.service.consul:5000/epl-app
docker load -i $EPL_PROVISIONING_DIR/apps/hello-world/result
IMAGE_TAG=$( docker load -i $EPL_PROVISIONING_DIR/apps/hello-world/result | sed -E 's/^.*: //g' )
RETAGGED_IMAGE=$IMAGE_PREFIX/$IMAGE_TAG
docker tag $IMAGE_TAG $RETAGGED_IMAGE
docker push $RETAGGED_IMAGE
RETAGGED_DIGEST_TAG=$( docker image inspect $RETAGGED_IMAGE | jq -r '.[0].RepoDigests[0]' )
echo "RETAGGED_DIGEST_TAG -> [$RETAGGED_DIGEST_TAG]"
sed -i "s#@@EPL_APP_IMAGE_x86_64:hello-world@@#$RETAGGED_DIGEST_TAG#g" $EPL_PROVISIONING_DIR/nomad-jobs/apps_app-test-hello-world.hcl
docker load -i $EPL_PROVISIONING_DIR/apps/frontend-test/result
IMAGE_TAG=$( docker load -i $EPL_PROVISIONING_DIR/apps/frontend-test/result | sed -E 's/^.*: //g' )
RETAGGED_IMAGE=$IMAGE_PREFIX/$IMAGE_TAG
echo "RETAGGED_IMAGE -> [$RETAGGED_IMAGE]"
echo "IMAGE_TAG -> [$IMAGE_TAG]"
docker tag $IMAGE_TAG $RETAGGED_IMAGE
docker push $RETAGGED_IMAGE
RETAGGED_DIGEST_TAG=$( docker image inspect $RETAGGED_IMAGE | jq -r '.[0].RepoDigests[0]' )
echo "RETAGGED_DIGEST_TAG -> [$RETAGGED_DIGEST_TAG]"
sed -i "s#@@EPL_APP_IMAGE_x86_64:frontend-test@@#$RETAGGED_DIGEST_TAG#g" $EPL_PROVISIONING_DIR/nomad-jobs/apps_app-frontend-test.hcl
docker load -i $EPL_PROVISIONING_DIR/apps/frontend-other/result
IMAGE_TAG=$( docker load -i $EPL_PROVISIONING_DIR/apps/frontend-other/result | sed -E 's/^.*: //g' )
RETAGGED_IMAGE=$IMAGE_PREFIX/$IMAGE_TAG
echo "RETAGGED_IMAGE -> [$RETAGGED_IMAGE]"
echo "IMAGE_TAG -> [$IMAGE_TAG]"
docker tag $IMAGE_TAG $RETAGGED_IMAGE
docker push $RETAGGED_IMAGE
RETAGGED_DIGEST_TAG=$( docker image inspect $RETAGGED_IMAGE | jq -r '.[0].RepoDigests[0]' )
echo "RETAGGED_DIGEST_TAG -> [$RETAGGED_DIGEST_TAG]"
sed -i "s#@@EPL_APP_IMAGE_x86_64:frontend-other@@#$RETAGGED_DIGEST_TAG#g" $EPL_PROVISIONING_DIR/nomad-jobs/apps_app-frontend-other.hcl
