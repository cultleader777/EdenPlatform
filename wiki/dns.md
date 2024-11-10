# DNS

Eden platform makes assumptions about DNS and how it is configured.

## BIND

Bind is most stable and used DNS software out there. It is fully featured to create any configuration we can possibly imagine.

## Architecture

Every server datacenter has master DNS server and from one to two slave servers. The rest of the servers in the datacenter simply forward to master and slave servers their queries from their local DNS instance. Bind instance on every server always forwards .consul zone to the consul port 8600 regardless if it is master, slave or forwarder.

Every server has DNS name of `<hostname>.<datacenter>.<tld>`, for instance `server-a.dc1.epl-infra.net`

Internal IPs are assumed to have dc in the second octet of 10.0.0.0/8 range.

For example: dc1 ip range could be 10.1.0.0/16 dc2 ip range could be 10.2.0.0/16
dc3 ip range could be 10.3.0.0/16

This way every datacenter DNS ptr record entries can have a zone of
dc1 - 1.10.in-addr.arpa.
dc1 - 2.10.in-addr.arpa.
dc1 - 3.10.in-addr.arpa.

## Master datacenters

In multi DC setup one datacenter is picked a master and there are two slave datacenters. Master DNS servers in those datacenters must have public ip addresses and they forward the root TLD domains.

NS records from third party domain provider should point to these three master servers in master datacenters.

In single dc mode masters and slaves are the root TLD servers.
In multi dc mode (at least three datacenters) masters from every datacenter are the root tld zone.
