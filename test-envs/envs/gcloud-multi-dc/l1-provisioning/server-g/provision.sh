#!/bin/sh
umask 0077
mkdir -p /var/lib/epl-l1-prov
mkdir -p /var/log/epl-l1-prov
grep -q vector /etc/group && chgrp vector /var/log/epl-l1-prov || true
chmod 770 /var/log/epl-l1-prov
echo '

CREATE TABLE IF NOT EXISTS l1_provisionings (
  provisioning_id TEXT PRIMARY KEY,
  is_finished INTEGER DEFAULT 0,
  exit_code INTEGER DEFAULT 0,
  time_started TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  time_ended TIMESTAMP DEFAULT 0
);

-- for checking if l1 provisionings exist now
CREATE INDEX IF NOT EXISTS l1_provisionings_is_finished_index ON l1_provisionings (is_finished);

' | sqlite3 /var/lib/epl-l1-prov/provisionings.sqlite
cat > /run/epl-l1-prov <<'ThisIsEplProvL1Script'
#!/bin/sh
set -e
function trap_exit {
    EXIT_CODE=$?

    echo "
      UPDATE l1_provisionings
      SET exit_code = $EXIT_CODE,
          time_ended = CURRENT_TIMESTAMP,
          is_finished = 1
      WHERE provisioning_id = L1_EPL_PROVISIONING_ID
    " | sqlite3 /var/lib/epl-l1-prov/provisionings.sqlite
}
trap trap_exit ERR
umask 0077
mkdir -p /etc/nixos
pushd /etc/nixos
git config --global init.defaultBranch master
git config --global user.name 'EPL L1 provisioner'
git config --global user.email 'epl@example.com'
git init
cat > /etc/nixos/configuration.nix <<'LilBoiPeepLikesBenzTruck'

let
  pkgs = import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/4ecab3273592f27479a583fb6d975d4aba3486fe.tar.gz")) {};
  lib = pkgs.lib;
in

{ modulesPath, config, ... }:
{

    networking.firewall.enable = false;
    nix.settings = {
      experimental-features = [ "nix-command" "flakes" ];
      substituters = [
        "https://cache.nixos.org/"
      ];
      trusted-public-keys = [
        "epl-nix-cache:TKuTt1vtCbpAtW6YmzN2doZvTsoPunGMpXZzh2nAu2Q="
      ];

    };

    virtualisation.docker.enable = true;
    time.timeZone = "UTC";

    fileSystems."/" =
      { device = "/dev/disk/by-label/nixos";
        fsType = "ext4";
      };

    users.users.root.hashedPassword = "!";
    users.users.root.openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIIWH2F//Ff/MIYhKzwx+CYP3wJ5h9/h+VMQkk/uyKfo+ epl-root-ssh-key
"
      "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC3AkuDzzPrMaDav0kN7PIoaBU1Vtw1TfkHxWzPMrleocCltYl8TljwCqEJtmizx5DGKbXFQg31mRVswzuAq2vP2RFdPHQxfl5nJnWsQkelvpPO/Q3LUdtrm19zAgbbDL+AtIg3/lif6/2qNiWCSTfaUpjM7WOPszBNmMRGz/UBZTYc7COTt+I3lK8f6sBn5YyD796LBw6tsNpqfqF9NTAsLT8/PqrXeTpdxFe375gMxeIpNWeE5exMGJKgqnZCcOMOoKMJy61+wdEAYzDFNgIX7ZFvpBYQPf/rTs7LWgtyTSw3fqvMDnfwAf7oIF8rZRwYdVnqTGCWA2h3f4lOf6BERIPkKEK7/DGjmekKnXJrRiLSfcgRjri3VuGBxrJ+Va/Dn6e7o7CdzdJ+fkw7KxTFKuf17Z2r3ZFi1xOduIxXW8/QY6zhq2A11e+HsMe/oaBh3bRcpdMFmW5mqQjGm05xvxArSCAARBKkHjywGs6mRLN2PjNPYdzlI2J8nF6bmSk= henlo"

    ];
    services.sshd.enable = true;
    services.openssh.permitRootLogin = lib.mkDefault "yes";
    services.getty.autologinUser = lib.mkDefault "root";

    swapDevices = [ ];

    system.stateVersion = "22.05";

    environment.sessionVariables = {
      NOMAD_ADDR = "https://nomad-servers.service.consul:4646";
      VAULT_ADDR = "https://vault.service.consul:8200";
    };

    security.pki.certificates = [
      ''-----BEGIN CERTIFICATE-----
MIIB0zCCAXmgAwIBAgIUbiD8I8IyzT783hwO1XBOopPvP8cwCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAxMGQ0EgS2V5MB4XDTIzMTIxMDA4NDAwMFoXDTQwMTIwNTA4NDAw
MFowADBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABBNBCaU48kK+3pcf3GJ0kzYj
yzVGTUG888B87V/C+oEdR6eghscScFi18DeZ+inogvDeK7IUesUsJact7rMMXzmj
gb8wgbwwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBSpna8r2/KxjqtMjNVYNjob4DJo
djAfBgNVHSMEGDAWgBS8xb/JxxGG4yHM0LOG3F6dddmgYTA9BgNVHREBAf8EMzAx
gg1lcGwtaW5mcmEubmV0gg8qLmVwbC1pbmZyYS5uZXSCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNIADBFAiEAx5ug99RrnV//6EsY/RV9GqxY2j/rKPD8Nfj3
+ECg3IoCICwhKh86NzitxL9Av43JMy1DVnjg5OUCKSHgaVtKCvHc
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBZTCCAQqgAwIBAgIUA1Rf+QzWDRp7TOCcm77pJrufaX0wCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjMxMjEwMDg0MDAwWhcNNDAxMjA1MDg0MDAw
WjAQMQ4wDAYDVQQDEwVub21hZDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABMEG
BMbE77RYrk44Sx6N0iRvrDemC60NFF5mSOmqd5ISiL9HnmxSesSuLUD2CimRonBa
b3CwHUXc19fCUIUvcZmjQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBSIaEmGc1TZqroZHSDON2TLjgFazDAKBggqhkjOPQQDAgNJ
ADBGAiEA96Kbui7gZAtmLFWC25/SLeYWtLmhHhiX/SX8bviWtTMCIQDv0h32ruvR
d8U8yrMaNQ7XFbDBnHeoKbiIg7t/kww1kQ==
-----END CERTIFICATE-----
''
      ''-----BEGIN CERTIFICATE-----
MIIBZTCCAQqgAwIBAgIUEG/FUYCY2e7t5RpR2Fiob2DRYWgwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjMxMjEwMDg0MDAwWhcNNDAxMjA1MDg0MDAw
WjAQMQ4wDAYDVQQDEwV2YXVsdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABO0S
uQyFy64bbzPnt8SXlEMMG5F7w6bK3c+7WbhDlmxdtL7G5T4F0jQZa9tYMzZWdPJy
bcFk/D0d+njRx2dfEFujQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBQro1VtoctSvekfgXtFOeiOXNn21zAKBggqhkjOPQQDAgNJ
ADBGAiEAwJ8BoWndiIp6UTQg10YI3dUj0OBMlk3EbODNSaBi894CIQDvMfK6uu0c
vtgvVueNMmbOlTGoFOi0xZjX2tK3KVmbRg==
-----END CERTIFICATE-----
''
    ];

    environment.systemPackages =
      let
        epl-consul-bootstrap = pkgs.writeShellScriptBin "epl-consul-bootstrap" ''

            export CONSUL_HTTP_TOKEN=$( cat /run/keys/consul-management-token.txt )

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

            # inter DC routing, allow every node to access routes
            key_prefix "epl-interdc-routes/" {
                policy = "list"
            }
            EOL

            ${pkgs.consul}/bin/consul acl policy create \
                -name "agent-policy" \
                -description "Agent Token Policy" \
                -rules @/tmp/epl-consul-agent-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Agent Token" \
                -policy-name "agent-policy" \
                -secret=$( cat /run/keys/consul-agent-token.txt )

            ${pkgs.consul}/bin/consul acl policy create \
                -name "default-token" \
                -description "Default Token Policy" \
                -rules @/tmp/epl-consul-default-policy.hcl

            ${pkgs.consul}/bin/consul acl token create \
                -description "Default Token" \
                -policy-name "default-token" \
                -secret=$( cat /run/keys/consul-default-token.txt )

        '';
        epl-consul-vrrp-acl = pkgs.writeShellScriptBin "epl-consul-vrrp-acl" ''

            export CONSUL_HTTP_TOKEN=$( cat /run/keys/consul-management-token.txt )

            while :
            do
                consul members | grep alive &>/dev/null && break
                sleep 1
            done

            if ! consul acl policy list | grep '^vrrp-policy-dc1:$'
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
                    -secret=$( cat /run/keys/consul-vrrp-token-dc1.txt )
            fi

            # after policy provisioning key is no longer needed
            rm -f /run/keys/consul-vrrp-token-dc1.txt


            if ! consul acl policy list | grep '^vrrp-policy-dc2:$'
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
                    -secret=$( cat /run/keys/consul-vrrp-token-dc2.txt )
            fi

            # after policy provisioning key is no longer needed
            rm -f /run/keys/consul-vrrp-token-dc2.txt


            if ! consul acl policy list | grep '^vrrp-policy-dc3:$'
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
                    -secret=$( cat /run/keys/consul-vrrp-token-dc3.txt )
            fi

            # after policy provisioning key is no longer needed
            rm -f /run/keys/consul-vrrp-token-dc3.txt


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

            sleep 3

            NOMAD_BOOTSTRAP_TOKEN=$( nomad acl bootstrap | grep 'Secret ID' | sed -E 's/^.*= //g' )
            # first just export token to the outside
            echo $NOMAD_BOOTSTRAP_TOKEN
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

            nomad namespace apply -description "System jobs" system
            nomad namespace apply -description "Eden platform user jobs" epl


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
        epl-process-route-data = pkgs.writeShellScriptBin "epl-process-route-data" ''

# all files created are hidden
umask 0077

PREFIX=/run/current-system/sw/bin
$PREFIX/cat /dev/stdin > /run/epl-routes-tmp

# delete old routes if exist
if [ -f /run/epl-routes ];
then
    OLD_SCRIPT=$( $PREFIX/cat /run/epl-routes | $PREFIX/jq -r '.Value' | $PREFIX/base64 -d )
    # delete old routes
    DELETE_BLOCK=$( $PREFIX/echo "$OLD_SCRIPT" | $PREFIX/sed -n '/ROUTES DELETE/,/FINISH/p' )
    $PREFIX/echo "export PATH=/run/current-system/sw/bin/:$PATH; $DELETE_BLOCK" | /bin/sh
    $PREFIX/echo old routes deleted
    $PREFIX/echo "$DELETE_BLOCK"
fi

NEW_SCRIPT=$( $PREFIX/cat /run/epl-routes-tmp | $PREFIX/jq -r '.Value' | $PREFIX/base64 -d )
NEW_ADD_BLOCK=$( $PREFIX/echo "$OLD_SCRIPT" | $PREFIX/sed -n '/ROUTES CREATE/,/ROUTES DELETE/p' )
# add new routes
$PREFIX/echo "export PATH=/run/current-system/sw/bin/:$PATH; $NEW_ADD_BLOCK" | /bin/sh

# set new file in place for old route deletion
$PREFIX/mv -f /run/epl-routes-tmp /run/epl-routes

$PREFIX/echo routes were changed
$PREFIX/echo "$NEW_ADD_BLOCK"


        '';

      in
      [
        pkgs.bmon
        pkgs.cadvisor
        pkgs.curl
        pkgs.dig
        pkgs.htop
        pkgs.iftop
        pkgs.iotop
        pkgs.jq
        pkgs.moreutils
        pkgs.natscli
        pkgs.nftables
        pkgs.nomad
        pkgs.postgresql
        pkgs.prometheus-node-exporter
        pkgs.python3
        pkgs.sqlite
        pkgs.tmux
        pkgs.vector
        pkgs.vim
        pkgs.wget
        epl-consul-bootstrap
        epl-consul-vrrp-acl
        epl-nomad-acl-bootstrap
        epl-nomad-acl-policies
        epl-nomad-vault-policies
        epl-process-route-data
      ];

    imports = [ "${modulesPath}/virtualisation/google-compute-image.nix" ];

    system.activationScripts.provisionSecretsVolumes.text = ''
        mkdir -m 700 -p /run/sec_volumes/ssl_certs
        cp /run/keys/public_tls_key.pem /run/sec_volumes/ssl_certs/
        cp /run/keys/public_tls_cert.pem /run/sec_volumes/ssl_certs/
    '';

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
  epl-infra.net. initial-key 257 3 15 "RdMqIAn9Igdoj1/WuaS1Ax5GrmWBgj0BYP5k+k/HarE=";
  us-west.epl-infra.net. initial-key 257 3 15 "LEXLv/EILvpz18YApP3LSwFg5Btgo2NLhzf1eozDRic=";
  10.in-addr.arpa. initial-key 257 3 15 "bl8iC8SU8SfruHIeH1Ae01G3Q32Itfzwpdyr8PO4qdw=";
  17.10.in-addr.arpa. initial-key 257 3 15 "drvbe3R6GQMoQxeQDMOyNilYpl7s/o/khjJXzlyfppQ=";
  18.10.in-addr.arpa. initial-key 257 3 15 "sr71+iR3W4xAD7UfCjM2Fgo8hpynY/YhzAd4nKdBw/E=";
  19.10.in-addr.arpa. initial-key 257 3 15 "+SVbqQWacRqV6CHGgPyK8NJ98udi5fMrPsUz2MXT9TY=";
  in-addr.arpa. initial-key 257 3 15 "HGNILHENJtwtEUPeRwTJg5pXPxg8fWiW5kcbuEtfkh8=";
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
          match-clients { 10.0.0.0/8; 172.21.7.0/24; localhost; };
          zone "consul." IN {
              type forward;
              forward only;
              forwarders { 127.0.0.1 port 8600; };
          };

          zone "us-west.epl-infra.net" IN {
              type forward;
              forward only;
              forwarders {
                10.18.0.11 port 53;
                10.18.0.10 port 53;
                10.19.0.10 port 53;
              };

          };

          zone "17.10.in-addr.arpa." IN {
              type forward;
              forward only;
              forwarders {
                10.18.0.11 port 53;
                10.18.0.10 port 53;
                10.19.0.10 port 53;
              };

          };

          zone "18.10.in-addr.arpa." IN {
              type forward;
              forward only;
              forwarders {
                10.18.0.11 port 53;
                10.18.0.10 port 53;
                10.19.0.10 port 53;
              };

          };

          zone "19.10.in-addr.arpa." IN {
              type forward;
              forward only;
              forwarders {
                10.18.0.11 port 53;
                10.18.0.10 port 53;
                10.19.0.10 port 53;
              };

          };


          zone "epl-infra.net." IN {
              type forward;
              forward only;
              forwarders {
                10.18.0.11 port 53;
                10.18.0.10 port 53;
                10.19.0.10 port 53;
              };

          };

          zone "10.in-addr.arpa." IN {
              type forward;
              forward only;
              forwarders {
                10.18.0.11 port 53;
                10.18.0.10 port 53;
                10.19.0.10 port 53;
              };

          };


};

        '';
    };

    system.activationScripts.bindActivation.text =

    ''

function update_dns_file() {
  CS=/run/current-system/sw/bin/
  $CS/cp $SOURCE_FILE $TARGET_FILE
  # This will work only until exhausting
  # last number of 32 bit space for 42 years
  # we cannot provision more often than every
  # minute for different serials. We win time by subtracting
  # 23 years - year of when this line was written
  DATE=$( $CS/date +%y%m%d%H%M -d '23 years ago' )
  $CS/sed -i "s/SERIAL_TO_REPLACE/$DATE/g" $TARGET_FILE
  echo ";$SOURCE_FILE" >> $TARGET_FILE
}

function maybe_update_dns_file() {
   CS=/run/current-system/sw/bin/
   SOURCE_FILE=$1
   TARGET_FILE=$2
   if [ ! -f $TARGET_FILE ]
   then
      echo zone target $TARGET_FILE doesnt exist, installing $SOURCE_FILE
      update_dns_file $SOURCE_FILE $TARGET_FILE
      return 0
   fi
   if ! $CS/grep ";$SOURCE_FILE" $TARGET_FILE
   then
      echo Source file $SOURCE_FILE changed, installing to $TARGET_FILE
      update_dns_file $SOURCE_FILE $TARGET_FILE
      return 0
   fi
}


      # prepare for dnssec
      mkdir -p /run/dnsseckeys
      chown named:named /run/dnsseckeys
      chmod 700 /run/dnsseckeys


      # bind zones directory to allow modifying keys



    '';

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
        ExecStart = "${pkgs.nomad}/bin/nomad agent -config=/run/keys/nomad-config.hcl";
        ExecReload = "/bin/kill -HUP $MAINPID";
        KillMode = "process";
        KillSignal = "SIGINT";
        LimitNOFILE = "infinity";
        LimitNPROC = "infinity";
        Restart = "always";
        RestartSec = "120";
        TasksMax = "infinity";
      };

      enable = true;
    };

    system.activationScripts.provisionVolumesDir.text = ''
      mkdir -m 700 -p /srv/volumes

    '';


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

    system.activationScripts.registerNodeExporterToConsul.text = ''
      # wait for consul to be up and running if restarted for up to 10 seconds
      for I in $(seq 1 10); do netstat -tulpn | grep 127.0.0.1:8500 && break || true; sleep 1; done

      export CONSUL_HTTP_TOKEN=$(cat /run/keys/consul-agent-token.txt)
      for I in $(seq 1 5); do
        ${pkgs.consul}/bin/consul services register /run/keys/epl-node-exporter-service.hcl && break || true
        # try a few times if consul is down
        sleep 1
      done
    '';

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
          " --web.listen-address=10.19.0.12:9100" +
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

    system.activationScripts.registerCadvisorToConsul.text = ''
      # wait for consul to be up and running if restarted for up to 10 seconds
      for I in $(seq 1 10); do netstat -tulpn | grep 127.0.0.1:8500 && break || true; sleep 1; done

      export CONSUL_HTTP_TOKEN=$(cat /run/keys/consul-agent-token.txt)
      for I in $(seq 1 5); do
        ${pkgs.consul}/bin/consul services register /run/keys/epl-cadvisor-service.hcl && break || true
        # try a few times if consul is down
        sleep 1
      done
    '';

    systemd.services.cadvisor = {
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        User = "root";
        Group = "root";
        Type = "simple";
        ExecStart = "${pkgs.cadvisor}/bin/cadvisor" +
          " --listen_ip=10.19.0.12" +
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

    system.activationScripts.registerVectorToConsul.text = ''
      # wait for consul to be up and running if restarted for up to 10 seconds
      for I in $(seq 1 10); do netstat -tulpn | grep 127.0.0.1:8500 && break || true; sleep 1; done

      export CONSUL_HTTP_TOKEN=$(cat /run/keys/consul-agent-token.txt)
      for I in $(seq 1 5); do
        ${pkgs.consul}/bin/consul services register /run/keys/epl-vector-service.hcl && break || true
        # try a few times if consul is down
        sleep 1
      done
    '';


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

    system.activationScripts.vectorDirectory.text = ''
      mkdir --mode 700 -p /var/lib/vector
      chown vector:vector /var/lib/vector
    '';

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

    system.activationScripts.addWireguardRoutes.text = ''
      /run/current-system/sw/bin/ip route del 10.0.0.0/8 || true
      /run/current-system/sw/bin/ip route add 10.0.0.0/8 src 10.19.0.12 scope global via 10.19.0.7
    '';

    systemd.services.epl-route-watcher = {
      wantedBy = [ "multi-user.target" ];
      requires = [ "network-online.target" ];
      after = [ "network-online.target" "consul.service" ];

      serviceConfig = {
        User = "root";
        Group = "root";
        Type = "simple";
        ExecStart = "${pkgs.consul}/bin/consul watch -type=key -key=epl-interdc-routes/dc3 /run/current-system/sw/bin/epl-process-route-data";
        Restart = "always";
        RestartSec = "10";
        TasksMax = "infinity";
      };

      enable = true;
    };

}

LilBoiPeepLikesBenzTruck
echo L1_EPL_PROVISIONING_ID > /etc/nixos/epl-prov-id
chown root:root /etc/nixos/configuration.nix
chmod 0600 /etc/nixos/configuration.nix
git add .
git commit -am 'Update L1_EPL_PROVISIONING_ID' || true
popd
nixos-rebuild switch || L1_PROVISIONING_TOLERATE_REBUILD_FAIL
rm -f /run/tmpsec-*
mkdir -p /run/keys
chmod 755 /run/keys
TMP_SECRET_PATH=/run/tmpsec-$RANDOM
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
{
  "acl": {
    "default_policy": "deny",
    "enable_token_persistence": true,
    "enabled": true,
    "tokens": {
      "agent": "edaf13c7-6efe-484d-a19a-639fb89a4186",
      "default": "8e85c699-4992-4373-ad93-eaeb34dcfcb2"
    }
  },
  "addresses": {
    "dns": "127.0.0.1",
    "grpc": "127.0.0.1",
    "http": "127.0.0.1",
    "https": "10.19.0.12"
  },
  "advertise_addr": "10.19.0.12",
  "advertise_addr_wan": "10.19.0.12",
  "auto_encrypt": {
    "tls": true
  },
  "bind_addr": "10.19.0.12",
  "client_addr": "127.0.0.1",
  "data_dir": "/var/lib/consul",
  "datacenter": "us-west",
  "disable_update_check": false,
  "domain": "consul",
  "enable_local_script_checks": false,
  "enable_script_checks": false,
  "encrypt": "aOpV+IZzfh3euz5cfUD1nALYcKOquepYYE663TCrFdA=",
  "encrypt_verify_incoming": true,
  "encrypt_verify_outgoing": true,
  "log_level": "INFO",
  "log_rotate_bytes": 0,
  "log_rotate_duration": "24h",
  "log_rotate_max_files": 0,
  "node_name": "server-g",
  "performance": {
    "leave_drain_time": "5s",
    "raft_multiplier": 1,
    "rpc_hold_timeout": "7s"
  },
  "ports": {
    "dns": 8600,
    "grpc": -1,
    "http": 8500,
    "https": 8501,
    "serf_lan": 8301,
    "serf_wan": 8302,
    "server": 8300
  },
  "raft_protocol": 3,
  "retry_interval": "30s",
  "retry_join": [
    "10.17.0.10",
    "10.18.0.10",
    "10.19.0.10"
  ],
  "retry_max": 0,
  "server": false,
  "tls": {
    "defaults": {
      "ca_file": "/run/keys/consul-tls-ca-cert.pem",
      "tls_min_version": "TLSv1_2",
      "verify_incoming": false,
      "verify_outgoing": true
    },
    "https": {
      "verify_incoming": false
    },
    "internal_rpc": {
      "verify_incoming": false,
      "verify_server_hostname": true
    }
  },
  "translate_wan_addrs": false,
  "ui_config": {
    "enabled": true
  }
}
LilBoiPeepLikesBenzTruck
chown consul $TMP_SECRET_PATH
chgrp consul $TMP_SECRET_PATH
unset NEEDS_MOVE
cmp --silent $TMP_SECRET_PATH /run/keys/consul-config.json || NEEDS_MOVE=true
[ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/consul-config.json')" ] || NEEDS_MOVE=true
[ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/consul-config.json || rm -f $TMP_SECRET_PATH
TMP_SECRET_PATH=/run/tmpsec-$RANDOM
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'


name = "server-g"
region = "us-west"
datacenter = "dc3"

enable_debug = false
disable_update_check = false


bind_addr = "10.19.0.12"
advertise {
    http = "10.19.0.12:4646"
    rpc = "10.19.0.12:4647"
    serf = "10.19.0.12:4648"
}

ports {
    http = 4646
    rpc = 4647
    serf = 4648
}

consul {
    # The address to the Consul agent.
    address = "127.0.0.1:8500"
    ssl = false
    ca_file = ""
    cert_file = ""
    key_file = ""
    token = "29fa2b93-6aaf-4ca1-935e-b102bf331d9b"
    # The service name to register the server and client with Consul.
    server_service_name = "nomad-servers"
    client_service_name = "nomad-clients"
    tags = {
    }

    # Enables automatically registering the services.
    auto_advertise = true

    # Enabling the server and client to bootstrap using Consul.
    server_auto_join = true
    client_auto_join = true
}

data_dir = "/var/nomad"

log_level = "INFO"
enable_syslog = true

leave_on_terminate = true
leave_on_interrupt = false


tls {
    http = true
    rpc = true
    ca_file = "/run/keys/nomad-ca.crt"
    cert_file = "/run/keys/nomad-client.crt"
    key_file = "/run/keys/nomad-client.key"
    rpc_upgrade_mode = false
    verify_server_hostname = "true"
    verify_https_client = "false"
}


vault {
    enabled = true
    address = "https://vault.service.consul:8200"
    allow_unauthenticated = false
    create_from_role = "nomad-cluster"
    task_token_ttl = ""
    ca_file = "/run/keys/vault-ca.crt"
    ca_path = ""
    cert_file = ""
    key_file = ""
    tls_server_name = ""
    tls_skip_verify = false
    namespace = ""

}

client {
  enabled = true

  node_class = ""
  no_host_uuid = false

  max_kill_timeout = "3600s"

  network_speed = 0
  cpu_total_compute = 0

  gc_interval = "1m"
  gc_disk_usage_threshold = 80
  gc_inode_usage_threshold = 70
  gc_parallel_destroys = 2

  reserved {
    cpu = 0
    memory = 0
    disk = 0
  }

  meta = {
    "private_ip" = "10.19.0.12"
  }

  host_volume "ssl_certs" {
    path = "/run/sec_volumes/ssl_certs"
    read_only = true
  }

  host_network "lan" {
    cidr = "10.0.0.0/8"
  }

}

acl {
    enabled = true
    token_ttl = "30s"
    policy_ttl = "30s"
    replication_token = ""
}

telemetry {
    disable_hostname = "false"
    collection_interval = "1s"
    use_node_name = "false"
    publish_allocation_metrics = "true"
    publish_node_metrics = "true"
    filter_default = "true"
    prefix_filter = []
    disable_dispatched_job_summary_metrics = "false"
    statsite_address = ""
    statsd_address = ""
    datadog_address = ""
    datadog_tags = []
    prometheus_metrics = "true"
    circonus_api_token = ""
    circonus_api_app = "nomad"
    circonus_api_url = "https://api.circonus.com/v2"
    circonus_submission_interval = "10s"
    circonus_submission_url = ""
    circonus_check_id = ""
    circonus_check_force_metric_activation = "false"
    circonus_check_instance_id = ""
    circonus_check_search_tag = ""
    circonus_check_display_name = ""
    circonus_check_tags = ""
    circonus_broker_id = ""
    circonus_broker_select_tag = ""
}

plugin "docker" {
    config {
        extra_labels = ["*"]
        logging {
            type = "json-file"
            config {
                max-file = 3
                max-size = "30m"
            }
        }
    }
}
LilBoiPeepLikesBenzTruck
unset NEEDS_MOVE
cmp --silent $TMP_SECRET_PATH /run/keys/nomad-config.hcl || NEEDS_MOVE=true
[ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/nomad-config.hcl')" ] || NEEDS_MOVE=true
[ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/nomad-config.hcl || rm -f $TMP_SECRET_PATH
TMP_SECRET_PATH=/run/tmpsec-$RANDOM
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'

service {
  name = "epl-node-exporter"
  id   = "epl-node-exporter"
  port = 9100
  tags = ["epl-mon-default"]

  meta = {
    metrics_path = "/metrics"
  }

  tagged_addresses = {
    lan = {
      address = "10.19.0.12"
      port    = 9100
    }
  }

  checks = [
    {
        id       = "home"
        name     = "/"
        http     = "http://10.19.0.12:9100/"
        interval = "15s"
    },
  ]
}
LilBoiPeepLikesBenzTruck
unset NEEDS_MOVE
cmp --silent $TMP_SECRET_PATH /run/keys/epl-node-exporter-service.hcl || NEEDS_MOVE=true
[ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/epl-node-exporter-service.hcl')" ] || NEEDS_MOVE=true
[ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/epl-node-exporter-service.hcl || rm -f $TMP_SECRET_PATH
TMP_SECRET_PATH=/run/tmpsec-$RANDOM
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'

service {
  name = "epl-cadvisor"
  id   = "epl-cadvisor"
  port = 9280
  tags = ["epl-mon-default"]

  tagged_addresses = {
    lan = {
      address = "10.19.0.12"
      port    = 9280
    }
  }

  meta = {
    metrics_path = "/metrics"
  }

  checks = [
    {
        id       = "healthcheck"
        name     = "/healthz"
        http     = "http://10.19.0.12:9280/"
        interval = "15s"
    },
  ]
}
LilBoiPeepLikesBenzTruck
unset NEEDS_MOVE
cmp --silent $TMP_SECRET_PATH /run/keys/epl-cadvisor-service.hcl || NEEDS_MOVE=true
[ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/epl-cadvisor-service.hcl')" ] || NEEDS_MOVE=true
[ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/epl-cadvisor-service.hcl || rm -f $TMP_SECRET_PATH
TMP_SECRET_PATH=/run/tmpsec-$RANDOM
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'

service {
  name = "epl-vector"
  id   = "epl-vector"
  port = 9281
  tags = ["epl-mon-default"]

  tagged_addresses = {
    lan = {
      address = "10.19.0.12"
      port    = 9281
    }
  }

  meta = {
    metrics_path = "/metrics"
  }

  checks = [
    {
        id       = "home"
        tcp      = "10.19.0.12:9281"
        interval = "15s"
    },
  ]
}
LilBoiPeepLikesBenzTruck
unset NEEDS_MOVE
cmp --silent $TMP_SECRET_PATH /run/keys/epl-vector-service.hcl || NEEDS_MOVE=true
[ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/epl-vector-service.hcl')" ] || NEEDS_MOVE=true
[ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/epl-vector-service.hcl || rm -f $TMP_SECRET_PATH
TMP_SECRET_PATH=/run/tmpsec-$RANDOM
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'

# ----------------------------------
# prometheus metrics
# ----------------------------------
[sources.internal_metrics]
type = "internal_metrics"
scrape_interval_secs = 2

[sinks.prometheus_exporter_sink]
inputs = ["internal_metrics"]
type = "prometheus_exporter"
address = "10.19.0.12:9281"

# ---------------------------------------------------------
# journald source
# ---------------------------------------------------------
[sources.journald]
type = "journald"
current_boot_only = true
exclude_units = [
  "dbus.service",
  "init.scope",
  "systemd-journald.service",
  "systemd-udevd.service",
]

# ----------------------------------
# docker source
# ----------------------------------
[sources.docker]
type = "docker_logs"

# ----------------------------------
# l1 provisioning sources
# ----------------------------------
[sources.l1_provisioning_logs]
type = "file"
include = [ "/var/log/epl-l1-prov/*.log" ]
read_from = "beginning"
remove_after_secs = 86400

[transforms.l1_provisioning_logs_extra]
type = "remap"
inputs = ["l1_provisioning_logs"]
source = """
segments = split!(.file, "/")
fname = split!(get!(segments, [-1]), ".")
.filename = get!(segments, [-1])
.provisioning_id = get!(fname, [-2])
"""

# ----------------------------------
# l2 provisioning sources
# ----------------------------------
[sources.l2_provisioning_logs]
type = "file"
include = [ "/var/log/epl-l2-prov/*/*.log" ]
read_from = "beginning"

[transforms.l2_provisioning_logs_extra]
type = "remap"
inputs = ["l2_provisioning_logs"]
source = """
segments = split!(.file, "/")
.filename = get!(segments, [-1])
.provisioning_id = get!(segments, [-2])
"""

# ----------------------------------
# loki journald sink
# ----------------------------------
[sinks.loki_journald]
type = "loki"
inputs = [ "journald" ]
endpoint = "http://epl-loki-main-loki-writer.service.consul:3010"
healthcheck = false
encoding.codec = "raw_message"
request.retry_max_duration_secs = 60

[sinks.loki_journald.buffer]
type = "disk"
max_size = 268435488
when_full = "block"

[sinks.loki_journald.labels]
source_type = "journald"
host = "server-g.us-west.epl-infra.net"
systemd_unit = "{{ _SYSTEMD_UNIT }}"

# ----------------------------------
# loki l1 provisioning sink
# ----------------------------------
[sinks.loki_l1_provisioning]
type = "loki"
inputs = [ "l1_provisioning_logs_extra" ]
endpoint = "http://epl-loki-main-loki-writer.service.consul:3010"
healthcheck = false
encoding.codec = "raw_message"
request.retry_max_duration_secs = 60

[sinks.loki_l1_provisioning.buffer]
type = "disk"
max_size = 1073741824
when_full = "block"

[sinks.loki_l1_provisioning.labels]
source_type = "l1_provisioning"
host = "server-g.us-west.epl-infra.net"
file = "{{ filename }}"
provisioning_id = "{{ provisioning_id }}"

# ----------------------------------
# loki l2 provisioning sink
# ----------------------------------
[sinks.loki_l2_provisioning]
type = "loki"
inputs = [ "l2_provisioning_logs_extra" ]
endpoint = "http://epl-loki-main-loki-writer.service.consul:3010"
healthcheck = false
encoding.codec = "raw_message"
request.retry_max_duration_secs = 60

[sinks.loki_l2_provisioning.buffer]
type = "disk"
max_size = 1073741824
when_full = "block"

[sinks.loki_l2_provisioning.labels]
source_type = "l2_provisioning"
host = "server-g.us-west.epl-infra.net"
file = "{{ filename }}"
provisioning_id = "{{ provisioning_id }}"

# ----------------------------------
# loki nomad docker sink for main
# ----------------------------------
[transforms.loki_nomad_docker_router]
type = "route"
inputs = [ "docker" ]
[transforms.loki_nomad_docker_router.route]
main = '.label.epl_loki_cluster == "main"'


# ----------------------------------
# loki nomad docker sink for main
# ----------------------------------
[sinks.loki_nomad_docker_main]
type = "loki"
inputs = [ "loki_nomad_docker_router.main", "loki_nomad_docker_router._unmatched" ]
endpoint = "http://epl-loki-main-loki-writer.service.consul:3010"
healthcheck = false
encoding.codec = "raw_message"
request.retry_max_duration_secs = 60

[sinks.loki_nomad_docker_main.buffer]
type = "disk"
max_size = 1073741824
when_full = "block"

[sinks.loki_nomad_docker_main.labels]
source_type = "nomad_docker"
host = "server-g.us-west.epl-infra.net"
namespace = "{{ label.\"com.hashicorp.nomad.namespace\" }}"
job_name = "{{ label.\"com.hashicorp.nomad.job_name\" }}"
task_group_name = "{{ label.\"com.hashicorp.nomad.task_group_name\" }}"
task_name = "{{ label.\"com.hashicorp.nomad.task_name\" }}"
alloc_id = "{{ label.\"com.hashicorp.nomad.alloc_id\" }}"
image = "{{ image }}"
LilBoiPeepLikesBenzTruck
chown vector $TMP_SECRET_PATH
chgrp vector $TMP_SECRET_PATH
unset NEEDS_MOVE
cmp --silent $TMP_SECRET_PATH /run/keys/vector.toml || NEEDS_MOVE=true
[ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/vector.toml')" ] || NEEDS_MOVE=true
[ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/vector.toml || rm -f $TMP_SECRET_PATH
TMP_SECRET_PATH=/run/tmpsec-$RANDOM
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIHZOhG7uAnYEQZuvRVkOdCHFdxJAkxjBOQX/sbbY2bwIoAoGCCqGSM49
AwEHoUQDQgAEE0EJpTjyQr7elx/cYnSTNiPLNUZNQbzzwHztX8L6gR1Hp6CGxxJw
WLXwN5n6KeiC8N4rshR6xSwlpy3uswxfOQ==
-----END EC PRIVATE KEY-----
LilBoiPeepLikesBenzTruck
unset NEEDS_MOVE
cmp --silent $TMP_SECRET_PATH /run/keys/public_tls_key.pem || NEEDS_MOVE=true
[ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/public_tls_key.pem')" ] || NEEDS_MOVE=true
[ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/public_tls_key.pem || rm -f $TMP_SECRET_PATH
TMP_SECRET_PATH=/run/tmpsec-$RANDOM
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIB0zCCAXmgAwIBAgIUbiD8I8IyzT783hwO1XBOopPvP8cwCgYIKoZIzj0EAwIw
ETEPMA0GA1UEAxMGQ0EgS2V5MB4XDTIzMTIxMDA4NDAwMFoXDTQwMTIwNTA4NDAw
MFowADBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABBNBCaU48kK+3pcf3GJ0kzYj
yzVGTUG888B87V/C+oEdR6eghscScFi18DeZ+inogvDeK7IUesUsJact7rMMXzmj
gb8wgbwwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAMBgNVHRMBAf8EAjAAMB0GA1UdDgQWBBSpna8r2/KxjqtMjNVYNjob4DJo
djAfBgNVHSMEGDAWgBS8xb/JxxGG4yHM0LOG3F6dddmgYTA9BgNVHREBAf8EMzAx
gg1lcGwtaW5mcmEubmV0gg8qLmVwbC1pbmZyYS5uZXSCCWxvY2FsaG9zdIcEfwAA
ATAKBggqhkjOPQQDAgNIADBFAiEAx5ug99RrnV//6EsY/RV9GqxY2j/rKPD8Nfj3
+ECg3IoCICwhKh86NzitxL9Av43JMy1DVnjg5OUCKSHgaVtKCvHc
-----END CERTIFICATE-----
LilBoiPeepLikesBenzTruck
unset NEEDS_MOVE
cmp --silent $TMP_SECRET_PATH /run/keys/public_tls_cert.pem || NEEDS_MOVE=true
[ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/public_tls_cert.pem')" ] || NEEDS_MOVE=true
[ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/public_tls_cert.pem || rm -f $TMP_SECRET_PATH
TMP_SECRET_PATH=/run/tmpsec-$RANDOM
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIC7DCCApOgAwIBAgIQTG+xxul0oTyDrwn76tA23zAKBggqhkjOPQQDAjCBuTEL
MAkGA1UEBhMCVVMxCzAJBgNVBAgTAkNBMRYwFAYDVQQHEw1TYW4gRnJhbmNpc2Nv
MRowGAYDVQQJExExMDEgU2Vjb25kIFN0cmVldDEOMAwGA1UEERMFOTQxMDUxFzAV
BgNVBAoTDkhhc2hpQ29ycCBJbmMuMUAwPgYDVQQDEzdDb25zdWwgQWdlbnQgQ0Eg
MTAxNjAxMjc4Mzc2ODc4MDAwNDc4MjA2MDAyMjA5MDk1Njk0MDQ3MB4XDTIzMTIx
MDA4NDQzNFoXDTQwMTIwNTA4NDQzNFowgbkxCzAJBgNVBAYTAlVTMQswCQYDVQQI
EwJDQTEWMBQGA1UEBxMNU2FuIEZyYW5jaXNjbzEaMBgGA1UECRMRMTAxIFNlY29u
ZCBTdHJlZXQxDjAMBgNVBBETBTk0MTA1MRcwFQYDVQQKEw5IYXNoaUNvcnAgSW5j
LjFAMD4GA1UEAxM3Q29uc3VsIEFnZW50IENBIDEwMTYwMTI3ODM3Njg3ODAwMDQ3
ODIwNjAwMjIwOTA5NTY5NDA0NzBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABHfc
0iOhg1QVN2OIq3Qb2WhX9Y3rHLrNdHeG0Xz/n2ltHLAauiKWalxgOXRbBmAtEJUk
UHRFJDzWJrMZTmeueB2jezB5MA4GA1UdDwEB/wQEAwIBhjAPBgNVHRMBAf8EBTAD
AQH/MCkGA1UdDgQiBCCztbgjUDgNOMTEynYTcSWobb2CBiLBLFLdKrhj6VungDAr
BgNVHSMEJDAigCCztbgjUDgNOMTEynYTcSWobb2CBiLBLFLdKrhj6VungDAKBggq
hkjOPQQDAgNHADBEAiBVqXP2mG8KbrBo+S3kYmBs4TUVzzZkzrS67HPwipsXrAIg
NaAH90rh5KxmSBaRZnYZkqXVeiCS4xcG7r3ZZfl+5iE=
-----END CERTIFICATE-----
LilBoiPeepLikesBenzTruck
chown consul $TMP_SECRET_PATH
chgrp consul $TMP_SECRET_PATH
unset NEEDS_MOVE
cmp --silent $TMP_SECRET_PATH /run/keys/consul-tls-ca-cert.pem || NEEDS_MOVE=true
[ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/consul-tls-ca-cert.pem')" ] || NEEDS_MOVE=true
[ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/consul-tls-ca-cert.pem || rm -f $TMP_SECRET_PATH
TMP_SECRET_PATH=/run/tmpsec-$RANDOM
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
edaf13c7-6efe-484d-a19a-639fb89a4186
LilBoiPeepLikesBenzTruck
unset NEEDS_MOVE
cmp --silent $TMP_SECRET_PATH /run/keys/consul-agent-token.txt || NEEDS_MOVE=true
[ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/consul-agent-token.txt')" ] || NEEDS_MOVE=true
[ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/consul-agent-token.txt || rm -f $TMP_SECRET_PATH
TMP_SECRET_PATH=/run/tmpsec-$RANDOM
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIBZTCCAQqgAwIBAgIUA1Rf+QzWDRp7TOCcm77pJrufaX0wCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjMxMjEwMDg0MDAwWhcNNDAxMjA1MDg0MDAw
WjAQMQ4wDAYDVQQDEwVub21hZDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABMEG
BMbE77RYrk44Sx6N0iRvrDemC60NFF5mSOmqd5ISiL9HnmxSesSuLUD2CimRonBa
b3CwHUXc19fCUIUvcZmjQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBSIaEmGc1TZqroZHSDON2TLjgFazDAKBggqhkjOPQQDAgNJ
ADBGAiEA96Kbui7gZAtmLFWC25/SLeYWtLmhHhiX/SX8bviWtTMCIQDv0h32ruvR
d8U8yrMaNQ7XFbDBnHeoKbiIg7t/kww1kQ==
-----END CERTIFICATE-----
LilBoiPeepLikesBenzTruck
unset NEEDS_MOVE
cmp --silent $TMP_SECRET_PATH /run/keys/nomad-ca.crt || NEEDS_MOVE=true
[ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/nomad-ca.crt')" ] || NEEDS_MOVE=true
[ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/nomad-ca.crt || rm -f $TMP_SECRET_PATH
TMP_SECRET_PATH=/run/tmpsec-$RANDOM
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIBxzCCAW6gAwIBAgIUAM3JpWE5R4ElpgOHxE9lsFhHELgwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFbm9tYWQwHhcNMjMxMjEwMDg0MDAwWhcNMjQxMjA5MDg0MDAw
WjAAMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEilzvvJsGs8OHN/QzZWmnKuVj
MD4GJJfWrrTWIMDtmsOnwVa6u3yjxNB2fn4DQ3/qE4Z65lfshC1Bm9Fqic7t7KOB
tTCBsjAOBgNVHQ8BAf8EBAMCBaAwHQYDVR0lBBYwFAYIKwYBBQUHAwEGCCsGAQUF
BwMCMAwGA1UdEwEB/wQCMAAwHQYDVR0OBBYEFHnWBszk3LSnzmWY6+FKkkXtS9tt
MB8GA1UdIwQYMBaAFIhoSYZzVNmquhkdIM43ZMuOAVrMMDMGA1UdEQEB/wQpMCeC
FGNsaWVudC51cy13ZXN0Lm5vbWFkgglsb2NhbGhvc3SHBH8AAAEwCgYIKoZIzj0E
AwIDRwAwRAIgEF/a9of7rB9BuWSqq4xmxYaTKdbjIAOXefQpQaq+s9sCIFWxXyQB
v5LZR3wnCgw0jbov3kxxFE4TgrcRCloVY+VN
-----END CERTIFICATE-----
LilBoiPeepLikesBenzTruck
unset NEEDS_MOVE
cmp --silent $TMP_SECRET_PATH /run/keys/nomad-client.crt || NEEDS_MOVE=true
[ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/nomad-client.crt')" ] || NEEDS_MOVE=true
[ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/nomad-client.crt || rm -f $TMP_SECRET_PATH
TMP_SECRET_PATH=/run/tmpsec-$RANDOM
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIPulviehaNqqhhogr0fP86wmrle18MxQGV/5cFCNn+ZGoAoGCCqGSM49
AwEHoUQDQgAEilzvvJsGs8OHN/QzZWmnKuVjMD4GJJfWrrTWIMDtmsOnwVa6u3yj
xNB2fn4DQ3/qE4Z65lfshC1Bm9Fqic7t7A==
-----END EC PRIVATE KEY-----
LilBoiPeepLikesBenzTruck
unset NEEDS_MOVE
cmp --silent $TMP_SECRET_PATH /run/keys/nomad-client.key || NEEDS_MOVE=true
[ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/nomad-client.key')" ] || NEEDS_MOVE=true
[ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/nomad-client.key || rm -f $TMP_SECRET_PATH
TMP_SECRET_PATH=/run/tmpsec-$RANDOM
cat > $TMP_SECRET_PATH <<'LilBoiPeepLikesBenzTruck'
-----BEGIN CERTIFICATE-----
MIIBZTCCAQqgAwIBAgIUEG/FUYCY2e7t5RpR2Fiob2DRYWgwCgYIKoZIzj0EAwIw
EDEOMAwGA1UEAxMFdmF1bHQwHhcNMjMxMjEwMDg0MDAwWhcNNDAxMjA1MDg0MDAw
WjAQMQ4wDAYDVQQDEwV2YXVsdDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABO0S
uQyFy64bbzPnt8SXlEMMG5F7w6bK3c+7WbhDlmxdtL7G5T4F0jQZa9tYMzZWdPJy
bcFk/D0d+njRx2dfEFujQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTAD
AQH/MB0GA1UdDgQWBBQro1VtoctSvekfgXtFOeiOXNn21zAKBggqhkjOPQQDAgNJ
ADBGAiEAwJ8BoWndiIp6UTQg10YI3dUj0OBMlk3EbODNSaBi894CIQDvMfK6uu0c
vtgvVueNMmbOlTGoFOi0xZjX2tK3KVmbRg==
-----END CERTIFICATE-----
LilBoiPeepLikesBenzTruck
chmod 0644 $TMP_SECRET_PATH
unset NEEDS_MOVE
cmp --silent $TMP_SECRET_PATH /run/keys/vault-ca.crt || NEEDS_MOVE=true
[ "$(stat -c '%A:%U:%G' $TMP_SECRET_PATH)" == "$(stat -c '%A:%U:%G' '/run/keys/vault-ca.crt')" ] || NEEDS_MOVE=true
[ -n "$NEEDS_MOVE" ] && mv -f $TMP_SECRET_PATH /run/keys/vault-ca.crt || rm -f $TMP_SECRET_PATH
L1_RESTART_CONSUL_POST_SECRETS && echo restarting consul after sleepint 10 seconds... && sleep 10 && systemctl restart consul.service || true
rm -f /run/epl-l1-prov

echo "
    UPDATE l1_provisionings
    SET exit_code = 0,
        time_ended = CURRENT_TIMESTAMP,
        is_finished = 1
    WHERE provisioning_id = L1_EPL_PROVISIONING_ID
" | sqlite3 /var/lib/epl-l1-prov/provisionings.sqlite

chmod 644 /var/log/epl-l1-prov/L1_EPL_PROVISIONING_ID.log

ThisIsEplProvL1Script
chmod 700 /run/epl-l1-prov
echo "SELECT 'running provisioning id is unfinished', provisioning_id FROM l1_provisionings WHERE is_finished = 0;" | sqlite3 /var/lib/epl-l1-prov/provisionings.sqlite | grep unfinished && exit 27 || true
echo 'INSERT INTO l1_provisionings(provisioning_id) VALUES (L1_EPL_PROVISIONING_ID);' | sqlite3 /var/lib/epl-l1-prov/provisionings.sqlite
tmux new-session -d '/run/epl-l1-prov |& tee /var/log/epl-l1-prov/L1_EPL_PROVISIONING_ID.log'
