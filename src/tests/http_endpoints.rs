#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::common;

#[cfg(test)]
use edendb::checker::errors::DatabaseValidationError;

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn test_http_multiple_question_marks_in_path() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/henlo/boi?hey?yo",
        http_method: GET,
      }
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::HttpPathBadFormat {
            explanation: "More than one ? (question mark) found in path".to_string(),
            path: "/henlo/boi?hey?yo".to_string(),
        }
    );
}

#[test]
fn test_http_invalid_query_segment() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/henlo/boi?hey",
        http_method: GET,
      }
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::HttpCantParseQueryArgument {
            full_path: "/henlo/boi?hey".to_string(),
            expected_segment_example: "{some_variable:INT}",
            actual_segment: "hey".to_string(),
        }
    );
}

#[test]
fn test_http_invalid_query_type() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/henlo/boi?{var:BAD}",
        http_method: GET,
      }
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::HttpInvalidArgumentType {
            segment: "{var:BAD}".to_string(),
            the_type: "BAD".to_string(),
            full_path: "/henlo/boi?{var:BAD}".to_string(),
            allowed_types: vec!["INT", "FLOAT", "BOOL", "TEXT"],
        }
    );
}

#[test]
fn test_http_empty_core_path() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "",
        http_method: GET,
      }
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::HttpEmptyCorePath {
            full_path: "".to_string(),
        }
    );
}

#[test]
fn test_http_empty_core_path_wquery() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "?{var:INT}",
        http_method: GET,
      }
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::HttpEmptyCorePath {
            full_path: "?{var:INT}".to_string(),
        }
    );
}

#[test]
fn test_http_core_path_doesnt_start_with_slash() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "hello?{var:INT}",
        http_method: GET,
      }
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::HttpCorePathMustStartWithSlash {
            full_path: "hello?{var:INT}".to_string(),
        }
    );
}

// path end becomes a comment and cannot be parsed
// #[test]
// fn test_http_core_path_multiple_slashes() {
//     let err = common::assert_platform_validation_error_wcustom_data(r#"
// DATA STRUCT application [
//   {
//     application_name: hello-world,
//     WITH http_endpoint [
//       {
//         http_endpoint_name: primary,
//         data_type: json,
//         path: "/he//llo?{var:INT}",
//         http_method: GET,
//       }
//     ]
//   }
// ]
// "#);
//     assert_eq!(err, PlatformValidationError::HttpMultipleConsecutiveSlashesNotAllowed {
//         full_path: "/he//llo?{var:INT}".to_string(),
//     });
// }

#[test]
fn test_http_core_path_invalid_segment() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/hello/^/{other_var:INT}?{var:INT}",
        http_method: GET,
      }
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::HttpInvalidCoreSegment {
            full_path: "/hello/^/{other_var:INT}?{var:INT}".to_string(),
            segment: "^".to_string(),
            explanation: "Either argument or alphanumeric string is allowed in core path segment",
        }
    );
}

#[test]
fn test_http_core_path_invalid_segment_type() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/hello/{other_var:UNKNOWN}?{var:INT}",
        http_method: GET,
      }
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::HttpInvalidArgumentType {
            full_path: "/hello/{other_var:UNKNOWN}?{var:INT}".to_string(),
            segment: "{other_var:UNKNOWN}".to_string(),
            the_type: "UNKNOWN".to_string(),
            allowed_types: vec!["INT", "FLOAT", "BOOL", "TEXT"],
        }
    );
}

#[test]
fn test_http_core_path_duplicate_args() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/{var:INT}/{var:BOOL}",
        http_method: GET,
      }
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::HttpPathDuplicateArgumentName {
            full_path: "/{var:INT}/{var:BOOL}".to_string(),
            duplicate_arg_name: "var".to_string(),
        }
    );
}

#[test]
fn test_http_core_path_duplicate_query() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/hello?{var:BOOL}&{var:INT}",
        http_method: GET,
      }
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::HttpPathDuplicateArgumentName {
            full_path: "/hello?{var:BOOL}&{var:INT}".to_string(),
            duplicate_arg_name: "var".to_string(),
        }
    );
}

#[test]
fn test_http_core_path_duplicate_core_and_query() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/hello/{var:BOOL}?{var:INT}",
        http_method: GET,
      }
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::HttpPathDuplicateArgumentName {
            full_path: "/hello/{var:BOOL}?{var:INT}".to_string(),
            duplicate_arg_name: "var".to_string(),
        }
    );
}

#[test]
fn test_http_get_method_cannot_have_input_body() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/hello",
        http_method: GET,
        input_body_type: henlo,
      }
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::HttpEndpointGetMethodCannotHaveInputBody {
            full_path: "/hello".to_string(),
            application_name: "hello-world".to_string(),
            endpoint_name: "primary".to_string(),
            input_body_type: "henlo".to_string(),
        }
    );
}

#[test]
fn test_http_unknown_input_body_type() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/hello",
        http_method: POST,
        input_body_type: henlo,
      }
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::HttpPathInputBwTypeNotFound {
            full_path: "/hello".to_string(),
            application_name: "hello-world".to_string(),
            endpoint_name: "primary".to_string(),
            input_body_type: "henlo".to_string(),
        }
    );
}

#[test]
fn test_http_unknown_output_body_type() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/hello",
        http_method: POST,
        output_body_type: henlo,
      }
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::HttpPathOutputBwTypeNotFound {
            full_path: "/hello".to_string(),
            application_name: "hello-world".to_string(),
            endpoint_name: "primary".to_string(),
            output_body_type: "henlo".to_string(),
        }
    );
}

#[test]
fn test_http_json_unspecified_output_type() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/hello",
        http_method: GET,
      }
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::HttpEndpointBodyTypeIsJsonButOutputBwTypeIsUnspecified {
            full_path: "/hello".to_string(),
            application_name: "hello-world".to_string(),
            endpoint_name: "primary".to_string(),
        }
    );
}

#[test]
fn test_http_json_unspecified_input_type() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/hello",
        http_method: POST,
        output_body_type: test_type,
      }
    ]
  }
]

DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 1,
        snapshot_source: "{
            input @0 :String,
        }"
    }]
}
"#,
    );
    assert_eq!(err, PlatformValidationError::HttpEndpointBodyTypeIsJsonAndPostPutMethodButInputBwTypeIsUnspecified {
        full_path: "/hello".to_string(),
        application_name: "hello-world".to_string(),
        endpoint_name: "primary".to_string(),
        http_method: "POST".to_string(),
    });
}

#[test]
fn test_http_path_cant_start_with_reserved_name_metrics() {
    let err = common::assert_eden_db_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/metrics",
        http_method: GET,
      }
    ]
  }
]
"#,
    );

    match err {
        DatabaseValidationError::LuaCheckEvaluationFailed { .. } => {}
        _ => {
            panic!("Wrong error: {:?}", err)
        }
    }
}

#[test]
fn test_http_path_cant_start_with_reserved_name_health() {
    let err = common::assert_eden_db_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/health",
        http_method: GET,
      }
    ]
  }
]
"#,
    );

    match err {
        DatabaseValidationError::LuaCheckEvaluationFailed { .. } => {}
        _ => {
            panic!("Wrong error: {:?}", err)
        }
    }
}

#[test]
fn test_http_duplicate_path_detection() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/endpoint_a",
        http_method: GET,
      },
      {
        http_endpoint_name: secondary,
        data_type: html,
        path: "/endpoint_a",
        http_method: GET,
      },
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::AppHttpTreeErrorDuplicatePagePath {
            application_name: "hello-world".to_string(),
            previous_endpoint_name: "primary".to_string(),
            previous_endpoint_path: "/endpoint_a".to_string(),
            duplicate_endpoint_name: "secondary".to_string(),
            duplicate_endpoint_path: "/endpoint_a".to_string(),
            page_method: "GET".to_string(),
        }
    );
}

#[test]
fn test_http_double_root_lock_detection() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/endpoint_a/",
        http_method: GET,
      },
      {
        http_endpoint_name: secondary,
        data_type: html,
        path: "/endpoint_a/",
        http_method: GET,
      },
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::AppHttpTreeErrorDuplicatePagePath {
            application_name: "hello-world".to_string(),
            previous_endpoint_name: "primary".to_string(),
            previous_endpoint_path: "/endpoint_a/".to_string(),
            duplicate_endpoint_name: "secondary".to_string(),
            duplicate_endpoint_path: "/endpoint_a/".to_string(),
            page_method: "GET".to_string(),
        }
    );
}

#[test]
fn test_http_path_segment_mixing() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: secondary,
        data_type: html,
        path: "/endpoint_a/",
        http_method: GET,
      },
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/{arg_a:INT}/",
        http_method: GET,
      },
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::AppHttpTreeErrorArgPageMixedWithStaticPage {
            application_name: "hello-world".to_string(),
            a_endpoint_name: "primary".to_string(),
            a_endpoint_path: "/{arg_a:INT}/".to_string(),
            b_endpoint_name: "secondary".to_string(),
            b_endpoint_path: "/endpoint_a/".to_string(),
        }
    );
}

#[test]
fn test_http_path_segment_arg_page_diff_types() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/{arg_a:INT}/",
        http_method: GET,
      },
      {
        http_endpoint_name: secondary,
        data_type: html,
        path: "/{arg_a:BOOL}/",
        http_method: GET,
      },
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::AppHttpTreeErrorArgPageSegmentMultipleNames {
            application_name: "hello-world".to_string(),
            arg_a_endpoint_name: "primary".to_string(),
            arg_a_endpoint_path: "/{arg_a:INT}/".to_string(),
            arg_b_endpoint_name: "secondary".to_string(),
            arg_b_endpoint_path: "/{arg_a:BOOL}/".to_string(),
        }
    );
}

#[test]
fn test_http_path_segment_arg_page_diff_names() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/{arg_a:INT}/",
        http_method: GET,
      },
      {
        http_endpoint_name: secondary,
        data_type: html,
        path: "/{arg_b:INT}/",
        http_method: GET,
      },
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::AppHttpTreeErrorArgPageSegmentMultipleNames {
            application_name: "hello-world".to_string(),
            arg_a_endpoint_name: "primary".to_string(),
            arg_a_endpoint_path: "/{arg_a:INT}/".to_string(),
            arg_b_endpoint_name: "secondary".to_string(),
            arg_b_endpoint_path: "/{arg_b:INT}/".to_string(),
        }
    );
}

#[test]
fn test_http_path_segment_arg_page_take_twice() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/{arg_a:INT}",
        http_method: GET,
      },
      {
        http_endpoint_name: secondary,
        data_type: html,
        path: "/{arg_a:INT}",
        http_method: GET,
      },
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::AppHttpTreeErrorArgPageTakenTwice {
            application_name: "hello-world".to_string(),
            arg_a_endpoint_name: "primary".to_string(),
            arg_a_endpoint_path: "/{arg_a:INT}".to_string(),
            arg_b_endpoint_name: "secondary".to_string(),
            arg_b_endpoint_path: "/{arg_a:INT}".to_string(),
            page_method: "GET".to_string(),
        }
    );
}

#[test]
fn test_http_path_segment_arg_page_different_http_methods() {
    // should work as different POST methods should have different http trees
    let _ = common::assert_platform_validation_success(
        r#"
DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot {
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/{arg_a:INT}",
        http_method: GET,
      },
      {
        http_endpoint_name: secondary,
        data_type: json,
        path: "/{arg_a:INT}",
        http_method: POST,
        input_body_type: test_type,
        output_body_type: test_type,
      },
      {
        http_endpoint_name: third,
        data_type: json,
        path: "/{arg_a:INT}",
        http_method: PUT,
        input_body_type: test_type,
        output_body_type: test_type,
      },
    ]
  }
]
"#,
    );
}

#[test]
fn test_http_duplicate_arg_name() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/endpoint_a/{arg_1:INT}",
        http_method: GET,
      },
      {
        http_endpoint_name: secondary,
        data_type: html,
        path: "/endpoint_a/{other_arg:TEXT}",
        http_method: GET,
      },
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::AppHttpTreeErrorArgPageSegmentMultipleNames {
            application_name: "hello-world".to_string(),
            arg_a_endpoint_name: "primary".to_string(),
            arg_a_endpoint_path: "/endpoint_a/{arg_1:INT}".to_string(),
            arg_b_endpoint_name: "secondary".to_string(),
            arg_b_endpoint_path: "/endpoint_a/{other_arg:TEXT}".to_string(),
        }
    );
}

#[test]
fn test_http_endpoint_non_snake_case() {
    let err = common::assert_eden_db_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: pri-mary,
        data_type: html,
        path: "/mookie",
        http_method: GET,
      }
    ]
  }
]
"#,
    );

    match err {
        DatabaseValidationError::LuaCheckEvaluationFailed { .. } => {}
        _ => {
            panic!("Wrong error: {:?}", err)
        }
    }
}

#[test]
fn test_http_reserved_argument_name() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/endpoint_a/{input_body:INT}",
        http_method: GET,
      },
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::HttpPathReservedArgumentName {
            full_path: "/endpoint_a/{input_body:INT}".to_string(),
            reserved_arg_name: "input_body".to_string(),
        }
    );
}

#[test]
fn test_http_multiple_arguments_for_path() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/endpoint_a/{arg_1:INT[]}",
        http_method: GET,
      },
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::HttpMultipleArgumentsOnlyAllowedInQuery {
            bad_argument_name: "arg_1".to_string(),
            full_path: "/endpoint_a/{arg_1:INT[]}".to_string(),
        }
    );
}

#[test]
fn test_application_deployment_ingress_bad_mountpoint() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/endpoint_a",
        http_method: GET,
      },
    ]
  }
]

DATA STRUCT backend_application_deployment_ingress {
    deployment: test-depl,
    mountpoint: '/bad-mountpoint',
    subdomain: '',
    tld: epl-infra.net,
    endpoint_list: '
      primary
    ',
}

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    nats1, exclusive, 4k;
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    nats1, exclusive, 4k;
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    nats1, exclusive, 4k;
    minio, exclusive, 1M;
    mon, exclusive, 4k;
    am, exclusive, 4k;
  };
  server-d, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.13;
  } WITH server_root_volume(volume_name, intended_usage, zfs_recordsize) {
    minio, exclusive, 1M;
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT loki_cluster {
  cluster_name: default,
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
  WITH alertmanager_instance [
    { instance_id: 1, alertmanager_server: server-a=>am },
    { instance_id: 2, alertmanager_server: server-b=>am },
    { instance_id: 3, alertmanager_server: server-c=>am },
  ]
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: us-west,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio,
    },
    {
      instance_id: 4,
      instance_volume: server-d=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: docker, },
    { bucket_name: logging, },
    { bucket_name: tempo, },
  ]
}
"#,
    );
    assert_eq!(err, PlatformValidationError::AppIngressInvalidMountpoint {
      deployment: "test-depl".to_string(),
      application_name: "hello-world".to_string(),
      mountpoint: "/bad-mountpoint".to_string(),
      explanation: "Mountpoint must start and end with a slash and its path segments must be alphanumeric symbols",
    });
}

#[test]
fn test_application_deployment_ingress_non_existing_endpoint() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/endpoint_a",
        http_method: GET,
      },
      {
        http_endpoint_name: secondary,
        data_type: html,
        path: "/endpoint_b",
        http_method: GET,
      },
    ]
  }
]

DATA STRUCT backend_application_deployment_ingress {
    deployment: test-depl,
    mountpoint: '/good/',
    subdomain: '',
    tld: epl-infra.net,
    endpoint_list: '
      non_existing
    ',
}

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume {
    nats1;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::AppIngressEndpointNotFound {
            deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            non_existing_endpoint: "non_existing".to_string(),
            endpoints_list: "
      non_existing
    "
            .to_string(),
            available_endpoints: "primary\nsecondary".to_string(),
        }
    );
}

#[test]
fn test_application_deployment_ingress_duplicate_endpoint() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/endpoint_a",
        http_method: GET,
      },
      {
        http_endpoint_name: secondary,
        data_type: html,
        path: "/endpoint_b",
        http_method: GET,
      },
    ]
  }
]

DATA STRUCT backend_application_deployment_ingress {
    deployment: test-depl,
    mountpoint: '/good/',
    subdomain: '',
    tld: epl-infra.net,
    endpoint_list: '
      primary
      primary
    ',
}

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume {
    nats1;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume {
    nats1;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume {
    nats1;
  };
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::AppIngressDuplicateEndpointInEndpointList {
            deployment: "test-depl".to_string(),
            application_name: "hello-world".to_string(),
            duplicate_endpoint: "primary".to_string(),
            endpoints_list: "
      primary
      primary
    "
            .to_string(),
        }
    );
}

#[test]
fn test_application_deployment_raw_data_type_but_input_data_type_exists() {
    assert_eq!(
        PlatformValidationError::HttpEndpointDataTypeIsRawButInputBodyTypeIsSpecified {
            application_name: "hello-world".to_string(),
            endpoint_name: "primary".to_string(),
            full_path: "/endpoint_a".to_string(),
            http_method: "POST".to_string(),
            input_body_type: "test_type".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: raw,
        input_body_type: test_type,
        path: "/endpoint_a",
        http_method: POST,
      },
    ]
  }
]

DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot {
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }
}

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume(volume_name, zfs_recordsize) {
    nats1, 4k;
    minio, 1M;
    mon, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, zfs_recordsize) {
    nats1, 4k;
    minio, 1M;
    mon, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, zfs_recordsize) {
    nats1, 4k;
    minio, 1M;
    mon, 4k;
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT loki_cluster {
  cluster_name: default,
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: us-west,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: docker, },
    { bucket_name: logging, },
    { bucket_name: tempo, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_deployment_raw_data_type_but_output_data_type_exists() {
    assert_eq!(
        PlatformValidationError::HttpEndpointDataTypeIsRawButOutputBodyTypeIsSpecified {
            application_name: "hello-world".to_string(),
            endpoint_name: "primary".to_string(),
            full_path: "/endpoint_a".to_string(),
            http_method: "POST".to_string(),
            output_body_type: "test_type".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: raw,
        output_body_type: test_type,
        path: "/endpoint_a",
        http_method: POST,
      },
    ]
  }
]

DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot {
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }
}

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume(volume_name, zfs_recordsize) {
    nats1, 4k;
    minio, 1M;
    mon, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, zfs_recordsize) {
    nats1, 4k;
    minio, 1M;
    mon, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, zfs_recordsize) {
    nats1, 4k;
    minio, 1M;
    mon, 4k;
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT loki_cluster {
  cluster_name: default,
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: us-west,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: docker, },
    { bucket_name: logging, },
    { bucket_name: tempo, },
  ]
}
"#,
        )
    );
}

#[test]
fn test_application_deployment_expects_stream_but_not_raw_data_type() {
    assert_eq!(
        PlatformValidationError::HttpEndpointReceiveBodyAsStreamIsOnlySupportedForRawEndpointsWithRawInputBodyType {
            application_name: "hello-world".to_string(),
            endpoint_name: "primary".to_string(),
            full_path: "/endpoint_a".to_string(),
            http_method: "POST".to_string(),
            input_body_type: "test_type".to_string(),
            endpoint_data_type: "json".to_string(),
            expected_endpoint_data_type: "raw".to_string(),
            expected_input_body_type: "raw".to_string(),
        },
        common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT backend_application_deployment {
  deployment_name: test-depl,
  application_name: hello-world,
}

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        input_body_type: test_type,
        output_body_type: test_type,
        path: "/endpoint_a",
        receive_body_as_stream: true,
        http_method: POST,
      },
    ]
  }
]

DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot {
        version: 7,
        snapshot_source: "{
            bozo @0 :String,
        }"
    }
}

DATA STRUCT network [
  {
    network_name: lan,
    cidr: '10.0.0.0/8',
  }
]

DATA subnet_router_floating_ip {
  '10.17.0.2/24';
}

DATA server(hostname, ssh_interface, is_consul_master, is_nomad_master, is_vault_instance, is_vpn_gateway) {
  server-a, eth0, true, true, false, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.10;
  } WITH server_root_volume(volume_name, zfs_recordsize) {
    nats1, 4k;
    minio, 1M;
    mon, 4k;
  };
  server-b, eth0, true, false, true, true WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.11;
  } WITH server_root_volume(volume_name, zfs_recordsize) {
    nats1, 4k;
    minio, 1M;
    mon, 4k;
  };
  server-c, eth0, true, false, true, false WITH server_disk(disk_id) {
    'vda';
  } WITH network_interface {
    eth0, lan, 10.17.0.12;
  } WITH server_root_volume(volume_name, zfs_recordsize) {
    nats1, 4k;
    minio, 1M;
    mon, 4k;
  };
}

DATA STRUCT docker_registry_instance {
  region: us-west,
  minio_bucket: 'us-west=>docker',
}

DATA STRUCT loki_cluster {
  cluster_name: default,
  storage_bucket: us-west=>logging,
}

DATA STRUCT monitoring_cluster {
  cluster_name: default,
  WITH monitoring_instance [
    { instance_id: 1, monitoring_server: server-a=>mon },
    { instance_id: 2, monitoring_server: server-b=>mon },
    { instance_id: 3, monitoring_server: server-c=>mon },
  ]
}

DATA STRUCT tempo_cluster {
  region: us-west,
  cluster_name: r1-tempo,
  storage_bucket: us-west=>tempo,
}

DATA STRUCT minio_cluster {
  cluster_name: us-west,
  WITH minio_instance [
    {
      instance_id: 1,
      instance_volume: server-a=>minio,
    },
    {
      instance_id: 2,
      instance_volume: server-b=>minio,
    },
    {
      instance_id: 3,
      instance_volume: server-c=>minio,
    },
  ]
  WITH minio_bucket [
    { bucket_name: docker, },
    { bucket_name: logging, },
    { bucket_name: tempo, },
  ]
}
"#,
        )
    );
}
