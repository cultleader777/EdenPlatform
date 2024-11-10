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
        "https://cache.nixos.org/"
      ];
      trusted-public-keys = [
        "epl-nix-cache:knUE4VbRh9LazjdBw2qsBAMPgtyf5OI7cAVqF1hLuyw="
      ];

    };

    networking.hostId = "a79b8498";


    virtualisation.docker.enable = true;
    time.timeZone = "UTC";

    users.users.root.hashedPassword = "!";
    security.sudo.wheelNeedsPassword = false;
    users.users.admin = {
      isNormalUser = true;
      home = "/home/admin";
      extraGroups = [ "docker" "wheel" "epl-prov" ];
      openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIONUZdMtNbaNBA+F2IS18RAcVToqkvGVDw4/3nFvE9TR epl-root-ssh-key"
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
MIIB0zCCAXmgAwIBAgIUFXKgm49joHEO2/JgwMTGkqxYucswCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAxMGQ0EgS2V5MB4XDTIzMTIxNzA2MTAwMFoXDTQwMTIxMjA2MTAw
MFowADBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABK7zoU22emDngas6AZKLfnq3
1PgLNpNOkY2972BxKfdsdHXslUatwjEV0D66bzvL1qvi7M1o5h1aHMv1fq8soxej
gb8wgbwwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBQY7DF/fuqV2uraikN0w7zZdDoD
VTAfBgNVHSMEGDAWgBT8ui2xHlr0+GQMkGXxm7Y6ZSVGOzA9BgNVHREBAf8EMzAx
gg1lcGwtaW5mcmEubmV0gg8qLmVwbC1pbmZyYS5uZXSCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNIADBFAiEAhvuw0GpetbyMgepkcZZV0rTT2uQ/iyKXHuDF
gdlBjG4CIG1L4SCFCw7rIRNcFfsfLskAwb7K7SqftCeQHJ9Z3Qec
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBYzCCAQqgAwIBAgIUcwf2Wx0BAcab9GKxA/3NBEqkYLgwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjMxMjE3MDYxMDAwWhcNNDAxMjEyMDYxMDAw
WjAQMQ4wDAYDVQQDEwVub21hZDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABLVj
Pe+gqLwyF3DQMRAabqVvVBJ8+BhSHpF5F9ps9x8pO9oym8WUsMEDDQy5MWOqryIY
kcm2w6yeFqQtI2wkGy6jQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBQZUCUg2M10b1hICyW+TrTBDwZpBjAKBggqhkjOPQQDAgNH
ADBEAiAsPC7WS/zkA2vtfdQsEkENH9qeLOIAqLdoCbi+N+9ktAIgYcMpR0b5tfQr
yfNQWyGeMzoFRZ8sgpeOXVWXki2Cqmw=
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBZDCCAQqgAwIBAgIUKJGRPQ0w0nLzoNYftEqb9Qqk7AwwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjMxMjE3MDYxMDAwWhcNNDAxMjEyMDYxMDAw
WjAQMQ4wDAYDVQQDEwV2YXVsdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABI0r
mro4rc4aaNwfFajPMuDVsfsoHuOw4K1FF4524iZ5Yfw4mlOU0PDWMjTjNHAUQhdU
JETmg35q6Tn5imq5v82jQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBSfhC32z/CuhNhrc5hGF2wVJrLDFjAKBggqhkjOPQQDAgNI
ADBFAiEAo+RsRay1tItvzyeJDfDb2gja7DVsL/cXCzJb6gn6B6UCIAZ+O5ZoouUS
EinlUnb7MGKl0z5/dCH8pzSnisY3Amto
-----END CERTIFICATE-----
''
    ];

    environment.systemPackages =
      let
        epl-acme-vault-policies = pkgs.writeShellScriptBin "epl-acme-vault-policies" ''

                if [ -z "$VAULT_TOKEN" ]
                then
                    echo Must set VAULT_TOKEN for this script
                    exit 7
                fi

                while ! curl -f -s https://vault.service.consul:8200 &>/dev/null
                do
                    sleep 1
                done

                cat > /tmp/epl-acme-vault-token-policy.json<<EOL
                {
                    "token_explicit_max_ttl": 0,
                    "name": "epl-acme-certs",
                    "orphan": true,
                    "token_period": 259200,
                    "renewable": true
                }
                EOL

                cat > /tmp/epl-acme-vault-policy.hcl<<EOL
                path "epl/data/certs/*" {
                    capabilities = ["read", "list", "create", "patch", "update", "delete"]
                }

                # allow updating external load balancer as well
                path "epl/data/ext-lb" {
                    capabilities = ["read", "list", "create", "patch", "update", "delete"]
                }

                # Allow looking up the token passed to validate the token has the
                # proper capabilities. This is provided by the "default" policy.
                path "auth/token/lookup-self" {
                    capabilities = ["read"]
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

                vault policy write epl-acme-certs /tmp/epl-acme-vault-policy.hcl
                vault write /auth/token/roles/epl-acme-certs @/tmp/epl-acme-vault-token-policy.json

                ORIGINAL_TOKEN=$VAULT_TOKEN
                export VAULT_TOKEN=$1
                if ! vault token lookup
                then
                    # token invalid, needs to be created
                    export VAULT_TOKEN=$ORIGINAL_TOKEN
                    NEW_TOKEN=$( vault token create -policy epl-acme-certs -period 168h -orphan | grep 'hvs.' | sed -E 's/^.* hvs/hvs/' )
                    echo "ACME_CERTS_VAULT_TOKEN $NEW_TOKEN"
                fi

        '';
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

if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc1:$'
then
    cat > /tmp/epl-consul-vrrp-dc1-policy.hcl<<EOL
    key_prefix "epl-interdc-routes/dc1" {
        policy = "write"
    }
EOL

    ${pkgs.consul}/bin/consul acl policy create \
        -name "vrrp-policy-dc1" \
        -description "VRRP policy for datacenter dc1" \
        -rules @/tmp/epl-consul-vrrp-dc1-policy.hcl

    ${pkgs.consul}/bin/consul acl token create \
        -description "VRRP Token for datacenter dc1" \
        -policy-name "vrrp-policy-dc1" \
        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc1.txt )
fi


${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc1/l3_vpn_hop || echo '
            # ROUTES CREATE
            ip route add 10.19.0.0/16 via 10.17.128.10

            # ROUTES DELETE
            ip route del 10.19.0.0/16

            # FINISH
' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/l3_vpn_hop -


# after policy provisioning key is no longer needed
rm -f /run/keys/consul-vrrp-token-dc1.txt


if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc2:$'
then
    cat > /tmp/epl-consul-vrrp-dc2-policy.hcl<<EOL
    key_prefix "epl-interdc-routes/dc2" {
        policy = "write"
    }
EOL

    ${pkgs.consul}/bin/consul acl policy create \
        -name "vrrp-policy-dc2" \
        -description "VRRP policy for datacenter dc2" \
        -rules @/tmp/epl-consul-vrrp-dc2-policy.hcl

    ${pkgs.consul}/bin/consul acl token create \
        -description "VRRP Token for datacenter dc2" \
        -policy-name "vrrp-policy-dc2" \
        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc2.txt )
fi


${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc2/l3_vpn_hop || echo '
            # ROUTES CREATE
            ip route add 10.19.0.0/16 via 10.18.128.10

            # ROUTES DELETE
            ip route del 10.19.0.0/16

            # FINISH
' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc2/l3_vpn_hop -


# after policy provisioning key is no longer needed
rm -f /run/keys/consul-vrrp-token-dc2.txt


if ! ${pkgs.consul}/bin/consul acl policy list | grep '^vrrp-policy-dc3:$'
then
    cat > /tmp/epl-consul-vrrp-dc3-policy.hcl<<EOL
    key_prefix "epl-interdc-routes/dc3" {
        policy = "write"
    }
EOL

    ${pkgs.consul}/bin/consul acl policy create \
        -name "vrrp-policy-dc3" \
        -description "VRRP policy for datacenter dc3" \
        -rules @/tmp/epl-consul-vrrp-dc3-policy.hcl

    ${pkgs.consul}/bin/consul acl token create \
        -description "VRRP Token for datacenter dc3" \
        -policy-name "vrrp-policy-dc3" \
        -secret=$( sudo cat /run/keys/consul-vrrp-token-dc3.txt )
fi


${pkgs.consul}/bin/consul kv get epl-interdc-routes/dc3/10.19.0.0p24 || echo '
            # ROUTES CREATE
            ip route add 10.0.0.0/8 via 10.19.0.10
            ip route add 0.0.0.0/0 via 10.19.0.10

            # ROUTES DELETE
            ip route del 10.0.0.0/8
            ip route del 0.0.0.0/0

            # FINISH
' | ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc3/10.19.0.0p24 -


# after policy provisioning key is no longer needed
rm -f /run/keys/consul-vrrp-token-dc3.txt


# NIX REGION consul_vrrp_bootstrap_script END

        '';
        epl-consul-vrrp-switch = pkgs.writeShellScriptBin "epl-consul-vrrp-switch" ''

# NIX REGION consul_vrrp_switch_script START

/run/current-system/sw/bin/echo '
# ROUTES CREATE
ip route add 10.19.0.0/16 via 10.17.128.10

# ROUTES DELETE
ip route del 10.19.0.0/16

# FINISH
' | \
  CONSUL_HTTP_TOKEN=$( ${pkgs.coreutils}/bin/cat /run/keys/consul-vrrp-token.txt ) \
  ${pkgs.consul}/bin/consul kv put epl-interdc-routes/dc1/10.17.0.0p24 -

# NIX REGION consul_vrrp_switch_script END

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
        epl-nomad-consul-acl-bootstrap = pkgs.writeShellScriptBin "epl-nomad-consul-acl-bootstrap" ''

            export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

            while :
            do
                consul members | grep alive &>/dev/null && break
                sleep 1
            done

            consul acl policy list | grep '^nomad-server:$' &>/dev/null && exit 0

            cat > /tmp/epl-nomad-server-consul-acl-policy.hcl<<EOL
            agent_prefix "" {
                policy = "read"
            }

            node_prefix "" {
                policy = "read"
            }

            service_prefix "" {
                policy = "write"
            }

            # TODO: remove after nomad 1.9 and use consul identities instead
            key_prefix "epl-kv/" {
                policy = "read"
            }

            acl = "write"
            EOL

            cat > /tmp/epl-nomad-client-consul-acl-policy.hcl<<EOL
            agent_prefix "" {
                policy = "read"
            }

            node_prefix "" {
                policy = "read"
            }

            service_prefix "" {
                policy = "write"
            }

            acl = "write"
            EOL

            ${pkgs.consul}/bin/consul acl policy create \
                -name "nomad-server" \
                -description "Nomad Server Policy" \
                -rules @/tmp/epl-nomad-server-consul-acl-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Nomad Server Token" \
                -policy-name "nomad-server" \
                -secret=$( sudo cat /run/keys/nomad-server-consul-acl-token.txt )

            ${pkgs.consul}/bin/consul acl policy create \
                -name "nomad-client" \
                -description "Nomad Client Policy" \
                -rules @/tmp/epl-nomad-client-consul-acl-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Nomad Client Token" \
                -policy-name "nomad-client" \
                -secret=$( sudo cat /run/keys/nomad-client-consul-acl-token.txt )

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
        epl-vault-consul-acl-bootstrap = pkgs.writeShellScriptBin "epl-vault-consul-acl-bootstrap" ''

            export CONSUL_HTTP_TOKEN=$( sudo cat /run/keys/consul-management-token.txt )

            while :
            do
                consul members | grep alive &>/dev/null && break
                sleep 1
            done

            consul acl policy list | grep '^vault-service:$' &>/dev/null && exit 0

            cat > /tmp/epl-consul-vault-service-policy.hcl<<EOL
            service "vault" {
              policy = "write"
            }

            agent_prefix "" {
              policy = "read"
            }

            session_prefix "" {
              policy = "write"
            }
            EOL

            ${pkgs.consul}/bin/consul acl policy create \
                -name "vault-service" \
                -description "Vault Service Policy" \
                -rules @/tmp/epl-consul-vault-service-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Vault Service Token" \
                -policy-name "vault-service" \
                -secret=$( sudo cat /run/keys/vault-service-consul-acl-token.txt )

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
        pkgs.google-cloud-sdk
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
        epl-acme-vault-policies
        epl-consul-bootstrap
        epl-consul-vrrp-acl
        epl-consul-vrrp-switch
        epl-nomad-acl-bootstrap
        epl-nomad-acl-policies
        epl-nomad-consul-acl-bootstrap
        epl-nomad-vault-policies
        epl-vault-consul-acl-bootstrap
        epl-wait-for-consul
      ];

# NIX REGION static_node_routes START

    networking.interfaces."eth0".ipv4.routes = [

      { address = "0.0.0.0"; prefixLength = 0; via = "10.17.0.1"; }

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
           iifname void ip saddr != { 10.0.0.0/8, 172.21.0.0/16 } ip daddr != { 34.102.96.253/32 } drop comment "Disallow traffic from internet to internal networks";
       }

              '';
            };

# NIX REGION epl_nft_rules_epl-internet-fw END

# NIX REGION firewall START

  networking.hostName = "server-a";
  networking.firewall.allowPing = true;
  networking.firewall.enable = true;
  networking.firewall.checkReversePath = false;
  networking.firewall.trustedInterfaces = [

    "eth0"

    "wg0"

    "vpnGre"

  ];

  networking.firewall.allowedTCPPorts = [ 22 80 443 53 ];

  networking.firewall.allowedUDPPorts = [ 53 51820 ];

# NIX REGION firewall END

   programs.bash.promptInit = ''
     # Provide a nice prompt if the terminal supports it.
     if [ "$TERM" != "dumb" ] || [ -n "$INSIDE_EMACS" ]; then
       PROMPT_COLOR="1;31m"
       ((UID)) && PROMPT_COLOR="1;32m"
       if [ -n "$INSIDE_EMACS" ]; then
         # Emacs term mode doesn't support xterm title escape sequence (\e]0;)
         PS1="\n\[\033[$PROMPT_COLOR\][\u@server-a.dc1.us-west.gcloud-aws-multi-dc:\w]\\$\[\033[0m\] "
       else
         PS1="\n\[\033[$PROMPT_COLOR\][\[\e]0;\u@server-a.dc1.us-west.gcloud-aws-multi-dc: \w\a\]\u@server-a.dc1.us-west.gcloud-aws-multi-dc:\w]\\$\[\033[0m\] "
       fi
       if test "$TERM" = "xterm"; then
         PS1="\[\033]2;server-a.dc1.us-west.gcloud-aws-multi-dc:\u:\w\007\]$PS1"
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
         while ! ${pkgs.consul}/bin/consul kv get epl-l1-plans/server-a
         do
           sleep 7
         done

         ${pkgs.consul}/bin/consul watch \
           -type=key -key=epl-l1-plans/server-a \
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

    imports = [ "${modulesPath}/virtualisation/google-compute-image.nix" ];


  services.zfs.expandOnBoot = "all";

  fileSystems."/" =
    # force because google-compute-config.nix makes it ext4
    pkgs.lib.mkForce
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

  fileSystems."/boot" = {
    # The ZFS image uses a partition labeled ESP whether or not we're
    # booting with EFI.
    device = "/dev/disk/by-label/ESP";
    fsType = "vfat";
  };


    networking.usePredictableInterfaceNames = false;

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
  epl-infra.net. initial-key 257 3 15 "3V0DydHMIEmN+PDJZxVEOHHU1QPVKkIxS6Y2iPyaHuQ=";
  us-west.epl-infra.net. initial-key 257 3 15 "k5nxYOGcYIC66fP80Z7nVeVWmZQGaJvlCQGncoKvLyM=";
  10.in-addr.arpa. initial-key 257 3 15 "EDcnSNMxM8jFNAzwt7sdpd2osA90HOAOw3OobPEe9VM=";
  17.10.in-addr.arpa. initial-key 257 3 15 "azn82AbZk0vfsYtNcKTofCYwfBqjmlWeLEtKqVHxwk8=";
  18.10.in-addr.arpa. initial-key 257 3 15 "Xf1YznaHMJkisSXa6QunCmivRgtYxPylLhpOs3uBtwE=";
  19.10.in-addr.arpa. initial-key 257 3 15 "20BS8hmIKY1gzYl7RHPoNTcB1luZ2Yay88E/PT9l6vY=";
  in-addr.arpa. initial-key 257 3 15 "pFGF/hEQu2aUEkeLvjCqKF7meN68e/b1hSMFmec7+7o=";
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
  type slave;
  file "/run/named/private-epl-infra.net.zone.signed";
  masters { 10.18.0.11; };
  transfer-source 10.17.0.10;
};
zone "10.in-addr.arpa." {
  type slave;
  file "/run/named/private-10.in-addr.arpa.zone.signed";
  masters { 10.18.0.11; };
  transfer-source 10.17.0.10;
};
zone "us-west.epl-infra.net." {
  type slave;
  file "/run/named/private-us-west.epl-infra.net.zone.signed";
  masters { 10.18.0.11; };
  transfer-source 10.17.0.10;
};
zone "17.10.in-addr.arpa." {
  type slave;
  file "/run/named/private-17.10.in-addr.arpa.zone.signed";
  masters { 10.18.0.11; };
  transfer-source 10.17.0.10;
};
zone "18.10.in-addr.arpa." {
  type slave;
  file "/run/named/private-18.10.in-addr.arpa.zone.signed";
  masters { 10.18.0.11; };
  transfer-source 10.17.0.10;
};
zone "19.10.in-addr.arpa." {
  type slave;
  file "/run/named/private-19.10.in-addr.arpa.zone.signed";
  masters { 10.18.0.11; };
  transfer-source 10.17.0.10;
};

};

view internet {
          match-clients { any; };
          recursion no;
zone "epl-infra.net." {
  type slave;
  file "/run/named/public-epl-infra.net.zone.signed";
  masters { 35.235.83.50; };

  allow-query { any; };
};
zone "in-addr.arpa." {
  type slave;
  file "/run/named/public-in-addr.arpa.zone.signed";
  masters { 35.235.83.50; };
};

};

        '';
    };

    virtualisation.docker.daemon.settings = { "registry-mirrors" = [ "https://registry-1.docker.io" "http://epl-docker-registry.service.consul:5000" ]; };
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
          " --web.listen-address=10.17.0.10:9100" +
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
          " --listen_ip=10.17.0.10" +
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
ip address add "172.21.7.10/16" dev "wg0" || true
wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"
ip link set up dev "wg0"

# peers
wg set wg0 peer "DyMxArxtervvFETX/B0o5DFW0hw9FxQAMQ8zVdW/xWs=" allowed-ips "172.21.7.254/32"

wg set wg0 peer "6zcv2TP74mzwko4I4GsoGS87ZnTo0gKfyBpBMggQM1M=" allowed-ips "172.21.7.14/32,10.19.0.0/16" endpoint "34.212.47.215:51820"

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
       }

              '';
            };

# NIX REGION epl_nft_rules_epl-nat END

    boot.kernelModules = [ "gre" ];

# NIX REGION l3_vpn_hop_interface START

  systemd.services.vpn-gre-tunnel = {
    description = "VPN GRE Tunnel - vpnGre";
    after = [ "network-pre.target" ];
    wants = [ "network.target" ];
    before = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];
    path = with pkgs; [ kmod iproute2 ];

    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      Restart = "on-failure";
      RestartSec = "10s";
    };

    script = ''
        ip tunnel add vpnGre mode gre local 10.17.0.10 key 17
        ip addr add 10.17.128.10/17 dev vpnGre


        ip neighbor add 10.17.128.12 lladdr 10.17.0.12 dev vpnGre

        ip link set dev vpnGre up
    '';

    postStop = ''
      ip link del dev vpnGre
    '';
  };

# NIX REGION l3_vpn_hop_interface END

# NIX REGION epl_nft_rules_l3-vpn-hop-address-translation START

            networking.nftables.tables.l3-vpn-hop-address-translation = {
              family = "ip";
              content = ''

        chain PREROUTING {
            type filter hook prerouting priority -300; policy accept;

            ip daddr 10.17.0.12 ip saddr 10.0.0.0/8 ip daddr set 10.17.128.12;
            ip saddr 10.17.128.12 ip saddr set 10.17.0.12;
        }

              '';
            };

# NIX REGION epl_nft_rules_l3-vpn-hop-address-translation END

# NIX REGION frr_ospf_config START

  services.frr.ospf = {
      enable = true;
      config = ''
        !
        router ospf
          ospf router-id 10.17.0.10
          redistribute bgp
          network 10.17.0.0/16 area 10.17.0.0
          area 10.17.0.0 range 10.17.0.0/16 advertise
          area 10.17.0.0 range 0.0.0.0/0 not-advertise
          area 10.17.0.0 authentication message-digest
          neighbor 10.17.0.11
        !
        interface eth0
          ip ospf cost 500
          ip ospf hello-interval 1
          ip ospf dead-interval 3
          ip ospf message-digest-key 12 md5 uhLkFREkhxLxQcIK
          ip ospf authentication message-digest
          ip ospf network non-broadcast
      '';
  };
# NIX REGION frr_ospf_config END

# NIX REGION frr_bfd_config START

  services.frr.bfd = {
      enable = true;
      config = ''
        !
        bfd
          peer 10.17.0.10
            no shutdown
          peer 172.21.7.10
            no shutdown
      '';
  };
# NIX REGION frr_bfd_config END

# NIX REGION frr_bgp_config START

  services.frr.bgp = {
      enable = true;
      config = ''
        !
        router bgp 64529
          bgp router-id 10.17.0.10
          address-family ipv4 unicast
            network 10.17.0.0/16
          exit-address-family
          neighbor 10.17.0.11 remote-as 64529
          neighbor 10.17.0.11 password KhPzEOshEdwIa9tge5vkSIP6DJjkYJm2WZeaafgEa3
          neighbor 10.17.0.11 bfd
          neighbor 172.21.7.12 remote-as 64530
          neighbor 172.21.7.12 password 6CU8kT4y2NR0KHmBogQPwW1d42zXYq9xZa1aN9mlGX
          neighbor 172.21.7.12 bfd
          neighbor 172.21.7.14 remote-as 64531
          neighbor 172.21.7.14 password DkzwtWffsOlzKnmR0WOvMn6QsMlfWZAwGa8wfvm3Ey
          neighbor 172.21.7.14 bfd
          address-family ipv4 unicast
            network 10.17.0.0/16
          exit-address-family
      '';
  };
# NIX REGION frr_bgp_config END

# NIX REGION frr_zebra_config START

  services.frr.zebra = {
      enable = true;
      config = ''
        !
        ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
        !
        ip prefix-list ANY seq 100 permit 0.0.0.0/0
        !
        route-map LANRM permit 100
          match ip address prefix-list LAN
          set src 10.17.0.10
        !
        route-map LANRM permit 110
          match ip address prefix-list ANY
        !
        ip protocol ospf route-map LANRM
        !
        ip protocol bgp route-map LANRM
        !
        ip prefix-list INTERSUBNET seq 100 permit 10.17.0.0/16 le 24
        !
        ip prefix-list INTERSUBNET seq 101 permit 10.17.0.0/16 le 24
        !
        route-map LANRM deny 90
          match ip address prefix-list INTERSUBNET
        !
        interface eth0
          ip address 10.17.0.10/24
      '';
  };
# NIX REGION frr_zebra_config END

# NIX REGION frr_static_routes START

  # You gotta be kidding me... https://github.com/NixOS/nixpkgs/issues/274286
  services.frr.mgmt.enable = true;
  environment.etc."frr/staticd.conf".text = ''
        !
        ip route 10.17.0.0/16 10.17.0.1

        !
        ip route 10.18.0.0/16 10.17.0.1

        !
        ip route 0.0.0.0/0 10.17.0.1

  '';
  systemd.services.staticd.serviceConfig.ExecStart = lib.mkForce "${pkgs.frr}/libexec/frr/staticd -A localhost";
  services.frr.static.enable = true;
# NIX REGION frr_static_routes END

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
      { address = "10.17.0.10"; prefixLength = 24; }

    ];

}
