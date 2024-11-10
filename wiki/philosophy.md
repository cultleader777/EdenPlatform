# Philosophy

The ideas behind Eden platform are very simple:
- All in one distributed, highly available web development platform
- Catch as many mistakes as possible from runtime and production to the compile time
- One way to do things
- Generate as much code as possible
- Everything is opinionated already so you don't need to think

## The one mind

A lot of organizations become buried in hundreds of different technology choices. This is not a sign of competence or technical prowess. This is a sign of confusion and everyone being fragmented hence many different teams reinventing the same wheels. All of this complexity can be eliminated if everything is made by one mind having a unified purpose (me).

In those days there was no king in Israel, but every man did that which was right in his own eyes. - Judges 17:6
And he said, I saw all Israel scattered upon the hills, as sheep that have not a shepherd: and the LORD said, These have no master: let them return every man to his house in peace. - 1 Kings 22:17
For God is not the author of confusion, but of peace, as in all churches of the saints. - 1 Corinthians 14:33

I, as the creator and a king of Eden platform have already decided all the tools that you'll need to use to achieve everything (if you want the unbeatable productivity and solidity of Eden platform that is).

Programming language? Rust (OCaml to be added in the future).
Database? Postgres.
Queues? NATS.
Archival analytical DB? Clickhouse.
S3 compatible storage? MinIO.
Frontend single page application framework? Yew + WebAssembly.
Backend api framework? Actix web.
External load balancer? Nginx/OpenResty.
TCP load balancer? HaProxy.
Scheduler? Hashicorp Nomad.
Service registration? Hashicorp Consul.
Secrets storage? Hashicorp Vault.
End server operating system? NixOS.
DNS? Bind.
Preferred filesystem? ZFS.
Build environments? Nix.
Metrics? Prometheus.
Logging? Grafana Loki.
Tracing? Grafana Tempo.
Metric visualizations? Grafana.
And many others.

Idea is that in the long run Eden platform should allow you to impement any functionality to the end user inside the Eden platform. The idea is to make all assumptions about everything and make our infrastructure as simple as possible.

Say, Postgres is main database. If you wanted to use MySQL all of a sudden you'll have a bad time in Eden platform. MySQL will never be supported and expectation is that every user will use integrated queries with Postgres. Same with queues engine - NATS is the blessed queue engine in Eden platform and Kafka will never be supported because that would be needless duplication of work and confusion.

Long story short, I'm not interested about people who decided one weekend to explore their sexuality by taking it into their butt. Or people who might want to use faggity, worthless database like MongoDB. We're not interested in weekend garbage projects that purchased thousands of fake github stars and put all their effort into website design. We're interested in shipping rock solid performant software on rock solid stable foundation which stood a test of time and already have all the battle scars to prove it.

That being said, the dance of the pattern doesn't have to be danced exactly one way. There are different nations under the sun with different cultures of doing things. That being said, this is one specific implementation of the pattern created under one mind. So, if you do have your own preferences or tools to be used infrastructure, say, you're heavy on MySQL, even though it is clearly inferior database to Postgres, you are free to fork Eden platform and replace Postgres with MySQL. And other components as well. The most important thing is that the entire system is developed as a whole single coherent unit and that everything in your implementation of the pattern works together. EdenDB and Eden platform have a permissive BSD license so you can do anything you want with this code. But officially, the current tech stack is largely settled of stable and rock solid components to provide the best possible experience.

## Independence

Eden platform does not, and will never rely on certain cloud provider services. For instance, Eden platform will never support AWS RDS modules. Our Postgres is self hosted and highly available with Patroni. Same with S3, which in Eden platform is replaced by self hosted MinIO. The idea is, that every cloud provider provides machines and on these machines anything you can possibly imagine can be hosted. These can be bare metals, VMs, possibly lightweight containers - they all can be provisioned and managed by Eden platform. Eden platform will manage DNS, cross cloud, cross datacenter VPN connections to connect all the machines to work together. All of components inside eden platform are self hosted, this way we don't rely on any specific cloud provider, we just rely on x86_64 servers (arm64 planned also) to handle our workloads which are fully managed by eden platform. Just like in running a restaurant, the more ingridients you make yourself, freshly picked herbs, eggs, fishes - the cheaper your operating costs are. The idea is that end users can buy cheap, beefy, colocated bare metals and have their own rock solid private cloud fully managed by Eden platform. We can assume everything in Eden platform of how software runs because we own everything from top to bottom. We own foundations of rock solid reproducible operating system (NixOS). We own our scheduler, build processes, container registry storage, databases, queues, S3, load balancers, DNS and we use no third party providers. Meaning, we can adapt to any demand ever made without being bogged down by third party services which we don't have control over and which are nightmare to deal with.

## Detect as many mistakes as possible by analyzing data

We avoid integration tests. If we know that there must be 3 or 5 consul servers to form a quorum, we don't spin up machines to test that. We simply have data of all consul server instances and in milliseconds we show compile time error that you have, say, 2 consul server instances configured. We don't allow user to perform meaningless and incorrect configurations, if your eden platform data successfully compiled you know for a fact that you have 3 or 5 instances of consul to form a quorum. Or, for instance, we detect in milliseconds by analyzing memory requirements for components that your postgres instance needs 16GB of RAM but no such server exists to accomodate this workload. We don't spin up machines to test this and don't need to dig through logs of something not working, feedback loop is instant. We do not allow users to the best of our ability to not be able to represent invalid states in their infrastructure.

## Responsibility

Unlike today's yaml hell, we don't shake off responsibility to the end user that can make all sorts of mistakes configuring their yamls and they can only find out after deploying their infrastructure. If something doesn't work in Eden platform after deployment it is assumed to be responsibility of Eden platform compiler to have allowed invalid infrastructure state. If we lack certain information to catch certain errors before deployment, we don't throw hands into the air like most ponytail yaml fags do blaming the user. If we can't prove certain property of infrastructure yet - we add it as data into eden platform so then we can analyze it and tell the user about the issues before deployment.

## Zero overhead

We avoid dynamism in our system and make things as static as possible. For instance, if someone uses cancer of kubernetes they have problems of ip churn and then lots of messages to be exchanged for routing information and extra overhead. Our servers have internal IPs specified beforehand. They are checked to never have clashes. And our docker containers run in host network mode having the host ip. All of our services have the ports they will use decided beforehand and they will never clash in production because that is detected in milliseconds with static analysis of port clashes. You have 65 thousand ports, so you can have 65 thousand applications if they use one port in your infrastructure. Hence, we don't have unnecessary overhead in our networking stack beside minimal configuration of static VLAN ips which will never change per server and hence are blazing fast and rock solid. Also, since we use Rust we'll need a lot less hardware to begin with. Eden platform is intended for 1000 bare metal servers with 1000 user services and if you will ever outgrow that, at that point you'll likely need to make custom modifications for Eden platform. But our scope is assumed and finite and we need a lot less hardware to begin with because of using Rust and using lightweight components (avoiding memory hungry JVMs from Apache Software Foundation)

## Immutability

Our infrastructure should change as little as possible once it is in working state. All ips are static. For DNS we use good ol' BIND which serves huge portion of the internet and its records only need to change with new machines added (which shouldn't be often). There are many more modern DNS alternatives, which support dynamism, database backends like Postgres, but in our context, we assume that our infrastructure is largely static. Of course, we can run any dynamic services in Nomad and they will be rescheduled if one instance is down, but hosts are always static. The only reason to add more hosts is to grow, which is basically every once in a while ordering more bare metal servers or just bursting up in cloud with terraform. So hence, only every once in a while we add new servers so using DNS backend that supports dynamism is of no value. BIND is rock solid, stood a test of time, could provide any configuration possible and there's no reason to use anything else in the context of the entire Eden Platform. Consul might churn DNS entries up and down but BIND just forwards requests to consul and that's it. Overall, components in Eden Platform are immutable. Now, that is not to say state cannot be changed, you can easily spin up more postgres instances, more MinIO instances, Prometheus instances, add more servers and all DNS records will be added automatically and everything will work. What I mean is that once your desired state is applied in under 5 minutes for thousand servers it remains static and doesn't need to change. Hence, less moving parts and more good sleep with less firefighting due to dynamic problems.

## Eliminate all the boilerplate or die trying

We don't deal with writing our own HTTP endpoints in Rust. We simply declare in eden data language that we want such and such endpoint, with such and such path and that endpoint is parsed and analyzed in eden data compiler, many trivial mistakes detected early (like you have duplicate argument name in the path of your endpoint) and that endpoint boilerplate is generated in your rust code - you only as end user receive typesafe parsed struct in Rust with all the arguments that you specified in your endpoint, any request that doesn't match endpoint schema returns Bad Request HTTP error. To interact with these endpoints from frontend you can also only do that in typesafe way by generated endpoint calls and can never make a mistake. That reduces code you need to write by over 90%, literally making you a 10x developer. For instance, this is line count for hello-world test app at the point of writing this wiki:

```
wc -l *
 1310 generated.rs
   95 implementation.rs
    7 main.rs
 1412 total
```

User implementation is 95 lines of code and generated code, that has Postgres queries, interaction with NATS queues, typesafe generated http endpoints, prometheus metrics and special backwards compatible types is 1310 lines of code. Needless to say, a lot less error prone and no integration tests are needed when everything is generated and you know that things will work together from the first time.

## Security by default

Developers don't want to care about security. They want to get things done and move with the day. It is perfectly reasonable, that's why security is fully managed by Eden platform. If you say in Eden platform that this backend app uses this database you don't need to specify the password. If it is not generated yet it is automatically generated and saved into Hashicorp Vault, only accessible by policies that actually need the password. Your app gets password automatically and you as an implementer of backend app just get already managed and authenticated connection to database running inside Eden platform.

Eden platform strives to use most available security features out of the box. To name a few:
1. Hashicorp Vault TLS encryption
2. Hashicorp Vault Gossip encryption
3. Hashicorp Vault ACLs
4. Consul TLS encryption
5. Consul Gossip encryption
6. Consul ACLs with tokens
7. Nomad TLS encryption
8. Nomad Gossip encryption
9. Nomad ACLs
10. MinIO bucket policies
11. PostgreSQL separate users and passwords
12. DNSSEC

And many more. Every single node managed by Eden platform has all sensitive configs and secrets in memory and every server has swap disabled, if server is rebooted it needs to be reinstated by provisioning to rejoin the cluster.

User doesn't need to deal with security but it is there by default.
