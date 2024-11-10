#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use pretty_assertions::assert_eq;

#[cfg(test)]
use super::common;

#[cfg(test)]
use super::networking::simulation::scenarios;

#[test]
fn test_placement_failure_only_one_server_with_label() {
    let source = common::replace_sources(&[
        (
            r#"
DATA STRUCT server [
  {
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface {
      if_name: eth0,
      if_network: lan,
      if_ip: 10.17.0.10,
      if_prefix: 24,
    }
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
      {
        volume_name: mon-secondary,
      },
      {
        volume_name: mon-sec-am,
      },
    ]
  },"#, r#"
DATA valid_server_labels {
  da_logz;
}

DATA STRUCT server [
  {
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    WITH server_label {
      label_name: da_logz,
      label_value: tru,
    }
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface {
      if_name: eth0,
      if_network: lan,
      if_ip: 10.17.0.10,
      if_prefix: 24,
    }
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
      {
        volume_name: mon-secondary,
      },
      {
        volume_name: mon-sec-am,
      },
    ]
  },"#
        ),
        (
            r#"
DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]
"#,
            r#"
DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    loki_readers: 4,
    reader_placement: '
      match_keys_and_values:
        da_logz: tru
    ',
    storage_bucket: global=>loki,
  }
]
"#,
        )
    ], scenarios::scenario_single_dc_env());
    assert_eq!(
        PlatformValidationError::FailedToFindPlacements {
            context: "readers for loki_cluster named main in region us-west".to_string(),
            need_at_least: 4,
            found_servers_count: 1,
            placement_query: "
      match_keys_and_values:
        da_logz: tru
    ".to_string(),
            found_servers: vec![
                "server-a".to_string(),
            ],
        },
        common::assert_platform_validation_error_wcustom_data_raw(&source),
    )
}

#[test]
fn test_placement_invalid_label() {
    let source = common::replace_sources(&[
        (
            r#"
DATA STRUCT server [
  {
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface {
      if_name: eth0,
      if_network: lan,
      if_ip: 10.17.0.10,
      if_prefix: 24,
    }
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
      {
        volume_name: mon-secondary,
      },
      {
        volume_name: mon-sec-am,
      },
    ]
  },"#, r#"
DATA valid_server_labels {
  da_logz;
}

DATA STRUCT server [
  {
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    WITH server_label {
      label_name: da_logz,
      label_value: tru,
    }
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface {
      if_name: eth0,
      if_network: lan,
      if_ip: 10.17.0.10,
      if_prefix: 24,
    }
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
      {
        volume_name: mon-secondary,
      },
      {
        volume_name: mon-sec-am,
      },
    ]
  },"#
        ),
        (
            r#"
DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]
"#,
            r#"
DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    loki_readers: 4,
    reader_placement: '
      match_keys_and_values:
        unknown_label: tru
    ',
    storage_bucket: global=>loki,
  }
]
"#,
        )
    ], scenarios::scenario_single_dc_env());
    assert_eq!(
        PlatformValidationError::InvalidServerLabelInQuery {
            invalid_label_key: "unknown_label".to_string(),
            label_value: "tru".to_string(),
            placement_query: "
      match_keys_and_values:
        unknown_label: tru
    ".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_raw(&source),
    )
}

#[test]
fn test_placement_invalid_yaml() {
    let source = common::replace_sources(&[
        (
            r#"
DATA STRUCT server [
  {
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface {
      if_name: eth0,
      if_network: lan,
      if_ip: 10.17.0.10,
      if_prefix: 24,
    }
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
      {
        volume_name: mon-secondary,
      },
      {
        volume_name: mon-sec-am,
      },
    ]
  },"#, r#"
DATA valid_server_labels {
  da_logz;
}

DATA STRUCT server [
  {
    hostname: server-a,
    ssh_interface: eth0,
    is_consul_master: true,
    is_nomad_master: true,
    is_vault_instance: false,
    is_dns_master: false,
    is_dns_slave: false,
    WITH server_label {
      label_name: da_logz,
      label_value: tru,
    }
    WITH server_disk {
      disk_id: 'vda'
    }
    WITH network_interface {
      if_name: eth0,
      if_network: lan,
      if_ip: 10.17.0.10,
      if_prefix: 24,
    }
    WITH server_root_volume [
      {
        volume_name: pgtest1,
      },
      {
        volume_name: minio-docker-a,
        zfs_recordsize: 1M,
      },
      {
        volume_name: mon-default,
      },
      {
        volume_name: mon-am,
      },
      {
        volume_name: mon-secondary,
      },
      {
        volume_name: mon-sec-am,
      },
    ]
  },"#
        ),
        (
            r#"
DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]
"#,
            r#"
DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    loki_readers: 4,
    reader_placement: 'lol',
    storage_bucket: global=>loki,
  }
]
"#,
        )
    ], scenarios::scenario_single_dc_env());
    assert_eq!(
        PlatformValidationError::LabelQueryParseError {
            placement_query: "lol".to_string(),
            parsing_error: "invalid type: string \"lol\", expected struct LabelQuery".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data_raw(&source),
    )
}

#[test]
fn test_placement_failure_for_exceeding_region_count() {
    let source = common::replace_sources(&[
        (
            r#"
DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    storage_bucket: global=>loki,
  }
]
"#,
            r#"
DATA STRUCT loki_cluster [
  {
    cluster_name: main,
    is_region_default: true,
    loki_readers: 5,
    storage_bucket: global=>loki,
  }
]
"#,
        )
    ], scenarios::scenario_single_dc_env());
    assert_eq!(
        PlatformValidationError::FailedToFindPlacements {
            context: "readers for loki_cluster named main in region us-west".to_string(),
            need_at_least: 5,
            found_servers_count: 4,
            placement_query: "".to_string(),
            found_servers: vec![
                "server-a".to_string(),
                "server-b".to_string(),
                "server-c".to_string(),
                "server-d".to_string(),
            ],
        },
        common::assert_platform_validation_error_wcustom_data_raw(&source),
    )
}
