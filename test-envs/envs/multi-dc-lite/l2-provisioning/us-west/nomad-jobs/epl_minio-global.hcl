job "minio-global" {
  type = "service"
  namespace = "epl"
  region = "us-west"
  datacenters = ["dc1", "dc2", "dc3"]

  vault {
    policies = ["epl-minio-global"]
  }
  update {
    auto_revert = false
    max_parallel = 0
    health_check = "checks"
    min_healthy_time = "0s"
    stagger = "30s"
    healthy_deadline = "300s"
    progress_deadline = "600s"
  }

  group "epl-minio-global-1" {
    count = 1
    shutdown_delay = "0s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-minio-server-a-global}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "api" {
        static = 9000
        host_network = "lan"
      }
      port "con" {
        static = 9001
        host_network = "lan"
      }
      port "lb" {
        static = 9002
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "minio-docker-a"
      read_only = false
    }

    service {
      name = "epl-minio-global"
      port = "lb"
      address = "${meta.private_ip}"
      check {
        type = "tcp"
        port = "lb"
        interval = "10s"
        timeout = "2s"
      }
    }

    service {
      name = "epl-minio-global-api"
      port = "api"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/minio/v2/metrics/cluster"
      }
      check {
        type = "tcp"
        port = "api"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "epl-minio-global-1-provision-buckets" {
      driver = "docker"
      resources {
        memory = 128
        memory_max = 256
      }
      lifecycle {
        sidecar = false
        hook = "poststart"
      }
      config {
        image = "minio/mc@sha256:1f374e2f61a8a4902ed528fb1d19f93a44b3d81a158b003779f85883833990c3"
        network_mode = "host"
        entrypoint = [
          "/bin/bash",
          "/secrets/entrypoint.sh",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      template {
        destination = "secrets/entrypoint.sh"
        perms = "644"
        data = <<EOL

set -e

mkdir -p /secrets/.mc
ln -s /secrets/.mc /root/.mc

while ! curl -f http://epl-minio-global.service.consul:9002/minio/health/cluster
do
  echo minio healthcheck failed, retrying in one second...
  sleep 1
done

mc alias set thisminio http://epl-minio-global.service.consul:9002 minio {{ with secret "epl/data/minio/global" }}{{ .Data.data.admin_password }}{{ end }}

while ! mc ls thisminio/
do
  echo minio list buckets failed, retrying in one second...
  sleep 1
done

# provision buckets
mc mb --ignore-existing --with-lock thisminio/docker
mc mb --ignore-existing --with-lock thisminio/loki
mc mb --ignore-existing --with-lock thisminio/tempo


# privision policies

cat <<EOF > /secrets/policy.json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:ListBucket",
        "s3:GetBucketLocation",
        "s3:GetObject"
      ],
      "Resource": ["arn:aws:s3:::docker"]
    },
    {
      "Effect": "Allow",
      "Action": [
        "s3:*"
      ],
      "Resource": [
        "arn:aws:s3:::docker/*"
      ]
    }
  ]
}
EOF
mc admin policy add thisminio rw-docker /secrets/policy.json

cat <<EOF > /secrets/policy.json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:GetBucketLocation",
        "s3:GetObject",
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::docker/*",
        "arn:aws:s3:::docker"
      ]
    }
  ]
}
EOF
mc admin policy add thisminio ro-docker /secrets/policy.json

cat <<EOF > /secrets/policy.json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:ListBucket",
        "s3:GetBucketLocation",
        "s3:GetObject"
      ],
      "Resource": ["arn:aws:s3:::loki"]
    },
    {
      "Effect": "Allow",
      "Action": [
        "s3:*"
      ],
      "Resource": [
        "arn:aws:s3:::loki/*"
      ]
    }
  ]
}
EOF
mc admin policy add thisminio rw-loki /secrets/policy.json

cat <<EOF > /secrets/policy.json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:GetBucketLocation",
        "s3:GetObject",
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::loki/*",
        "arn:aws:s3:::loki"
      ]
    }
  ]
}
EOF
mc admin policy add thisminio ro-loki /secrets/policy.json

cat <<EOF > /secrets/policy.json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:ListBucket",
        "s3:GetBucketLocation",
        "s3:GetObject"
      ],
      "Resource": ["arn:aws:s3:::tempo"]
    },
    {
      "Effect": "Allow",
      "Action": [
        "s3:*"
      ],
      "Resource": [
        "arn:aws:s3:::tempo/*"
      ]
    }
  ]
}
EOF
mc admin policy add thisminio rw-tempo /secrets/policy.json

cat <<EOF > /secrets/policy.json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:GetBucketLocation",
        "s3:GetObject",
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::tempo/*",
        "arn:aws:s3:::tempo"
      ]
    }
  ]
}
EOF
mc admin policy add thisminio ro-tempo /secrets/policy.json


# provision extra bucket users
mc admin user add thisminio docker_registry {{ with secret "epl/data/minio/global" }}{{ .Data.data.minio_user_docker_registry_password }}{{ end }}
mc admin policy set thisminio rw-docker user=docker_registry
mc admin user add thisminio loki_main {{ with secret "epl/data/minio/global" }}{{ .Data.data.minio_user_loki_main_password }}{{ end }}
mc admin policy set thisminio rw-loki user=loki_main
mc admin user add thisminio tempo_r1_tempo {{ with secret "epl/data/minio/global" }}{{ .Data.data.minio_user_tempo_r1_tempo_password }}{{ end }}
mc admin policy set thisminio rw-tempo user=tempo_r1_tempo
EOL
      }
    }

    task "minio-global-daemon" {
      driver = "docker"
      resources {
        memory = 1024
        memory_max = 1152
      }
      env {
        MINIO_PROMETHEUS_AUTH_TYPE = "public"
        MINIO_ROOT_USER = "minio"
      }
      config {
        image = "minio/minio@sha256:5db7e40b69f0c3ad5a878521ff5029468e3070ef146c084dc2540e2d492075c4"
        network_mode = "host"
        args = [
          "server",
          "--address=${meta.private_ip}:9000",
          "--console-address=${meta.private_ip}:9001",
          "http://10.17.0.10:9000/var/lib/minio",
          "http://10.17.0.13:9000/var/lib/minio",
          "http://10.18.0.10:9000/var/lib/minio",
          "http://10.19.0.11:9000/var/lib/minio",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      volume_mount {
        volume = "v_1"
        destination = "/var/lib/minio"
      }

      template {
        destination = "secrets/admin_password"
        perms = "644"
        env = true
        data = <<EOL
MINIO_ROOT_PASSWORD={{ with secret "epl/data/minio/global" }}{{ .Data.data.admin_password }}{{ end }}
EOL
      }
    }

    task "minio-global-lb" {
      driver = "docker"
      resources {
        memory = 64
        memory_max = 192
      }
      config {
        image = "nginx@sha256:b8f2383a95879e1ae064940d9a200f67a6c79e710ed82ac42263397367e7cc4e"
        network_mode = "host"
        entrypoint = [
          "/usr/sbin/nginx",
          "-g",
          "daemon off;",
          "-c",
          "/secrets/nginx.conf",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      template {
        destination = "secrets/nginx.conf"
        perms = "644"
        data = <<EOL

pcre_jit on;

worker_processes auto;
worker_rlimit_nofile 12288;

events {
    worker_connections  1024;
}

http {
    include       /etc/nginx/mime.types;
    default_type  application/octet-stream;

    # Log in JSON Format
    log_format nginxlog_json escape=json '{ "@timestamp": "$time_iso8601", '
         '"remote_addr": "$remote_addr", '
         '"body_bytes_sent": $body_bytes_sent, '
         '"request_time": $request_time, '
         '"response_status": $status, '
         '"request": "$request", '
         '"request_method": "$request_method", '
         '"host": "$host",'
         '"upstream_addr": "$upstream_addr",'
         '"http_x_forwarded_for": "$http_x_forwarded_for",'
         '"http_referrer": "$http_referer", '
         '"http_user_agent": "$http_user_agent", '
         '"http_version": "$server_protocol", '
         '"server_port": "$server_port"}';
    access_log /dev/stdout nginxlog_json;

    sendfile        on;

    keepalive_timeout  65;

    include /secrets/site.conf;
}
EOL
      }

      template {
        destination = "secrets/site.conf"
        perms = "644"
        data = <<EOL

upstream minio {
    least_conn;
    server 10.17.0.10:9000;
    server 10.17.0.13:9000;
    server 10.18.0.10:9000;
    server 10.19.0.11:9000;

}

server {
    listen 10.17.0.10:9002;

    ignore_invalid_headers off;
    # Allow any size file to be uploaded.
    # Set to a value such as 1000m; to restrict file size to a specific value
    client_max_body_size 0;
    # To disable buffering
    proxy_buffering off;


    location / {
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Host $http_host;

        proxy_connect_timeout 300;
        # Default is HTTP/1, keepalive is only enabled in HTTP/1.1
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        chunked_transfer_encoding off;

        proxy_pass http://minio/;
    }
}
EOL
      }
    }
  }

  group "epl-minio-global-2" {
    count = 1
    shutdown_delay = "0s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-minio-server-b-global}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "api" {
        static = 9000
        host_network = "lan"
      }
      port "con" {
        static = 9001
        host_network = "lan"
      }
      port "lb" {
        static = 9002
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "minio-docker-b"
      read_only = false
    }

    service {
      name = "epl-minio-global"
      port = "lb"
      address = "${meta.private_ip}"
      check {
        type = "tcp"
        port = "lb"
        interval = "10s"
        timeout = "2s"
      }
    }

    service {
      name = "epl-minio-global-api"
      port = "api"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/minio/v2/metrics/cluster"
      }
      check {
        type = "tcp"
        port = "api"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "minio-global-daemon" {
      driver = "docker"
      resources {
        memory = 1024
        memory_max = 1152
      }
      env {
        MINIO_PROMETHEUS_AUTH_TYPE = "public"
        MINIO_ROOT_USER = "minio"
      }
      config {
        image = "minio/minio@sha256:5db7e40b69f0c3ad5a878521ff5029468e3070ef146c084dc2540e2d492075c4"
        network_mode = "host"
        args = [
          "server",
          "--address=${meta.private_ip}:9000",
          "--console-address=${meta.private_ip}:9001",
          "http://10.17.0.10:9000/var/lib/minio",
          "http://10.17.0.13:9000/var/lib/minio",
          "http://10.18.0.10:9000/var/lib/minio",
          "http://10.19.0.11:9000/var/lib/minio",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      volume_mount {
        volume = "v_1"
        destination = "/var/lib/minio"
      }

      template {
        destination = "secrets/admin_password"
        perms = "644"
        env = true
        data = <<EOL
MINIO_ROOT_PASSWORD={{ with secret "epl/data/minio/global" }}{{ .Data.data.admin_password }}{{ end }}
EOL
      }
    }

    task "minio-global-lb" {
      driver = "docker"
      resources {
        memory = 64
        memory_max = 192
      }
      config {
        image = "nginx@sha256:b8f2383a95879e1ae064940d9a200f67a6c79e710ed82ac42263397367e7cc4e"
        network_mode = "host"
        entrypoint = [
          "/usr/sbin/nginx",
          "-g",
          "daemon off;",
          "-c",
          "/secrets/nginx.conf",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      template {
        destination = "secrets/nginx.conf"
        perms = "644"
        data = <<EOL

pcre_jit on;

worker_processes auto;
worker_rlimit_nofile 12288;

events {
    worker_connections  1024;
}

http {
    include       /etc/nginx/mime.types;
    default_type  application/octet-stream;

    # Log in JSON Format
    log_format nginxlog_json escape=json '{ "@timestamp": "$time_iso8601", '
         '"remote_addr": "$remote_addr", '
         '"body_bytes_sent": $body_bytes_sent, '
         '"request_time": $request_time, '
         '"response_status": $status, '
         '"request": "$request", '
         '"request_method": "$request_method", '
         '"host": "$host",'
         '"upstream_addr": "$upstream_addr",'
         '"http_x_forwarded_for": "$http_x_forwarded_for",'
         '"http_referrer": "$http_referer", '
         '"http_user_agent": "$http_user_agent", '
         '"http_version": "$server_protocol", '
         '"server_port": "$server_port"}';
    access_log /dev/stdout nginxlog_json;

    sendfile        on;

    keepalive_timeout  65;

    include /secrets/site.conf;
}
EOL
      }

      template {
        destination = "secrets/site.conf"
        perms = "644"
        data = <<EOL

upstream minio {
    least_conn;
    server 10.17.0.10:9000;
    server 10.17.0.13:9000;
    server 10.18.0.10:9000;
    server 10.19.0.11:9000;

}

server {
    listen 10.17.0.13:9002;

    ignore_invalid_headers off;
    # Allow any size file to be uploaded.
    # Set to a value such as 1000m; to restrict file size to a specific value
    client_max_body_size 0;
    # To disable buffering
    proxy_buffering off;


    location / {
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Host $http_host;

        proxy_connect_timeout 300;
        # Default is HTTP/1, keepalive is only enabled in HTTP/1.1
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        chunked_transfer_encoding off;

        proxy_pass http://minio/;
    }
}
EOL
      }
    }
  }

  group "epl-minio-global-3" {
    count = 1
    shutdown_delay = "0s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-minio-server-c-global}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "api" {
        static = 9000
        host_network = "lan"
      }
      port "con" {
        static = 9001
        host_network = "lan"
      }
      port "lb" {
        static = 9002
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "minio-docker-c"
      read_only = false
    }

    service {
      name = "epl-minio-global"
      port = "lb"
      address = "${meta.private_ip}"
      check {
        type = "tcp"
        port = "lb"
        interval = "10s"
        timeout = "2s"
      }
    }

    service {
      name = "epl-minio-global-api"
      port = "api"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/minio/v2/metrics/cluster"
      }
      check {
        type = "tcp"
        port = "api"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "minio-global-daemon" {
      driver = "docker"
      resources {
        memory = 1024
        memory_max = 1152
      }
      env {
        MINIO_PROMETHEUS_AUTH_TYPE = "public"
        MINIO_ROOT_USER = "minio"
      }
      config {
        image = "minio/minio@sha256:5db7e40b69f0c3ad5a878521ff5029468e3070ef146c084dc2540e2d492075c4"
        network_mode = "host"
        args = [
          "server",
          "--address=${meta.private_ip}:9000",
          "--console-address=${meta.private_ip}:9001",
          "http://10.17.0.10:9000/var/lib/minio",
          "http://10.17.0.13:9000/var/lib/minio",
          "http://10.18.0.10:9000/var/lib/minio",
          "http://10.19.0.11:9000/var/lib/minio",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      volume_mount {
        volume = "v_1"
        destination = "/var/lib/minio"
      }

      template {
        destination = "secrets/admin_password"
        perms = "644"
        env = true
        data = <<EOL
MINIO_ROOT_PASSWORD={{ with secret "epl/data/minio/global" }}{{ .Data.data.admin_password }}{{ end }}
EOL
      }
    }

    task "minio-global-lb" {
      driver = "docker"
      resources {
        memory = 64
        memory_max = 192
      }
      config {
        image = "nginx@sha256:b8f2383a95879e1ae064940d9a200f67a6c79e710ed82ac42263397367e7cc4e"
        network_mode = "host"
        entrypoint = [
          "/usr/sbin/nginx",
          "-g",
          "daemon off;",
          "-c",
          "/secrets/nginx.conf",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      template {
        destination = "secrets/nginx.conf"
        perms = "644"
        data = <<EOL

pcre_jit on;

worker_processes auto;
worker_rlimit_nofile 12288;

events {
    worker_connections  1024;
}

http {
    include       /etc/nginx/mime.types;
    default_type  application/octet-stream;

    # Log in JSON Format
    log_format nginxlog_json escape=json '{ "@timestamp": "$time_iso8601", '
         '"remote_addr": "$remote_addr", '
         '"body_bytes_sent": $body_bytes_sent, '
         '"request_time": $request_time, '
         '"response_status": $status, '
         '"request": "$request", '
         '"request_method": "$request_method", '
         '"host": "$host",'
         '"upstream_addr": "$upstream_addr",'
         '"http_x_forwarded_for": "$http_x_forwarded_for",'
         '"http_referrer": "$http_referer", '
         '"http_user_agent": "$http_user_agent", '
         '"http_version": "$server_protocol", '
         '"server_port": "$server_port"}';
    access_log /dev/stdout nginxlog_json;

    sendfile        on;

    keepalive_timeout  65;

    include /secrets/site.conf;
}
EOL
      }

      template {
        destination = "secrets/site.conf"
        perms = "644"
        data = <<EOL

upstream minio {
    least_conn;
    server 10.17.0.10:9000;
    server 10.17.0.13:9000;
    server 10.18.0.10:9000;
    server 10.19.0.11:9000;

}

server {
    listen 10.18.0.10:9002;

    ignore_invalid_headers off;
    # Allow any size file to be uploaded.
    # Set to a value such as 1000m; to restrict file size to a specific value
    client_max_body_size 0;
    # To disable buffering
    proxy_buffering off;


    location / {
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Host $http_host;

        proxy_connect_timeout 300;
        # Default is HTTP/1, keepalive is only enabled in HTTP/1.1
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        chunked_transfer_encoding off;

        proxy_pass http://minio/;
    }
}
EOL
      }
    }
  }

  group "epl-minio-global-4" {
    count = 1
    shutdown_delay = "0s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-minio-server-f-global}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "api" {
        static = 9000
        host_network = "lan"
      }
      port "con" {
        static = 9001
        host_network = "lan"
      }
      port "lb" {
        static = 9002
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "minio-docker-f"
      read_only = false
    }

    service {
      name = "epl-minio-global"
      port = "lb"
      address = "${meta.private_ip}"
      check {
        type = "tcp"
        port = "lb"
        interval = "10s"
        timeout = "2s"
      }
    }

    service {
      name = "epl-minio-global-api"
      port = "api"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/minio/v2/metrics/cluster"
      }
      check {
        type = "tcp"
        port = "api"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "minio-global-daemon" {
      driver = "docker"
      resources {
        memory = 1024
        memory_max = 1152
      }
      env {
        MINIO_PROMETHEUS_AUTH_TYPE = "public"
        MINIO_ROOT_USER = "minio"
      }
      config {
        image = "minio/minio@sha256:5db7e40b69f0c3ad5a878521ff5029468e3070ef146c084dc2540e2d492075c4"
        network_mode = "host"
        args = [
          "server",
          "--address=${meta.private_ip}:9000",
          "--console-address=${meta.private_ip}:9001",
          "http://10.17.0.10:9000/var/lib/minio",
          "http://10.17.0.13:9000/var/lib/minio",
          "http://10.18.0.10:9000/var/lib/minio",
          "http://10.19.0.11:9000/var/lib/minio",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      volume_mount {
        volume = "v_1"
        destination = "/var/lib/minio"
      }

      template {
        destination = "secrets/admin_password"
        perms = "644"
        env = true
        data = <<EOL
MINIO_ROOT_PASSWORD={{ with secret "epl/data/minio/global" }}{{ .Data.data.admin_password }}{{ end }}
EOL
      }
    }

    task "minio-global-lb" {
      driver = "docker"
      resources {
        memory = 64
        memory_max = 192
      }
      config {
        image = "nginx@sha256:b8f2383a95879e1ae064940d9a200f67a6c79e710ed82ac42263397367e7cc4e"
        network_mode = "host"
        entrypoint = [
          "/usr/sbin/nginx",
          "-g",
          "daemon off;",
          "-c",
          "/secrets/nginx.conf",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      template {
        destination = "secrets/nginx.conf"
        perms = "644"
        data = <<EOL

pcre_jit on;

worker_processes auto;
worker_rlimit_nofile 12288;

events {
    worker_connections  1024;
}

http {
    include       /etc/nginx/mime.types;
    default_type  application/octet-stream;

    # Log in JSON Format
    log_format nginxlog_json escape=json '{ "@timestamp": "$time_iso8601", '
         '"remote_addr": "$remote_addr", '
         '"body_bytes_sent": $body_bytes_sent, '
         '"request_time": $request_time, '
         '"response_status": $status, '
         '"request": "$request", '
         '"request_method": "$request_method", '
         '"host": "$host",'
         '"upstream_addr": "$upstream_addr",'
         '"http_x_forwarded_for": "$http_x_forwarded_for",'
         '"http_referrer": "$http_referer", '
         '"http_user_agent": "$http_user_agent", '
         '"http_version": "$server_protocol", '
         '"server_port": "$server_port"}';
    access_log /dev/stdout nginxlog_json;

    sendfile        on;

    keepalive_timeout  65;

    include /secrets/site.conf;
}
EOL
      }

      template {
        destination = "secrets/site.conf"
        perms = "644"
        data = <<EOL

upstream minio {
    least_conn;
    server 10.17.0.10:9000;
    server 10.17.0.13:9000;
    server 10.18.0.10:9000;
    server 10.19.0.11:9000;

}

server {
    listen 10.19.0.11:9002;

    ignore_invalid_headers off;
    # Allow any size file to be uploaded.
    # Set to a value such as 1000m; to restrict file size to a specific value
    client_max_body_size 0;
    # To disable buffering
    proxy_buffering off;


    location / {
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Host $http_host;

        proxy_connect_timeout 300;
        # Default is HTTP/1, keepalive is only enabled in HTTP/1.1
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        chunked_transfer_encoding off;

        proxy_pass http://minio/;
    }
}
EOL
      }
    }
  }

}
