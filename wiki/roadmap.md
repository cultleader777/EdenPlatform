# Roadmap

## High level features
- [x] Dns
Use good ol' bind. Three authoritative nameservers, we own our DNS
Replace DNS Masq with bind
- [x] Automatic https certs with dns challenge
- [ ] Self hosted email
- [x] Frontend generation
- [x] Expose S3 minio storage
- [x] Alerting
- [x] Logging
- [ ] Theoretical memory distribution html page
- [x] Docker image table with types
- [ ] Dynamic nginx config reloading
- [ ] Wiki documentation of all capabilities
- [ ] Auto query endpoints
Attach to the backend, add another columns for selecting special query
- [ ] User authentications with session
- [ ] FoundationDB

- [x] Multiple DC support + region data
- [ ] Open api/swagger garbage import from file and hooray we can use api
- [ ] Destructors for resources depending on the context
- [ ] Pre release checklist for all test environments
- [x] consul based fast l1 provisioning
- [x] Generate auto tests for environments

## Cloud implementations

- [x] aws
- [x] google cloud
- [ ] openstack
- [ ] azure
- [ ] digital ocean
- [ ] linode
- [ ] vultr

## Nice to have
- [ ] human readable errors

## Alpha checklist
- [x] move secrets out of hive.nix
- [x] generate ssh key in secrets.yml for testing
- [x] serial list generation when zone reloading
- [x] have edge nodes that have the egress load balancer for the zone
- [x] dynamic consul template reloading for nginx service
- [x] basic auth on admin panel all endpoints
- [x] network_interface.if_ip should contain ip with cidr like 10.17.0.10/24
cidr is separate but default /24 is everywhere
- [x] root on ZFS
- [x] always build our own NixOS image
- [x] disks reform for clouds
- [x] peer consul datacenters
- [x] peer nomad regions
- [x] expand VPN subnet to /16
- [x] memory_max limit enable in nomad and all reduce memory for all services
- [x] admin user instead of root
- [x] 10 ocaml data modules in EdenDB
- [x] 7 grafana tempo traces
- [x] 5 expose s3 api to apps
- [x] 9 coprocessor datacenter
- [x] 9 scalable consul based provisioning
- [x] 8 clickhouse support
- [x] 9 blackbox deployment
- [x] 9 certbot certs with rotation
- [x] 7 prometheus metric db
refresh for all prom clusters and detect which clusters have what metrics
- [ ] 6 backups of datasets
- [ ] 5 open api spec import
- [ ] 5 single nix file build all apps/artefacts
- [x] 4 ipv6 just add to load balancers and DNS
- [ ] 4 email
- [ ] 1 generate root dnscontrol config for external providers
