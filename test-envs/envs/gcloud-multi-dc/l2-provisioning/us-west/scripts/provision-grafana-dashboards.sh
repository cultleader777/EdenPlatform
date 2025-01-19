
set -e
# pass root vault token in to access all secrets
[ -n "$VAULT_TOKEN" ] || { echo VAULT_TOKEN environment variable is required; exit 7; }
# wait for instance to be healthy
while ! curl -f http://epl-grafana-main.service.consul:3000/api/health
do
  echo Waiting for grafana healthcheck to be up...
  sleep 5
done
ADMIN_PASSWORD=$( vault kv get -field=admin_password epl/grafana/main )

jq ".dashboard.id = null | .overwrite = true" grafana-dashboard/node-exporter-full.json > grafana-dashboard/node-exporter-full.json.fixed
while ! curl -f -u admin:$ADMIN_PASSWORD -XPOST -H 'Content-Type: application/json' --data @grafana-dashboard/node-exporter-full.json.fixed http://epl-grafana-main.service.consul:3000/api/dashboards/db
do
  echo Can\'t upload grafana dashboard grafana-dashboard/node-exporter-full.json from first time, trying again after second...
  sleep 1
done

jq ".dashboard.id = null | .overwrite = true" grafana-dashboard/loki.json > grafana-dashboard/loki.json.fixed
while ! curl -f -u admin:$ADMIN_PASSWORD -XPOST -H 'Content-Type: application/json' --data @grafana-dashboard/loki.json.fixed http://epl-grafana-main.service.consul:3000/api/dashboards/db
do
  echo Can\'t upload grafana dashboard grafana-dashboard/loki.json from first time, trying again after second...
  sleep 1
done

jq ".dashboard.id = null | .overwrite = true" grafana-dashboard/zfs-disk-space.json > grafana-dashboard/zfs-disk-space.json.fixed
while ! curl -f -u admin:$ADMIN_PASSWORD -XPOST -H 'Content-Type: application/json' --data @grafana-dashboard/zfs-disk-space.json.fixed http://epl-grafana-main.service.consul:3000/api/dashboards/db
do
  echo Can\'t upload grafana dashboard grafana-dashboard/zfs-disk-space.json from first time, trying again after second...
  sleep 1
done

jq ".dashboard.id = null | .overwrite = true" grafana-dashboard/zfs-performance.json > grafana-dashboard/zfs-performance.json.fixed
while ! curl -f -u admin:$ADMIN_PASSWORD -XPOST -H 'Content-Type: application/json' --data @grafana-dashboard/zfs-performance.json.fixed http://epl-grafana-main.service.consul:3000/api/dashboards/db
do
  echo Can\'t upload grafana dashboard grafana-dashboard/zfs-performance.json from first time, trying again after second...
  sleep 1
done

jq ".dashboard.id = null | .overwrite = true" grafana-dashboard/cadvisor.json > grafana-dashboard/cadvisor.json.fixed
while ! curl -f -u admin:$ADMIN_PASSWORD -XPOST -H 'Content-Type: application/json' --data @grafana-dashboard/cadvisor.json.fixed http://epl-grafana-main.service.consul:3000/api/dashboards/db
do
  echo Can\'t upload grafana dashboard grafana-dashboard/cadvisor.json from first time, trying again after second...
  sleep 1
done
