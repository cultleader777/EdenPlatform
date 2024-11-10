use std::fmt::Write;
use crate::{static_analysis::{CheckedDB, get_global_settings}, codegen::{nixplan::{NixAllServerPlans, mk_nix_region}, CodegenSecrets}, database::TableRowPointerServer};

use super::{routing::create_nftables_table_service, fast_l1::provision_fast_l1_provisioning};

pub fn provision_generic_every_server(
    db: &CheckedDB,
    plans: &mut NixAllServerPlans,
    cgen_secrets: &CodegenSecrets
) {
    for server in db.db.server().rows_iter() {
        provision_kernel_sysctl_settings(plans, server);
        provision_directories(plans, server);
        provision_server_packages(plans, server);
        provision_server_firewall(db, plans, server);
        provision_server_shell(db, plans, server);
        provision_fast_l1_provisioning(db, plans, server, cgen_secrets);
    }
}

fn provision_kernel_sysctl_settings(plans: &mut NixAllServerPlans, server: TableRowPointerServer) {
    let plan = plans.fetch_plan(server);
    plan.add_custom_nix_block(r#"
    boot.kernel.sysctl = {
      # for loki ScyllaDB
      "fs.aio-max-nr" = 1048576;
    };
"#.to_string());
}

/// Special directory for admin usre to put l2 secrets to
/// ./provision
/// ./epl-job-tokens
/// ./nomad-bootstrap-result
/// ./vault-bootstrap-result
fn provision_directories(plans: &mut NixAllServerPlans, server: TableRowPointerServer) {
    let plan = plans.fetch_plan(server);
    plan.add_pre_l1_provisioning_shell_hook(r#"
mkdir -m 0700 -p /run/secdir
chmod 0700 /run/secdir
chown admin /run/secdir

mkdir -m 0750 -p /var/log/epl-l2-prov
chown admin /var/log/epl-l2-prov
grep -q epl-prov /etc/group && chgrp epl-prov /var/log/epl-l2-prov || true
chmod 0750 /var/log/epl-l2-prov

mkdir -m 0750 -p /var/log/epl-l1-upload
find /var/log/epl-l1-upload/*.log -type f -ctime +7 -exec rm -rf {} \; || true
chown admin /var/log/epl-l1-upload
grep -q epl-prov /etc/group && chgrp epl-prov /var/log/epl-l1-upload || true
chmod 0750 /var/log/epl-l1-upload
"#.to_string());
}

fn provision_server_shell(db: &CheckedDB, plans: &mut NixAllServerPlans, server: TableRowPointerServer) {
    let global_settings = get_global_settings(&db.db);
    let project_name = &global_settings.project_name;
    let hostname = db.db.server().c_hostname(server);
    let dc = db.db.server().c_dc(server);
    let dc_name = db.db.datacenter().c_dc_name(dc);
    let region = db.db.datacenter().c_region(dc);
    let region_name = db.db.region().c_region_name(region);
    let plan = plans.fetch_plan(server);
    let shell_prompt = format!("{hostname}.{dc_name}.{region_name}.{project_name}");
    // copy paste edit from nixpkgs with change of \h to \H for fqdn
    plan.add_env_variable("HISTCONTROL".to_string(), "ignoreboth".to_string());
    plan.add_custom_nix_block(format!(r#"
   programs.bash.promptInit = ''
     # Provide a nice prompt if the terminal supports it.
     if [ "$TERM" != "dumb" ] || [ -n "$INSIDE_EMACS" ]; then
       PROMPT_COLOR="1;31m"
       ((UID)) && PROMPT_COLOR="1;32m"
       if [ -n "$INSIDE_EMACS" ]; then
         # Emacs term mode doesn't support xterm title escape sequence (\e]0;)
         PS1="\n\[\033[$PROMPT_COLOR\][\u@{shell_prompt}:\w]\\$\[\033[0m\] "
       else
         PS1="\n\[\033[$PROMPT_COLOR\][\[\e]0;\u@{shell_prompt}: \w\a\]\u@{shell_prompt}:\w]\\$\[\033[0m\] "
       fi
       if test "$TERM" = "xterm"; then
         PS1="\[\033]2;{shell_prompt}:\u:\w\007\]$PS1"
       fi
     fi
   '';
"#));
}

fn provision_server_firewall(db: &CheckedDB, plans: &mut NixAllServerPlans, server: TableRowPointerServer) {
    let plan = plans.fetch_plan(server);
    let hostname = db.db.server().c_hostname(server);
    let is_router = db.db.server().c_is_vpn_gateway(server) || db.db.server().c_is_router(server);
    let check_reverse_path = !is_router;
    let dc = db.db.server().c_dc(server);
    let net_ans = db.sync_res.network.networking_answers.dcs.get(&dc).unwrap();

    let mut firewall_rules_for_server = format!(r#"
  networking.hostName = "{hostname}";
  networking.firewall.allowPing = true;
  networking.firewall.enable = true;
  networking.firewall.checkReversePath = {check_reverse_path};
  networking.firewall.trustedInterfaces = [
"#);
    let mut internet_iface = "";
    let mut has_wireguard = false;
    let mut internet_if_tables = String::new();
    for ni in db.db.server().c_children_network_interface(server) {
        let network = db.db.network_interface().c_if_network(*ni);
        let network_name = db.db.network().c_network_name(network);
        let iface_name = db.db.network_interface().c_if_name(*ni);
        let iface_name = if iface_name.contains(":") {
            iface_name.split(":").next().unwrap()
        } else { iface_name.as_str() };
        let if_ip = db.db.network_interface().c_if_ip(*ni);
        let mut trust_iface = || {
            write!(&mut firewall_rules_for_server, r#"
    "{iface_name}"
"#).unwrap();
        };
        match network_name.as_str() {
            "lan" => {
                trust_iface();
            },
            "dcrouter" => {
                trust_iface();
            }
            "vpn" => {
                trust_iface();
                has_wireguard = true;
            }
            "internet" => {
                internet_iface = iface_name;

                write!(&mut internet_if_tables, r#"
       chain EPL_INTERNET_FIREWALL {{
           type filter hook prerouting priority mangle + 20; policy accept;
           iifname {iface_name} ip saddr != {{ 10.0.0.0/8, 172.21.0.0/16 }} ip daddr != {{ {if_ip}/32 }} drop comment "Disallow traffic from internet to internal networks";
       }}
"#).unwrap();
            }
            other => {
                panic!("Unknown network name {other}")
            }
        }
    }

    if net_ans.params.use_l3_hop_for_vpn_gateways {
        write!(&mut firewall_rules_for_server, r#"
    "vpnGre"
"#).unwrap();
    }

    write!(&mut firewall_rules_for_server, "
  ];
").unwrap();

    let mut allowed_internet_tcp_ports = vec![22];
    let mut allowed_internet_udp_ports = vec![];
    if db.db.server().c_is_ingress(server) {
        allowed_internet_tcp_ports.push(80);
        allowed_internet_tcp_ports.push(443);
    }
    if db.db.server().c_is_dns_master(server) ||
        db.db.server().c_is_dns_slave(server)
    {
        allowed_internet_tcp_ports.push(53);
        allowed_internet_udp_ports.push(53);
    }
    if has_wireguard {
        allowed_internet_udp_ports.push(51820);
    }

    if !internet_iface.is_empty() {
        if !net_ans.params.are_public_ips_hidden {
            write!(&mut firewall_rules_for_server, r#"
  networking.firewall.interfaces."{}".allowedTCPPorts = [ {} ];
"#,
                   internet_iface,
                   allowed_internet_tcp_ports
                   .iter()
                   .map(|i| i.to_string())
                   .collect::<Vec<_>>()
                   .join(" ")
            ).unwrap();

            if !allowed_internet_udp_ports.is_empty() {
                write!(&mut firewall_rules_for_server, r#"
  networking.firewall.interfaces."{}".allowedUDPPorts = [ {} ];
"#,
                       internet_iface,
                       allowed_internet_udp_ports
                       .iter()
                       .map(|i| i.to_string())
                       .collect::<Vec<_>>()
                       .join(" ")
                ).unwrap();
            }
        } else {
            write!(&mut firewall_rules_for_server, r#"
  networking.firewall.allowedTCPPorts = [ {} ];
"#,
                   allowed_internet_tcp_ports
                   .iter()
                   .map(|i| i.to_string())
                   .collect::<Vec<_>>()
                   .join(" ")
            ).unwrap();

            if !allowed_internet_udp_ports.is_empty() {
                write!(&mut firewall_rules_for_server, r#"
  networking.firewall.allowedUDPPorts = [ {} ];
"#,
                       allowed_internet_udp_ports
                       .iter()
                       .map(|i| i.to_string())
                       .collect::<Vec<_>>()
                       .join(" ")
                ).unwrap();
            }
        }
    }

    if !internet_if_tables.is_empty() {
        create_nftables_table_service(
            plan,
            "ip",
            "epl-internet-fw",
            &internet_if_tables
        );
    }

    plan.add_custom_nix_block(mk_nix_region("firewall", firewall_rules_for_server));
}

fn provision_server_packages(plans: &mut NixAllServerPlans, server: TableRowPointerServer) {
    let plan = plans.fetch_plan(server);

    let packages = [
      "vim",
      "wget",
      "curl",
      "htop",
      "dig",
      "jq",
      "postgresql",
      "natscli",
      "moreutils",
      "bmon",
      "iftop",
      "iotop",
      "sysstat",
      "inetutils",
      "iperf", // checking source/dest ips
      "netcat", // testing
      // l1 provisioning essentials
      "sqlite",
      "git",
      "gzip",
      "zstd",
      "tmux",
      "procmail", // lockfile for l1 provisioning
      "vault", // make command available everywhere
    ];

    for package in &packages {
        plan.add_nix_package(&package);
    }
}

pub(crate) fn gen_reload_service_on_path_change(
    res: &mut String,
    file_path: &str,
    service_unit_name: &str,
    target_service_name: &str,
    is_restart: bool,
) {
    assert!(!file_path.contains(' ') && !file_path.contains('\t') && !file_path.contains('\n'));
    let rld_command = if is_restart { "restart" } else { "reload" };

    res.push_str(&format!(
        r#"

    # reload service on file change
    systemd.services.{service_unit_name} = {{
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {{
        Type = "oneshot";
        # Only change file if modification occoured less than 10 seconds ago
        ExecStart = "/bin/sh -c 'find {file_path} -newermt -10seconds | grep . && /run/current-system/sw/bin/systemctl {rld_command} {target_service_name} || true'";
      }};

      enable = true;
    }};
    systemd.paths.{service_unit_name} = {{
      wantedBy = [ "multi-user.target" ];

      pathConfig = {{
        PathChanged = "{file_path}";
        Unit = "{service_unit_name}.service";
      }};

      enable = true;
    }};
"#
    ));
}

pub fn pad_string(input: &str, padding: &str) -> String {
    input.lines().map(|i| {
        if !i.is_empty() {
            format!("{padding}{i}")
        } else { String::new() }
    }).collect::<Vec<_>>().join("\n")
}

pub fn epl_arch_to_linux_arch(input: &str) -> &'static str {
    match input {
        "x86_64" => "x86_64",
        "arm64" => "aarch64",
        _ => panic!("Unknown EPL architecture {input}")
    }
}
