set -e
cd $EPL_PROVISIONING_DIR/apps/hello-world && nix build &
cd $EPL_PROVISIONING_DIR/apps/frontend-test && nix build &
wait
cd $EPL_PROVISIONING_DIR/apps/frontend-other && nix build &

wait

