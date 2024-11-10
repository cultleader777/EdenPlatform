# Networking

Eden platform makes assumptions about how networking is setup across the datacenters.

There are two networks in Eden platform:

## Internal LAN

Eden platform reserves usage of 10.0.0.0/8 range for internal networking.

Second octet identifies datacenter. For example:
dc1 ip range could be 10.1.0.0/16
dc2 ip range could be 10.2.0.0/16
dc3 ip range could be 10.3.0.0/16

Every server in internal LAN can reach any other server even if it is inside another DC or region. Of course, every datacenter should strive to be independent to avoid latency and use local datacenter services.

## Public internet

Every server may or may not have public IP. At least two (for high availability) servers in a datacenter should have a public ip to ensure the rest of the servers in internal LAN are reachable.

## Cross DC connectivity

Every datacenter is connected via wireguard and public IPs to communicate via two links with full mesh networking. At least one node in wireguard pairing must have public ip.

Say, if we have three datacenters:

dc1, dc2, dc3

dc1 has two connections to two wireguard servers in dc2
dc1 has two connections to two wireguard servers in dc3
dc2 has two connections to two wireguard servers in dc3

If one connection between datacenters is down failover happens to remaining connection.

## Cloud specific

### AWS

AWS networking mimicks real hardware, still 10.0.0.0/24 subnets per datacenter. One availability zone in AWS is one datacenter. If more than one AWS dc is used it is connected with AWS transit gateway. To all other datacenters connection is made via Wireguard

### Google cloud

Since google cloud doesn't support L2 traffic with their homo andromeda gay network workaround is made made to enable L2 routing to gateways to reach other datacenters. IP GRE protocol is used to send packets from inner google cloud nodes which want to connect to other clouds in eden platform, every node has IP GRE with routes to the edge nodes.

Say, node with lan ip of 10.17.0.10 has another ip gre ip address with added 128 to the third ip octet, in this case ip gre ip is 10.17.128.10, this effectively cuts in half hosts usable in google cloud datacenter to 32k which is a garbage cloud anyway.

## Bare metal DC architectures

### bm_simple

Simple DC network, assumes only one /24 subnet with a single switch.
Router gateway ip should be specified.
Modeled after simple home networks which have a home router and a switch.
