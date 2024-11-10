
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
