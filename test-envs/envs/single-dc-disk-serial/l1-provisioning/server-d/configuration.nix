# EDEN PLATFORM GENERATED NIX CONFIG
# changes done to this file will be overwritten by Eden platform
let
  pkgs = import (fetchTarball { url = "https://github.com/NixOS/nixpkgs/archive/057f9aecfb71c4437d2b27d3323df7f93c010b7e.tar.gz"; sha256 = "1ndiv385w1qyb3b18vw13991fzb9wg4cl21wglk89grsfsnra41k"; }) {};
  lib = pkgs.lib;
  modulesPath = pkgs.path + "/nixos/modules";
in

{ ... }:
{

    nix.settings = {
      tarball-ttl = 60 * 60 * 7;
      experimental-features = [ "nix-command" "flakes" ];
      substituters = [
        "http://10.17.0.1:12777/"
        "https://cache.nixos.org/"
      ];
      trusted-public-keys = [
        "epl-nix-cache:gi0dT5g3kUGLkTpP8WCgg79QSfAQ1yS7Ab5txcN5yag="
      ];

    };

    networking.hostId = "e8d7250b";


    virtualisation.docker.enable = true;
    time.timeZone = "UTC";

    users.users.root.hashedPassword = "!";
    security.sudo.wheelNeedsPassword = false;
    users.users.admin = {
      isNormalUser = true;
      home = "/home/admin";
      extraGroups = [ "docker" "wheel" "epl-prov" ];
      openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAICeIqJYRvj53KPvzNz+N6/Ip2Kk5issinxZ/SeQedFF0 epl-root-ssh-key"
      "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC3AkuDzzPrMaDav0kN7PIoaBU1Vtw1TfkHxWzPMrleocCltYl8TljwCqEJtmizx5DGKbXFQg31mRVswzuAq2vP2RFdPHQxfl5nJnWsQkelvpPO/Q3LUdtrm19zAgbbDL+AtIg3/lif6/2qNiWCSTfaUpjM7WOPszBNmMRGz/UBZTYc7COTt+I3lK8f6sBn5YyD796LBw6tsNpqfqF9NTAsLT8/PqrXeTpdxFe375gMxeIpNWeE5exMGJKgqnZCcOMOoKMJy61+wdEAYzDFNgIX7ZFvpBYQPf/rTs7LWgtyTSw3fqvMDnfwAf7oIF8rZRwYdVnqTGCWA2h3f4lOf6BERIPkKEK7/DGjmekKnXJrRiLSfcgRjri3VuGBxrJ+Va/Dn6e7o7CdzdJ+fkw7KxTFKuf17Z2r3ZFi1xOduIxXW8/QY6zhq2A11e+HsMe/oaBh3bRcpdMFmW5mqQjGm05xvxArSCAARBKkHjywGs6mRLN2PjNPYdzlI2J8nF6bmSk= henlo"

      ];
    };
    services.sshd.enable = true;
    services.openssh.settings.PermitRootLogin = "prohibit-password";
    services.getty.autologinUser = lib.mkDefault "root";

    swapDevices = [ ];

    nixpkgs.config.allowUnfreePredicate = pkg: builtins.elem (lib.getName pkg) [
        "consul"
        "nomad"
        "vault"
        "vault-bin"
     ];

    system.stateVersion = "23.11";

    environment.sessionVariables = {
      HISTCONTROL = "ignoreboth";
      NOMAD_ADDR = "https://nomad-servers.service.consul:4646";
      VAULT_ADDR = "https://vault.service.consul:8200";
    };

    security.pki.certificates = [
      ''-----BEGIN CERTIFICATE-----
MIIB0jCCAXmgAwIBAgIUJSfGGehDMhZ/st2l5v18+7w6hNYwCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAxMGQ0EgS2V5MB4XDTI0MDYxNTEzMjUwMFoXDTQxMDYxMTEzMjUw
MFowADBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABIfREL+Sy9b2Uu4z993tyYUL
rbp1tckuUbbiaXirJR5tRHVDYzWD4THsAvZH87a7Ykux0H2N+b8MUCM8WkopdT2j
gb8wgbwwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBR5Bx4Dw7ZFeEpkDDkpp6x0eLmK
MDAfBgNVHSMEGDAWgBT8KP9XEfBzHJr5IuXgkr4o9lDirzA9BgNVHREBAf8EMzAx
gg1lcGwtaW5mcmEubmV0gg8qLmVwbC1pbmZyYS5uZXSCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNHADBEAiAdSbX6OsG7+o5/XVexr7HA/CatiQi+hCxaa0dQ
2+IpGQIgG9d5is75sw8ki3PpM37a73fFJmjK8hf2QgQLNP9cn5o=
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBZDCCAQqgAwIBAgIUHM8ZVzqFS4FWZIBgI4wLwMufcBwwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjQwNjE1MTMyNTAwWhcNNDEwNjExMTMyNTAw
WjAQMQ4wDAYDVQQDEwVub21hZDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABMrE
kD+quFErQ/SY0HEjZWWudBs7ih4fdOHqyHY5s7NLSwvmfpJt24hU/3iOH1DhK9tS
aLnoOlgYOTNX1DLxaMGjQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBSTsI1m2M/XTmDmaZsiwAuIyML+HDAKBggqhkjOPQQDAgNI
ADBFAiEA6GZhViEkVzysJUAi0qo/XzRIJX+0X0dciQQEPIqLpV8CIBuPpzX7sbyK
Llvzp5UdBLhNku6Ti4LklEGCVPJ4zNuO
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBZTCCAQqgAwIBAgIUF8x+yXzfAonlV7xfN5Qh7IyEQ5swCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjQwNjE1MTMyNTAwWhcNNDEwNjExMTMyNTAw
WjAQMQ4wDAYDVQQDEwV2YXVsdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABCMb
pWYZNaofzdgZMM1cZfauBHLW9glea6oAoSiXQcw/s3kH3uf4o77HVgfI8/IJ4Crm
+h4Kv9SlT8LsvrRvaNKjQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBQjrfkbbpLmBASoOwmlgrLWkxKECzAKBggqhkjOPQQDAgNJ
ADBGAiEA9WTvUHA0M4tyhUn7eIzdKphlheZa8MaRAZOjt4hjackCIQDrCLZU5aGv
N+byf++Qo3oRmmmTeH04PTHjG5YM6otTFA==
-----END CERTIFICATE-----
''
    ];

    environment.systemPackages =
      let
        epl-consul-bootstrap = pkgs.writeShellScriptBin "epl-consul-bootstrap" ''

            export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

            while :
            do
                consul members | grep alive &>/dev/null && break
                sleep 1
            done

            consul acl policy list | grep '^agent-policy:$' &>/dev/null && exit 0

            cat > /tmp/epl-consul-agent-policy.hcl<<EOL
            node_prefix "" {
                policy = "write"
            }
            service_prefix "" {
                policy = "write"
            }
            EOL

            cat > /tmp/epl-consul-default-policy.hcl<<EOL
            # allow prometheus target scrapes
            agent_prefix "" {
                policy = "read"
            }
            node_prefix "" {
                policy = "read"
            }
            service_prefix "" {
                policy = "read"
            }
            # For DNS policiy, remove in the future when
            # we separate default token from DNS token
            query_prefix "" {
                policy = "read"
            }

            # inter DC routing, allow every node to access routes
            key_prefix "epl-interdc-routes/" {
                policy = "list"
            }

            # all l1 provisioning plans are sodium encrypted doesnt matter
            # if anyone reads, only intended node can decrypt
            key_prefix "epl-l1-plans/" {
                policy = "list"
            }
            EOL

            cat > /tmp/epl-consul-fast-l1-admin-policy.hcl<<EOL
            # allow plans upload for every server
            key_prefix "epl-l1-plans/" {
                policy = "write"
            }
            EOL

            ${pkgs.consul}/bin/consul acl policy create \
                -name "agent-policy" \
                -description "Agent Token Policy" \
                -rules @/tmp/epl-consul-agent-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Agent Token" \
                -policy-name "agent-policy" \
                -secret=$( sudo cat /run/keys/consul-agent-token.txt )

            ${pkgs.consul}/bin/consul acl policy create \
                -name "default-token" \
                -description "Default Token Policy" \
                -rules @/tmp/epl-consul-default-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Default Token" \
                -policy-name "default-token" \
                -secret=$( sudo cat /run/keys/consul-default-token.txt )

            ${pkgs.consul}/bin/consul acl policy create \
                -name "fast-l1-token" \
                -description "Fast L1 Admin Policy" \
                -rules @/tmp/epl-consul-fast-l1-admin-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Fast L1 Admin" \
                -policy-name "fast-l1-token" \
                -secret=$( sudo cat /run/keys/consul-fast-l1-token.txt )

        '';
        epl-consul-vrrp-acl = pkgs.writeShellScriptBin "epl-consul-vrrp-acl" ''

# NIX REGION consul_vrrp_bootstrap_script START

export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

while :
do
    consul members | grep alive &>/dev/null && break
    sleep 1
done

# NIX REGION consul_vrrp_bootstrap_script END

        '';
        epl-nomad-acl-bootstrap = pkgs.writeShellScriptBin "epl-nomad-acl-bootstrap" ''

            while ! curl -f -s https://nomad-servers.service.consul:4646 &>/dev/null
            do
                sleep 1
            done

            while [ "$( dig +short nomad-servers.service.consul | wc -l )" -lt 3 ]
            do
                sleep 1
            done

            while true
            do
              nomad acl bootstrap &> /run/secdir/nomad-bootstrap-output.txt.tmp
              if cat /run/secdir/nomad-bootstrap-output.txt.tmp | grep 'No cluster leader'
              then
                sleep 2
                continue
              fi

              if cat /run/secdir/nomad-bootstrap-output.txt.tmp | grep 'Secret ID'
              then
                mv -f /run/secdir/nomad-bootstrap-output.txt.tmp /run/secdir/nomad-bootstrap-output.txt
              fi

              break
            done

        '';
        epl-nomad-acl-policies = pkgs.writeShellScriptBin "epl-nomad-acl-policies" ''


            if [ -z "$NOMAD_TOKEN" ]
            then
                echo Must set NOMAD_TOKEN for this script
                exit 7
            fi

            while ! curl -f -s https://nomad-servers.service.consul:4646 &>/dev/null
            do
                sleep 1
            done

            cat > /tmp/epl-nomad-anonymous-policy.hcl<<EOL
              namespace "*" {
                policy       = "read"
                capabilities = [
                  "list-jobs"
                ]
              }

              agent {
                policy = "read"
              }

              operator {
                policy = "read"
              }

              quota {
                policy = "read"
              }

              node {
                policy = "read"
              }

              host_volume "*" {
                policy = "read"
              }
            EOL

            nomad acl policy apply -description "Anonymous policy" anonymous /tmp/epl-nomad-anonymous-policy.hcl

        '';
        epl-nomad-vault-policies = pkgs.writeShellScriptBin "epl-nomad-vault-policies" ''

            if [ -z "$VAULT_TOKEN" ]
            then
                echo Must set VAULT_TOKEN for this script
                exit 7
            fi

            while ! curl -f -s https://vault.service.consul:8200 &>/dev/null
            do
                sleep 1
            done

            cat > /tmp/epl-nomad-vault-token-policy.json<<EOL
            {
                "disallowed_policies": "nomad-server",
                "token_explicit_max_ttl": 0,
                "name": "nomad-cluster",
                "orphan": true,
                "token_period": 259200,
                "renewable": true
            }
            EOL

            cat > /tmp/epl-nomad-vault-policy.hcl<<EOL
            # Allow creating tokens under "nomad-cluster" token role. The token role name
            # should be updated if "nomad-cluster" is not used.
            path "auth/token/create/nomad-cluster" {
                capabilities = ["update"]
            }

            # Allow looking up "nomad-cluster" token role. The token role name should be
            # updated if "nomad-cluster" is not used.
            path "auth/token/roles/nomad-cluster" {
                capabilities = ["read"]
            }

            # Allow looking up the token passed to Nomad to validate # the token has the
            # proper capabilities. This is provided by the "default" policy.
            path "auth/token/lookup-self" {
                capabilities = ["read"]
            }

            # Allow looking up incoming tokens to validate they have permissions to access
            # the tokens they are requesting. This is only required if
            # allow_unauthenticated is set to false.
            path "auth/token/lookup" {
                capabilities = ["update"]
            }

            # Allow revoking tokens that should no longer exist. This allows revoking
            # tokens for dead tasks.
            path "auth/token/revoke-accessor" {
                capabilities = ["update"]
            }

            # Allow checking the capabilities of our own token. This is used to validate the
            # token upon startup.
            path "sys/capabilities-self" {
                capabilities = ["update"]
            }

            # Allow our own token to be renewed.
            path "auth/token/renew-self" {
                capabilities = ["update"]
            }
            EOL

            vault policy write nomad-server /tmp/epl-nomad-vault-policy.hcl
            vault write /auth/token/roles/nomad-cluster @/tmp/epl-nomad-vault-token-policy.json

            ORIGINAL_TOKEN=$VAULT_TOKEN
            export VAULT_TOKEN=$1
            if ! vault token lookup
            then
                # token invalid, needs to be created
                export VAULT_TOKEN=$ORIGINAL_TOKEN
                NEW_TOKEN=$( vault token create -policy nomad-server -period 72h -orphan | grep 'hvs.' | sed -E 's/^.* hvs/hvs/' )
                echo "NOMAD_VAULT_TOKEN $NEW_TOKEN"
            fi

        '';
        epl-vault-operator-init = pkgs.writeShellScriptBin "epl-vault-operator-init" ''

export VAULT_ADDR=$1
# pass initial text as argument or init vault

${pkgs.vault-bin}/bin/vault operator init &> /run/secdir/vault-init-output.txt.tmp

if grep -e 'Initial Root Token:' /run/secdir/vault-init-output.txt.tmp &> /dev/null
then
    mv /run/secdir/vault-init-output.txt.tmp /run/secdir/vault-init-output.txt

    echo Unsealing initial vault after 3 seconds...
    sleep 3

    for UK in $(seq 1 3)
    do
        UNSEAL_KEY=$( cat /run/secdir/vault-init-output.txt | grep "Unseal Key $UK:" | sed -E 's/^.*: //' )
        ${pkgs.vault-bin}/bin/vault operator unseal $UNSEAL_KEY
    done

    export VAULT_TOKEN=$( cat /run/secdir/vault-init-output.txt | grep "Initial Root Token:" | sed -E 's/^.*: //' )
    ${pkgs.vault-bin}/bin/vault secrets enable -path=epl kv-v2 || true
fi

        '';
        epl-vault-operator-unseal = pkgs.writeShellScriptBin "epl-vault-operator-unseal" ''

export VAULT_ADDR=$1
# pass initial text as argument or init vault
VAULT_KEY_BASE64=$2

export VAULT_INIT_RES=$( echo "$VAULT_KEY_BASE64" | base64 -d )

if [ -z "$VAULT_INIT_RES" ]
then
    echo Failed to initialize vault/key is not set
    exit 7
fi

# make sure vault is responsive before we try to unseal
while ! curl -s $VAULT_ADDR/v1/sys/seal-status | grep '"initialized":'
do
  sleep 3
done

# make sure vault is initialized
while ! curl -s $VAULT_ADDR/v1/sys/seal-status | grep '"initialized":true'
do
  sleep 3
done

if curl -s $VAULT_ADDR/v1/sys/seal-status | grep '"sealed":true'
then
  for UK in $(seq 1 3)
  do
    UNSEAL_KEY=$( echo "$VAULT_INIT_RES" | grep "Unseal Key $UK:" | sed -E 's/^.*: //' )
    ${pkgs.vault-bin}/bin/vault operator unseal $UNSEAL_KEY
  done
fi

export VAULT_TOKEN=$( echo "$VAULT_INIT_RES" | grep "Initial Root Token:" | sed -E 's/^.*: //' )
${pkgs.vault-bin}/bin/vault secrets enable -path=epl kv-v2 || true

        '';
        epl-wait-for-consul = pkgs.writeShellScriptBin "epl-wait-for-consul" ''

while ! ${pkgs.consul}/bin/consul members
do
  sleep 5
done

        '';

      in
      [
        pkgs.bmon
        pkgs.cadvisor
        pkgs.curl
        pkgs.dig
        pkgs.git
        pkgs.gzip
        pkgs.htop
        pkgs.iftop
        pkgs.inetutils
        pkgs.iotop
        pkgs.iperf
        pkgs.jq
        pkgs.moreutils
        pkgs.natscli
        pkgs.netcat
        pkgs.nftables
        pkgs.nomad
        pkgs.postgresql
        pkgs.procmail
        pkgs.prometheus-node-exporter
        pkgs.sqlite
        pkgs.sysstat
        pkgs.tmux
        pkgs.vault
        pkgs.vector
        pkgs.vim
        pkgs.wget
        pkgs.wireguard-tools
        pkgs.zstd
        epl-consul-bootstrap
        epl-consul-vrrp-acl
        epl-nomad-acl-bootstrap
        epl-nomad-acl-policies
        epl-nomad-vault-policies
        epl-vault-operator-init
        epl-vault-operator-unseal
        epl-wait-for-consul
      ];

# NIX REGION static_node_routes START

    networking.interfaces."eth1".ipv4.routes = [

      { address = "0.0.0.0"; prefixLength = 0; via = "77.77.77.1"; }

    ];

# NIX REGION static_node_routes END

    boot.kernel.sysctl = {
      # for loki ScyllaDB
      "fs.aio-max-nr" = 1048576;
    };

# NIX REGION epl_nft_rules_epl-internet-fw START

            networking.nftables.tables.epl-internet-fw = {
              family = "ip";
              content = ''

       chain EPL_INTERNET_FIREWALL {
           type filter hook prerouting priority mangle + 20; policy accept;
           iifname eth1 ip saddr != { 10.0.0.0/8, 172.21.0.0/16 } ip daddr != { 77.77.77.13/32 } drop comment "Disallow traffic from internet to internal networks";
       }

              '';
            };

# NIX REGION epl_nft_rules_epl-internet-fw END

# NIX REGION firewall START

  networking.hostName = "server-d";
  networking.firewall.allowPing = true;
  networking.firewall.enable = true;
  networking.firewall.checkReversePath = false;
  networking.firewall.trustedInterfaces = [

    "eth0"

    "wg0"

  ];

  networking.firewall.interfaces."eth1".allowedTCPPorts = [ 22 80 443 53 ];

  networking.firewall.interfaces."eth1".allowedUDPPorts = [ 53 51820 ];

# NIX REGION firewall END

   programs.bash.promptInit = ''
     # Provide a nice prompt if the terminal supports it.
     if [ "$TERM" != "dumb" ] || [ -n "$INSIDE_EMACS" ]; then
       PROMPT_COLOR="1;31m"
       ((UID)) && PROMPT_COLOR="1;32m"
       if [ -n "$INSIDE_EMACS" ]; then
         # Emacs term mode doesn't support xterm title escape sequence (\e]0;)
         PS1="\n\[\033[$PROMPT_COLOR\][\u@server-d.dc1.us-west.single-dc-disk-serial:\w]\\$\[\033[0m\] "
       else
         PS1="\n\[\033[$PROMPT_COLOR\][\[\e]0;\u@server-d.dc1.us-west.single-dc-disk-serial: \w\a\]\u@server-d.dc1.us-west.single-dc-disk-serial:\w]\\$\[\033[0m\] "
       fi
       if test "$TERM" = "xterm"; then
         PS1="\[\033]2;server-d.dc1.us-west.single-dc-disk-serial:\u:\w\007\]$PS1"
       fi
     fi
   '';

     # l1 agent
     systemd.services.l1-fast-agent = {
       wantedBy = [ "multi-user.target" ];
       requires = [ "network-online.target" ];
       after = [ "network-online.target" "consul.service" ];
       script =
       let
         l1Checker = import ./l1-checker/default.nix { pkgs = pkgs; };
       in
       ''
         export PATH=/run/current-system/sw/bin:$PATH
         # wait for consul to become available
         while ! ${pkgs.consul}/bin/consul kv get epl-l1-plans/server-d
         do
           sleep 7
         done

         ${pkgs.consul}/bin/consul watch \
           -type=key -key=epl-l1-plans/server-d \
           ${l1Checker}/checker \
             /run/keys/l1-fast-prov-decryption-key \
             /run/keys/l1-fast-prov-admin-pub-key \
             /run/secdir/l1-fast-plan.zst
       '';

       serviceConfig = {
         User = "root";
         Group = "root";
         Type = "simple";
         Restart = "always";
         RestartSec = "20";
       };

       enable = true;
     };
# NIX REGION custom_hardware START
    imports =
      [ "${modulesPath}/profiles/qemu-guest.nix" ];


  boot.zfs.devNodes = "/dev/disk/by-label/rpool";
  services.zfs.expandOnBoot = "all";

  boot.loader.grub = {
    enable = true;
    zfsSupport = true;
    efiSupport = false;
    efiInstallAsRemovable = false;
    mirroredBoots = [
      { devices = [ "nodev"]; path = "/boot"; }
    ];
  };

  fileSystems."/" =
    { device = "rpool/root";
      fsType = "zfs";
    };

  fileSystems."/nix" =
    { device = "rpool/nix";
      fsType = "zfs";
    };

  fileSystems."/var" =
    { device = "rpool/var";
      fsType = "zfs";
    };

  fileSystems."/var/log" =
    { device = "rpool/var/log";
      fsType = "zfs";
    };

  fileSystems."/boot" =
    { device = "/dev/vda2";
      fsType = "vfat";
    };


    boot.initrd.availableKernelModules = [ "zfs" "ahci" "xhci_pci" "virtio_pci" "virtio_blk" "virtio_console" ];
    boot.initrd.kernelModules = [ ];
    boot.kernelModules = [ "zfs" "kvm-amd" ];
    boot.extraModulePackages = [ ];

    boot.kernelParams = [ "console=ttyS0,115200n8" ];
    boot.loader.grub.extraConfig = "
      serial --speed=115200 --unit=0 --word=8 --parity=no --stop=1
      terminal_input serial
      terminal_output serial
    ";


    hardware.cpu.amd.updateMicrocode = lib.mkDefault true;
# NIX REGION custom_hardware END
    users.users.named.extraGroups = ["keys"];
    services.bind =
    {
        enable = true;
        extraOptions = ''
          recursion yes;
          dnssec-validation auto;
          validate-except { consul.; };
          key-directory "/run/dnsseckeys";
        '';
        forwarders = [ "1.1.1.1" ];
        cacheNetworks = [
          # bind can be internet
          # facing depending on DC
          "0.0.0.0/0"
        ];
        extraConfig = ''
          trust-anchors {
  epl-infra.net. initial-key 257 3 15 "fLtoDrcSUwvJd8s/HI5mWWHmz1aS9j668RC5EdpDeOw=";
  us-west.epl-infra.net. initial-key 257 3 15 "wneIDfnzKyR4QhXc3ZfVQ5muabKdyrdxLJ/O2FQW9iE=";
  10.in-addr.arpa. initial-key 257 3 15 "YBxGb3PUdnudpitkWV1lZvvWhdS0fb5brdGb7TGZRUA=";
  17.10.in-addr.arpa. initial-key 257 3 15 "eGFDMgkxWDsWIhdC6UzVBcWa21BEn0uG3EOTrewnnqc=";
  in-addr.arpa. initial-key 257 3 15 "FtdGrccSFtATCprxlY3EFRx3ryE0g9e0kDtB8CZEnNw=";
};




          dnssec-policy epl {
            keys {
              ksk key-directory lifetime unlimited algorithm ED25519;
              zsk key-directory lifetime unlimited algorithm ED25519;
            };
            dnskey-ttl 300;
            max-zone-ttl 3600;
            parent-ds-ttl 3600;
            parent-propagation-delay 2h;
            publish-safety 7d;
            retire-safety 7d;
            signatures-refresh 1439h;
            signatures-validity 90d;
            signatures-validity-dnskey 90d;
            zone-propagation-delay 2h;
          };

view lan {
          # add VPN address so local user integration tests pass
          match-clients { 10.0.0.0/8; 172.21.0.0/16; localhost; };
          zone "consul." IN {
              type forward;
              forward only;
              forwarders { 127.0.0.1 port 8600; };
          };


zone "epl-infra.net." {
  type master;
  file "/run/named/private-epl-infra.net.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.17.0.12;
  };
};
zone "10.in-addr.arpa." {
  type master;
  file "/run/named/private-10.in-addr.arpa.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.17.0.12;
  };
};
zone "us-west.epl-infra.net." {
  type master;
  file "/run/named/private-us-west.epl-infra.net.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.17.0.12;
  };
};
zone "17.10.in-addr.arpa." {
  type master;
  file "/run/named/private-17.10.in-addr.arpa.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    10.17.0.12;
  };
};

};

view internet {
          match-clients { any; };
          recursion no;
zone "epl-infra.net." {
  type master;
  file "/run/named/public-epl-infra.net.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    77.77.77.12;
  };
};
zone "in-addr.arpa." {
  type master;
  file "/run/named/public-in-addr.arpa.zone";
  dnssec-policy epl;
  inline-signing yes;
  allow-transfer {
    77.77.77.12;
  };
};

};

        '';
    };

    virtualisation.docker.daemon.settings = { "registry-mirrors" = [ "http://10.17.0.1:12778" "http://epl-docker-registry.service.consul:5000" ]; };
    virtualisation.docker.extraOptions = "--insecure-registry http://epl-docker-registry.service.consul:5000";

    users.groups.epl-prov = {};

    services.consul = {
      enable = true;
      webUi = true;
      forceAddrFamily = "ipv4";
      extraConfigFiles = [
        "/run/keys/consul-config.json"
      ];
    };
    users.users.consul.extraGroups = ["keys"];


    # reload service on file change
    systemd.services.consul-restart = {
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "oneshot";
        # Only change file if modification occoured less than 10 seconds ago
        ExecStart = "/bin/sh -c 'find /run/keys/consul-config.json -newermt -10seconds | grep . && /run/current-system/sw/bin/systemctl restart consul.service || true'";
      };

      enable = true;
    };
    systemd.paths.consul-restart = {
      wantedBy = [ "multi-user.target" ];

      pathConfig = {
        PathChanged = "/run/keys/consul-config.json";
        Unit = "consul-restart.service";
      };

      enable = true;
    };

    systemd.services.nomad = {
      wantedBy = [ "multi-user.target" ];
      requires = [ "network-online.target" ];
      after = [ "network-online.target" "consul.service" ];
      path = [ pkgs.iproute2 ];

      serviceConfig = {
        User = "root";
        Group = "root";
        Type = "simple";
        ExecStartPre = [
            "+${pkgs.coreutils}/bin/mkdir -p /var/lib/nomad"
            "+${pkgs.coreutils}/bin/chmod 700 /var/lib/nomad"
        ];
        ExecStart = "${pkgs.nomad}/bin/nomad agent -config=/run/keys/nomad-config.hcl";
        ExecReload = "/bin/kill -HUP $MAINPID";
        KillMode = "process";
        KillSignal = "SIGINT";
        LimitNOFILE = "infinity";
        LimitNPROC = "infinity";
        Restart = "always";
        RestartSec = "20";
        TasksMax = "infinity";
      };

      enable = true;
    };


    # reload service on file change
    systemd.services.nomad-restart = {
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "oneshot";
        # Only change file if modification occoured less than 10 seconds ago
        ExecStart = "/bin/sh -c 'find /run/keys/nomad-config.hcl -newermt -10seconds | grep . && /run/current-system/sw/bin/systemctl restart nomad.service || true'";
      };

      enable = true;
    };
    systemd.paths.nomad-restart = {
      wantedBy = [ "multi-user.target" ];

      pathConfig = {
        PathChanged = "/run/keys/nomad-config.hcl";
        Unit = "nomad-restart.service";
      };

      enable = true;
    };

    users.users.vault = {
        isSystemUser = true;
        description = "Vault service";
        extraGroups = ["keys"];
        group = "vault";
    };
    users.groups.vault = {};

    systemd.services.vault = {
      wantedBy = [ "multi-user.target" ];
      requires = [ "network-online.target" ];
      after = [ "network-online.target" "consul.service" ];

      serviceConfig = {
        User = "vault";
        Group = "vault";
        Type = "simple";
        ExecStartPre = [
            "+${pkgs.coreutils}/bin/mkdir -p /var/lib/vault"
            "+${pkgs.coreutils}/bin/chown vault:vault /var/lib/vault"
            "+${pkgs.coreutils}/bin/chmod 700 /var/lib/vault"
        ];
        ExecStart = "${pkgs.vault-bin}/bin/vault server -config=/run/keys/vault_config.hcl -log-level=info";
        Restart = "always";
        RestartSec = "20";
        TasksMax = "infinity";
      };

      enable = true;
    };

    users.users.node-exp = {
        isSystemUser = true;
        description = "Vault service";
        extraGroups = ["keys"];
        group = "node-exp";
    };
    users.groups.node-exp = {};

    systemd.services.node_exporter = {
      wantedBy = [ "multi-user.target" ];
      requires = [ "network-online.target" ];
      after = [ "network-online.target" ];

      serviceConfig = {
        User = "node-exp";
        Group = "node-exp";
        Type = "simple";
        ExecStart = "${pkgs.prometheus-node-exporter}/bin/node_exporter" +
          " --collector.systemd" +
          " --collector.textfile" +
          " --collector.textfile.directory=/var/lib/node_exporter" +
          " --web.listen-address=10.17.0.13:9100" +
          " --web.telemetry-path=/metrics";
        Restart = "always";
        RestartSec = "1";
        SyslogIdentifier = "node_exporter";
        ProtectHome = "yes";
        NoNewPrivileges = "yes";
        ProtectSystem = "strict";
        ProtectControlGroups = "true";
        ProtectKernelModules = "true";
        ProtectKernelTunables = "yes";
      };

      enable = true;
    };

    systemd.services.cadvisor = {
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        User = "root";
        Group = "root";
        Type = "simple";
        ExecStart = "${pkgs.cadvisor}/bin/cadvisor" +
          " --listen_ip=10.17.0.13" +
          " --port=9280" +
          " --prometheus_endpoint=/metrics" +
          " --docker_only" +
          " --store_container_labels=false" +
          " --whitelisted_container_labels=com.hashicorp.nomad.job.name,com.hashicorp.nomad.node_name,com.hashicorp.nomad.namespace";
        Restart = "always";
        RestartSec = "1";
      };

      enable = true;
    };


    # reload service on file change
    systemd.services.vector-restart = {
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "oneshot";
        # Only change file if modification occoured less than 10 seconds ago
        ExecStart = "/bin/sh -c 'find /run/keys/vector.toml -newermt -10seconds | grep . && /run/current-system/sw/bin/systemctl restart vector.service || true'";
      };

      enable = true;
    };
    systemd.paths.vector-restart = {
      wantedBy = [ "multi-user.target" ];

      pathConfig = {
        PathChanged = "/run/keys/vector.toml";
        Unit = "vector-restart.service";
      };

      enable = true;
    };

    users.users.vector = {
        isSystemUser = true;
        description = "Vector service";
        extraGroups = ["keys" "systemd-journal" "docker" "epl-prov" ];
        group = "vector";
    };
    users.groups.vector = {};

    systemd.services.vector = {
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        User = "vector";
        Group = "vector";
        Type = "simple";
        ExecStartPre = "${pkgs.vector}/bin/vector validate --config-toml=/run/keys/vector.toml";
        ExecStart = "${pkgs.vector}/bin/vector --threads=4 --config-toml=/run/keys/vector.toml";
        Restart = "always";
        RestartSec = "10";
      };

      enable = true;
    };

    networking.nftables.enable = true;

# NIX REGION wireguard_configs START

  systemd.services.wireguard-wg0 = {
    description = "WireGuard Tunnel - wg0";
    after = [ "network-pre.target" ];
    wants = [ "network.target" ];
    before = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];
    environment.DEVICE = "wg0";
    path = with pkgs; [ kmod iproute2 wireguard-tools ];

    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      Restart = "on-failure";
      RestartSec = "10s";
    };

    script = ''

modprobe wireguard || true
ip link add dev "wg0" type wireguard

# this might fail as kernel seems to remember ip address from previously
ip address add "172.21.7.11/16" dev "wg0" || true
wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
ip link set up dev "wg0"

# peers
wg set wg0 peer "6eta+uZZDCSBaDirTjxFCcVH+fBi8MoVBEhVUL1VgWI=" allowed-ips "172.21.7.254/32"

    '';

    postStop = ''
      ip link del dev "wg0"
    '';
  };

# NIX REGION wireguard_configs END

# NIX REGION epl_nft_rules_epl-nat START

            networking.nftables.tables.epl-nat = {
              family = "ip";
              content = ''

       chain EPL_POSTROUTING {
           type nat hook postrouting priority 0;

               ip saddr 172.21.7.254/32 ip daddr 10.0.0.0/8 masquerade comment "Admin VPN";
               ip saddr 10.0.0.0/8 ip daddr 10.0.0.0/8 return comment "Inter DC EPL traffic";
               ip saddr 10.17.0.0/16 ip daddr != { 10.0.0.0/8 } masquerade comment "Internet for private EPL nodes";
       }

              '';
            };

# NIX REGION epl_nft_rules_epl-nat END

# NIX REGION keepalived START

  systemd.services.keepalived = {
    description = "Keepalive Daemon (LVS and VRRP)";
    wantedBy = [ "multi-user.target" ];
    after = [ "network.target" "network-online.target" "syslog.target" ];
    wants = [ "network-online.target" ];
    serviceConfig = {
      Type = "forking";
      PIDFile = "/run/keepalived.pid";
      KillMode = "process";
      RuntimeDirectory = "keepalived";
      ExecStart = "${pkgs.keepalived}/sbin/keepalived -f /run/keys/keepalived.conf -p /run/keepalived.pid";
      ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
      Restart = "always";
      RestartSec = "1s";
    };
  };

# NIX REGION keepalived END


    # reload service on file change
    systemd.services.keepalived-restart = {
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "oneshot";
        # Only change file if modification occoured less than 10 seconds ago
        ExecStart = "/bin/sh -c 'find /run/keys/keepalived.conf -newermt -10seconds | grep . && /run/current-system/sw/bin/systemctl reload keepalived.service || true'";
      };

      enable = true;
    };
    systemd.paths.keepalived-restart = {
      wantedBy = [ "multi-user.target" ];

      pathConfig = {
        PathChanged = "/run/keys/keepalived.conf";
        Unit = "keepalived-restart.service";
      };

      enable = true;
    };

    services.prometheus.exporters.zfs.enable = true;
    services.prometheus.exporters.zfs.port = 9134;

    networking.useDHCP = false;

    networking.interfaces.eth0.ipv4.addresses = [
      { address = "10.17.0.13"; prefixLength = 24; }

    ];

    networking.interfaces.eth1.ipv4.addresses = [
      { address = "77.77.77.13"; prefixLength = 24; }

    ];

}
