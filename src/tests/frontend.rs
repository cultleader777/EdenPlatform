#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::common;

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn test_frontend_path_validation_error() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "totally_bad_path",
      }
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::HttpCorePathMustStartWithSlash {
            full_path: "totally_bad_path".to_string(),
        }
    );
}

#[test]
fn test_frontend_path_duplicate_pages() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
      {
        page_name: home2,
        path: "/",
      },
    ]
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::AppHttpTreeErrorDuplicatePagePath {
            application_name: "hello-frontend".to_string(),
            previous_endpoint_name: "home".to_string(),
            previous_endpoint_path: "/".to_string(),
            duplicate_endpoint_name: "home2".to_string(),
            duplicate_endpoint_path: "/".to_string(),
            page_method: "GET".to_string(),
        },
    );
}

#[test]
fn test_frontend_application_without_pages() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationHasNoPages {
            application_name: "hello-frontend".to_string(),
        }
    );
}

#[test]
fn test_frontend_app_multiple_ingress() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: test,
  }
]

DATA STRUCT frontend_application_deployment_ingress [
  {
    deployment: test,
    mountpoint: '/',
    tld: epl-infra.net,
    subdomain: www,
  },
  {
    deployment: test,
    mountpoint: '/other/',
    tld: epl-infra.net,
    subdomain: www,
  },
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationDeploymentHasMoreThanOneIngress {
            application_name: "hello-frontend".to_string(),
            deployment_name: "test".to_string(),
            maximum_allowed: 1,
            actual: 2,
        },
    );
}

#[test]
fn test_frontend_app_has_api_but_no_ingress() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_used_endpoint [
      {
        endpoint_name: test,
        backend_endpoint: hello-world=>primary,
      }
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: test,
  }
]

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/henlo/boi",
        http_method: GET,
        output_body_type: test_type,
      }
    ]
  }
]

DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 1,
        snapshot_source: "{
            output @0 :String,
        }"
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationDeploymentHasNoIngress {
            application_name: "hello-frontend".to_string(),
            deployment_name: "test".to_string(),
            ingress_count: 0,
        },
    );
}

#[test]
fn test_frontend_app_has_no_used_endpoints() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_used_endpoint [
      {
        endpoint_name: test_missing,
        backend_endpoint: hello-world=>primary,
      }
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: test,
  }
]

DATA STRUCT frontend_application_deployment_ingress [
  {
    deployment: test,
    mountpoint: "/",
    subdomain: www,
    tld: epl-infra.net,
  }
]

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/henlo/boi",
        http_method: GET,
        output_body_type: test_type,
      }
    ]
  }
]

DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 1,
        snapshot_source: "{
            output @0 :String,
        }"
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationUsedEndpointNotDeployed {
            application_name: "hello-frontend".to_string(),
            deployment_name: "test".to_string(),
            missing_endpoint_name: "test_missing".to_string(),
            missing_endpoint_backend_name: "hello-world".to_string(),
            missing_endpoint_backend_signature: "/henlo/boi".to_string(),
        },
    );
}

#[test]
fn test_frontend_app_has_ambigous_endpoints() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_used_endpoint [
      {
        endpoint_name: test_ambigous,
        backend_endpoint: hello-world=>primary,
      }
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: test,
  }
]

DATA STRUCT frontend_application_deployment_ingress [
  {
    deployment: test,
    mountpoint: "/",
    subdomain: www,
    tld: epl-infra.net,
  }
]

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/henlo/boi",
        http_method: GET,
        output_body_type: test_type,
      }
    ]
  }
]

DATA STRUCT backend_application_deployment [
  {
    deployment_name: backend1,
    application_name: hello-world,
  }
]

DATA STRUCT backend_application_deployment_ingress [
  {
    deployment: backend1,
    mountpoint: "/ambigous-a/",
    subdomain: www,
    tld: epl-infra.net,
    endpoint_list: '
      primary
    '
  },
  {
    deployment: backend1,
    mountpoint: "/ambigous-b/",
    subdomain: www,
    tld: epl-infra.net,
    endpoint_list: '
      primary
    '
  },
]

DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 1,
        snapshot_source: "{
            output @0 :String,
        }"
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationUsedEndpointAmbigiousIngress {
            application_name: "hello-frontend".to_string(),
            deployment_name: "test".to_string(),
            ambigous_endpoint_name: "test_ambigous".to_string(),
            ambigous_endpoint_backend_name: "hello-world".to_string(),
            ambigous_endpoint_backend_signature: "/henlo/boi".to_string(),
            tld: "epl-infra.net".to_string(),
            subdomain: "www".to_string(),
            matching_ingress_mountpoints: vec![
                "/ambigous-a/".to_string(),
                "/ambigous-b/".to_string()
            ],
            matching_ingress_deployment_names: vec!["backend1".to_string(), "backend1".to_string()],
        },
    );
}

#[test]
fn test_frontend_app_bad_endpoint_wiring() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_used_endpoint [
      {
        endpoint_name: test_ambigous,
        backend_endpoint: hello-world=>primary,
      }
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: test,
    explicit_endpoint_wiring: '
      bad syntax
    ',
  }
]

DATA STRUCT frontend_application_deployment_ingress [
  {
    deployment: test,
    mountpoint: "/",
    subdomain: www,
    tld: epl-infra.net,
  }
]

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/henlo/boi",
        http_method: GET,
        output_body_type: test_type,
      }
    ]
  }
]

DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 1,
        snapshot_source: "{
            output @0 :String,
        }"
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationEndpointWiringInvalidLine {
          application_name: "hello-frontend".to_string(),
          deployment_name: "test".to_string(),
          invalid_line: "bad syntax".to_string(),
          endpoint_wiring: "
      bad syntax
    ".to_string(),
          explanation: "Example wiring 'frontend_endpoint: backend_deployment_ingress=>backend_endpoint_name'".to_string()
        },
    );
}

#[test]
fn test_frontend_app_wiring_non_existing_endpoint() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_used_endpoint [
      {
        endpoint_name: test_ambigous,
        backend_endpoint: hello-world=>primary,
      }
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: test,
    explicit_endpoint_wiring: '
      test_non_existing: hmm
    ',
  }
]

DATA STRUCT frontend_application_deployment_ingress [
  {
    deployment: test,
    mountpoint: "/",
    subdomain: www,
    tld: epl-infra.net,
  }
]

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/henlo/boi",
        http_method: GET,
        output_body_type: test_type,
      }
    ]
  }
]

DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 1,
        snapshot_source: "{
            output @0 :String,
        }"
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationEndpointWiringNonExistingUsedEndpoint {
            application_name: "hello-frontend".to_string(),
            deployment_name: "test".to_string(),
            non_existing_endpoint: "test_non_existing".to_string(),
            valid_candidate_endpoints: vec!["test_ambigous".to_string()],
            endpoint_wiring: "
      test_non_existing: hmm
    "
            .to_string(),
        },
    );
}

#[test]
fn test_frontend_app_wiring_non_existing_backend_deployment() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_used_endpoint [
      {
        endpoint_name: test_existing,
        backend_endpoint: hello-world=>primary,
      }
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: test,
    explicit_endpoint_wiring: '
      test_existing: hmm
    ',
  }
]

DATA STRUCT frontend_application_deployment_ingress [
  {
    deployment: test,
    mountpoint: "/",
    subdomain: www,
    tld: epl-infra.net,
  }
]

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/henlo/boi",
        http_method: GET,
        output_body_type: test_type,
      }
    ]
  }
]

DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 1,
        snapshot_source: "{
            output @0 :String,
        }"
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationEndpointWiringNonExistingBackendDeployment {
            application_name: "hello-frontend".to_string(),
            deployment_name: "test".to_string(),
            non_existing_deployment: "hmm".to_string(),
            endpoint_wiring: "
      test_existing: hmm
    "
            .to_string(),
        },
    );
}

#[test]
fn test_frontend_app_wiring_incorrect_backend_application() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_used_endpoint [
      {
        endpoint_name: test_existing,
        backend_endpoint: hello-world=>primary,
      }
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: test,
    explicit_endpoint_wiring: '
      test_existing: test-depl2
    ',
  }
]

DATA STRUCT frontend_application_deployment_ingress [
  {
    deployment: test,
    mountpoint: "/",
    subdomain: www,
    tld: epl-infra.net,
  }
]

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/henlo/boi",
        http_method: GET,
        output_body_type: test_type,
      }
    ]
  },
  {
    application_name: other,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/henlo/boi",
        http_method: GET,
        output_body_type: test_type,
      }
    ]
  }
]

DATA STRUCT backend_application_deployment [
  {
    deployment_name: test-depl,
    application_name: hello-world,
  },
  {
    deployment_name: test-depl2,
    application_name: other,
  },
]

DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 1,
        snapshot_source: "{
            output @0 :String,
        }"
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationEndpointWiringIncorrectTargetApplication {
            application_name: "hello-frontend".to_string(),
            deployment_name: "test".to_string(),
            expected_backend_application: "hello-world".to_string(),
            actual_backend_application: "other".to_string(),
            endpoint_wiring_line: "test_existing: test-depl2".to_string(),
            endpoint_wiring: "
      test_existing: test-depl2
    "
            .to_string(),
        },
    );
}

#[test]
fn test_frontend_app_wiring_backend_app_has_no_ingress() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_used_endpoint [
      {
        endpoint_name: test_existing,
        backend_endpoint: hello-world=>primary,
      }
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: test,
    explicit_endpoint_wiring: '
      test_existing: test-depl
    ',
  }
]

DATA STRUCT frontend_application_deployment_ingress [
  {
    deployment: test,
    mountpoint: "/",
    subdomain: www,
    tld: epl-infra.net,
  }
]

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/henlo/boi",
        http_method: GET,
        output_body_type: test_type,
      }
    ]
  },
]

DATA STRUCT backend_application_deployment [
  {
    deployment_name: test-depl,
    application_name: hello-world,
  },
]

DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 1,
        snapshot_source: "{
            output @0 :String,
        }"
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationEndpointWiringTargetDeploymentHasNoIngress {
            application_name: "hello-frontend".to_string(),
            deployment_name: "test".to_string(),
            backend_application_deployment_without_ingress: "test-depl".to_string(),
            endpoint_wiring_line: "test_existing: test-depl".to_string(),
            endpoint_wiring: "
      test_existing: test-depl
    "
            .to_string(),
        },
    );
}

#[test]
fn test_frontend_app_wiring_backend_app_has_ingress_in_wrong_domain() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_used_endpoint [
      {
        endpoint_name: test_existing,
        backend_endpoint: hello-world=>primary,
      }
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: test,
    explicit_endpoint_wiring: '
      test_existing: test-depl
    ',
  }
]

DATA STRUCT frontend_application_deployment_ingress [
  {
    deployment: test,
    mountpoint: "/",
    subdomain: www,
    tld: epl-infra.net,
  }
]

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/henlo/boi",
        http_method: GET,
        output_body_type: test_type,
      }
    ]
  },
]

DATA STRUCT backend_application_deployment [
  {
    deployment_name: test-depl,
    application_name: hello-world,
  },
]

DATA STRUCT backend_application_deployment_ingress [
  {
    deployment: test-depl,
    mountpoint: "/",
    subdomain: henlo,
    tld: epl-infra.net,
  }
]

DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 1,
        snapshot_source: "{
            output @0 :String,
        }"
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationEndpointWiringCantFindCompatibleIngress {
          application_name: "hello-frontend".to_string(),
          deployment_name: "test".to_string(),
          explanation: "Can't find reachable backend endpoint from frontend application. Make sure frontend app is on the same subdomain and TLD not to violate CORS constraints.".to_string(),
          endpoint_wiring_line: "test_existing: test-depl".to_string(),
          endpoint_wiring: "
      test_existing: test-depl
    ".to_string(),
        },
    );
}

#[test]
fn test_frontend_app_wiring_endpoint_defined_twice() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_used_endpoint [
      {
        endpoint_name: test_existing,
        backend_endpoint: hello-world=>primary,
      }
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: test,
    explicit_endpoint_wiring: '
      test_existing: test-depl
      test_existing: hmm
    ',
  }
]

DATA STRUCT frontend_application_deployment_ingress [
  {
    deployment: test,
    mountpoint: "/",
    subdomain: www,
    tld: epl-infra.net,
  }
]

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/henlo/boi",
        http_method: GET,
        output_body_type: test_type,
      }
    ]
  },
]

DATA STRUCT backend_application_deployment [
  {
    deployment_name: test-depl,
    application_name: hello-world,
  },
]

DATA STRUCT backend_application_deployment_ingress [
  {
    deployment: test-depl,
    mountpoint: "/",
    subdomain: www,
    tld: epl-infra.net,
  }
]

DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 1,
        snapshot_source: "{
            output @0 :String,
        }"
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationEndpointWiringEndpointDefinedTwice {
            application_name: "hello-frontend".to_string(),
            deployment_name: "test".to_string(),
            endpoint_defined_twice: "test_existing".to_string(),
            endpoint_wiring: "
      test_existing: test-depl
      test_existing: hmm
    "
            .to_string(),
        },
    );
}

#[test]
fn test_frontend_app_cant_fetch_non_json() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_used_endpoint [
      {
        endpoint_name: test_existing,
        backend_endpoint: hello-world=>primary,
      }
    ]
  }
]

DATA STRUCT backend_application [
  {
    application_name: hello-world,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/henlo/boi",
        http_method: GET,
      }
    ]
  },
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationEndpointDisallowedDataType {
            frontend_application_name: "hello-frontend".to_string(),
            backend_application_name: "hello-world".to_string(),
            backend_endpoint_name: "primary".to_string(),
            backend_endpoint_data_type: "html".to_string(),
            backend_endpoint_allowed_types: vec!["json".to_string()],
            backend_endpoint_path: "/henlo/boi".to_string(),
        },
    );
}

#[test]
fn test_frontend_app_page_link_wiring_unknown_page() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_external_page [
      {
        link_name: test_existing,
        frontend_page: target-frontend=>page,
      }
    ]
  },
  {
    application_name: target-frontend,
    WITH frontend_page [
      {
        page_name: page,
        path: "/page",
      },
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: source,
    page_wiring: '
        unknown_page: target-frontend
    ',
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: www,
        tld: epl-infra.net,
    }
  },
  {
    application_name: target-frontend,
    deployment_name: target,
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: other,
        tld: epl-infra.net,
    }
  },
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationPageWiringUnknownPage {
            application_name: "hello-frontend".to_string(),
            deployment_name: "source".to_string(),
            invalid_line: "unknown_page: target-frontend".to_string(),
            page_wiring: "
        unknown_page: target-frontend
    "
            .to_string(),
            non_existing_page: "unknown_page".to_string(),
            valid_link_pages: vec!["test_existing".to_string()],
        },
    );
}

#[test]
fn test_frontend_app_page_link_wiring_deployment() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_external_page [
      {
        link_name: test_existing,
        frontend_page: target-frontend=>page,
      }
    ]
  },
  {
    application_name: target-frontend,
    WITH frontend_page [
      {
        page_name: page,
        path: "/page",
      },
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: source,
    page_wiring: '
        test_existing: unknown_deployment
    ',
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: www,
        tld: epl-infra.net,
    }
  },
  {
    application_name: target-frontend,
    deployment_name: target,
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: other,
        tld: epl-infra.net,
    }
  },
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationPageWiringUnknownFrontendDeployment {
            application_name: "hello-frontend".to_string(),
            deployment_name: "source".to_string(),
            invalid_line: "test_existing: unknown_deployment".to_string(),
            page_wiring: "
        test_existing: unknown_deployment
    "
            .to_string(),
            non_existing_frontend_deployment: "unknown_deployment".to_string(),
        },
    );
}

#[test]
fn test_frontend_app_page_link_wiring_same_deployment() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_external_page [
      {
        link_name: test_existing,
        frontend_page: target-frontend=>page,
      }
    ]
  },
  {
    application_name: target-frontend,
    WITH frontend_page [
      {
        page_name: page,
        path: "/page",
      },
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: source,
    page_wiring: '
        test_existing: source
    ',
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: www,
        tld: epl-infra.net,
    }
  },
  {
    application_name: target-frontend,
    deployment_name: target,
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: other,
        tld: epl-infra.net,
    }
  },
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationPageWiringPointsToTheSameDeployment {
            application_name: "hello-frontend".to_string(),
            deployment_name: "source".to_string(),
            invalid_line: "test_existing: source".to_string(),
            page_wiring: "
        test_existing: source
    "
            .to_string(),
            same_frontend_deployment: "source".to_string(),
        },
    );
}

#[test]
fn test_frontend_app_page_link_wiring_bad_target_app() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_external_page [
      {
        link_name: test_existing,
        frontend_page: target-frontend=>page,
      }
    ]
  },
  {
    application_name: target-frontend,
    WITH frontend_page [
      {
        page_name: page,
        path: "/page",
      },
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: source,
    page_wiring: '
        test_existing: target
    ',
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: www,
        tld: epl-infra.net,
    }
  },
  {
    application_name: hello-frontend,
    deployment_name: target,
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: other,
        tld: epl-infra.net,
    }
  },
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationPageWiringPointsToUnexpectedFrontendApp {
            application_name: "hello-frontend".to_string(),
            deployment_name: "source".to_string(),
            invalid_line: "test_existing: target".to_string(),
            page_wiring: "
        test_existing: target
    "
            .to_string(),
            expected_frontend_app: "target-frontend".to_string(),
            actual_frontend_app: "hello-frontend".to_string(),
        },
    );
}

#[test]
fn test_frontend_app_page_link_wiring_no_ingress() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_external_page [
      {
        link_name: test_existing,
        frontend_page: target-frontend=>page,
      }
    ]
  },
  {
    application_name: target-frontend,
    WITH frontend_page [
      {
        page_name: page,
        path: "/page",
      },
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: source,
    page_wiring: '
        test_existing: target
    ',
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: www,
        tld: epl-infra.net,
    }
  },
  {
    application_name: target-frontend,
    deployment_name: target
  },
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationPageWiringTargetDeploymentHasNoIngress {
            application_name: "hello-frontend".to_string(),
            deployment_name: "source".to_string(),
            invalid_line: "test_existing: target".to_string(),
            page_wiring: "
        test_existing: target
    "
            .to_string(),
            frontend_application_deployment_without_ingress: "target".to_string(),
        },
    );
}

#[test]
fn test_frontend_app_page_link_wiring_defined_multiple_times() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_external_page [
      {
        link_name: test_existing,
        frontend_page: target-frontend=>page,
      }
    ]
  },
  {
    application_name: target-frontend,
    WITH frontend_page [
      {
        page_name: page,
        path: "/page",
      },
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: source,
    page_wiring: '
        test_existing: target
        test_existing: target
    ',
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: www,
        tld: epl-infra.net,
    }
  },
  {
    application_name: target-frontend,
    deployment_name: target
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: other,
        tld: epl-infra.net,
    }
  },
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationPageWiringLinkDefinedMultipleTimes {
            application_name: "hello-frontend".to_string(),
            deployment_name: "source".to_string(),
            invalid_line: "test_existing: target".to_string(),
            page_wiring: "
        test_existing: target
        test_existing: target
    "
            .to_string(),
            page_defined_multiple_times: "test_existing".to_string(),
        },
    );
}

#[test]
fn test_frontend_app_page_link_wiring_bad_syntax() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_external_page [
      {
        link_name: test_existing,
        frontend_page: target-frontend=>page,
      }
    ]
  },
  {
    application_name: target-frontend,
    WITH frontend_page [
      {
        page_name: page,
        path: "/page",
      },
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: source,
    page_wiring: '
        invalid wiring syntax
    ',
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: www,
        tld: epl-infra.net,
    }
  },
  {
    application_name: target-frontend,
    deployment_name: target
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: other,
        tld: epl-infra.net,
    }
  },
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationPageWiringInvalidLine {
            application_name: "hello-frontend".to_string(),
            deployment_name: "source".to_string(),
            invalid_line: "invalid wiring syntax".to_string(),
            page_wiring: "
        invalid wiring syntax
    "
            .to_string(),
            explanation: "Valid wiring example: `page_name: frontend_deployment`".to_string(),
        },
    );
}

#[test]
fn test_frontend_app_page_link_wiring_undefined() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_external_page [
      {
        link_name: test_existing,
        frontend_page: target-frontend=>page,
      }
    ]
  },
  {
    application_name: target-frontend,
    WITH frontend_page [
      {
        page_name: page,
        path: "/page",
      },
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: source,
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: www,
        tld: epl-infra.net,
    }
  },
  {
    application_name: target-frontend,
    deployment_name: target
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: other,
        tld: epl-infra.net,
    }
  },
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationPageWiringExternalPageUndefined {
            application_name: "hello-frontend".to_string(),
            deployment_name: "source".to_string(),
            page_wiring: "".to_string(),
            undefined_page: "test_existing".to_string(),
        },
    );
}

#[test]
fn test_frontend_app_backend_link_wiring_bad_data_type() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_external_link [
      {
        link_name: test_html,
        backend_endpoint: target-app=>primary,
      }
    ]
  }
]

DATA STRUCT backend_application [
  {
    application_name: target-app,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: json,
        path: "/target-html",
        http_method: GET,
        output_body_type: test_type,
      }
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: source,
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: www,
        tld: epl-infra.net,
    }
  },
]

DATA STRUCT versioned_type {
    type_name: test_type WITH versioned_type_snapshot [{
        version: 1,
        snapshot_source: "{
            output @0 :String,
        }"
    }]
}
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationLinkWiringBadEndpointDataType {
            application_name: "hello-frontend".to_string(),
            bad_endpoint: "test_html".to_string(),
            allowed_endpoint_types: vec!["html".to_string()],
            bad_endpoint_data_type: "json".to_string(),
        },
    );
}

#[test]
fn test_frontend_app_backend_link_wiring_bad_syntax() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_external_link [
      {
        link_name: test_html,
        backend_endpoint: target-app=>primary,
      }
    ]
  }
]

DATA STRUCT backend_application [
  {
    application_name: target-app,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/target-html",
        http_method: GET,
      }
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: source,
    link_wiring: '
      invalid syntax
    ',
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: www,
        tld: epl-infra.net,
    }
  },
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationLinkWiringInvalidLine {
            application_name: "hello-frontend".to_string(),
            deployment_name: "source".to_string(),
            invalid_line: "invalid syntax".to_string(),
            explanation: "Valid wiring example: `link_name: backend_deployment`".to_string(),
            link_wiring: "
      invalid syntax
    "
            .to_string(),
        },
    );
}

#[test]
fn test_frontend_app_backend_link_wiring_unknown_endpoint() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_external_link [
      {
        link_name: test_html,
        backend_endpoint: target-app=>primary,
      }
    ]
  }
]

DATA STRUCT backend_application [
  {
    application_name: target-app,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/target-html",
        http_method: GET,
      }
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: source,
    link_wiring: '
      unknown_endpoint: hmmm
    ',
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: www,
        tld: epl-infra.net,
    }
  },
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationLinkWiringUnknownEndpoint {
            application_name: "hello-frontend".to_string(),
            deployment_name: "source".to_string(),
            non_existing_endpoint: "unknown_endpoint".to_string(),
            invalid_line: "unknown_endpoint: hmmm".to_string(),
            valid_endpoints: vec!["test_html".to_string()],
            link_wiring: "
      unknown_endpoint: hmmm
    "
            .to_string(),
        },
    );
}

#[test]
fn test_frontend_app_backend_link_wiring_unknown_backend_deployment() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_external_link [
      {
        link_name: test_html,
        backend_endpoint: target-app=>primary,
      }
    ]
  }
]

DATA STRUCT backend_application [
  {
    application_name: target-app,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/target-html",
        http_method: GET,
      }
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: source,
    link_wiring: '
      test_html: hmmm
    ',
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: www,
        tld: epl-infra.net,
    }
  },
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationLinkWiringUnknownBackendDeployment {
            application_name: "hello-frontend".to_string(),
            deployment_name: "source".to_string(),
            invalid_line: "test_html: hmmm".to_string(),
            non_existing_backend_deployment: "hmmm".to_string(),
            link_wiring: "
      test_html: hmmm
    "
            .to_string(),
        },
    );
}

#[test]
fn test_frontend_app_backend_link_wiring_wrong_backend_app() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_external_link [
      {
        link_name: test_html,
        backend_endpoint: target-app=>primary,
      }
    ]
  }
]

DATA STRUCT backend_application [
  {
    application_name: target-app,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/target-html",
        http_method: GET,
      }
    ],
  },
  {
    application_name: wrong-app,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/target-html",
        http_method: GET,
      }
    ]
  }
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: source,
    link_wiring: '
      test_html: wrong-depl
    ',
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: www,
        tld: epl-infra.net,
    }
  },
]

DATA STRUCT backend_application_deployment [
  {
    deployment_name: wrong-depl,
    application_name: wrong-app,
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationLinkWiringPointsToUnexpectedBackendApp {
            application_name: "hello-frontend".to_string(),
            deployment_name: "source".to_string(),
            invalid_line: "test_html: wrong-depl".to_string(),
            actual_backend_app: "wrong-app".to_string(),
            expected_backend_app: "target-app".to_string(),
            link_wiring: "
      test_html: wrong-depl
    "
            .to_string(),
        },
    );
}

#[test]
fn test_frontend_app_backend_link_wiring_no_backend_ingress() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_external_link [
      {
        link_name: test_html,
        backend_endpoint: target-app=>primary,
      }
    ]
  }
]

DATA STRUCT backend_application [
  {
    application_name: target-app,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/target-html",
        http_method: GET,
      }
    ],
  },
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: source,
    link_wiring: '
      test_html: target-depl
    ',
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: www,
        tld: epl-infra.net,
    }
  },
]

DATA STRUCT backend_application_deployment [
  {
    deployment_name: target-depl,
    application_name: target-app,
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationLinkWiringBackendDeploymentHasNoIngress {
            application_name: "hello-frontend".to_string(),
            deployment_name: "source".to_string(),
            invalid_line: "test_html: target-depl".to_string(),
            backend_application_deployment_without_ingress: "target-depl".to_string(),
            link_wiring: "
      test_html: target-depl
    "
            .to_string(),
        },
    );
}

#[test]
fn test_frontend_app_backend_link_wiring_defined_multiple_times() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_external_link [
      {
        link_name: test_html,
        backend_endpoint: target-app=>primary,
      }
    ]
  }
]

DATA STRUCT backend_application [
  {
    application_name: target-app,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/target-html",
        http_method: GET,
      }
    ],
  },
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: source,
    link_wiring: '
      test_html: target-depl
      test_html: target-depl
    ',
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: www,
        tld: epl-infra.net,
    }
  },
]

DATA STRUCT backend_application_deployment [
  {
    deployment_name: target-depl,
    application_name: target-app
    WITH backend_application_deployment_ingress {
      mountpoint: '/',
      subdomain: backend,
      tld: epl-infra.net,
    }
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationLinkWiringLinkDefinedMultipleTimes {
            application_name: "hello-frontend".to_string(),
            deployment_name: "source".to_string(),
            invalid_line: "test_html: target-depl".to_string(),
            link_defined_multiple_times: "test_html".to_string(),
            link_wiring: "
      test_html: target-depl
      test_html: target-depl
    "
            .to_string(),
        },
    );
}

#[test]
fn test_frontend_app_backend_link_wiring_undefined() {
    let err = common::assert_platform_validation_error_wcustom_data(
        r#"
DATA STRUCT frontend_application [
  {
    application_name: hello-frontend,
    WITH frontend_page [
      {
        page_name: home,
        path: "/",
      },
    ] WITH frontend_application_external_link [
      {
        link_name: test_html,
        backend_endpoint: target-app=>primary,
      }
    ]
  }
]

DATA STRUCT backend_application [
  {
    application_name: target-app,
    WITH backend_http_endpoint [
      {
        http_endpoint_name: primary,
        data_type: html,
        path: "/target-html",
        http_method: GET,
      }
    ],
  },
]

DATA STRUCT frontend_application_deployment [
  {
    application_name: hello-frontend,
    deployment_name: source,
    link_wiring: '',
    WITH frontend_application_deployment_ingress {
        mountpoint: '/',
        subdomain: www,
        tld: epl-infra.net,
    }
  },
]

DATA STRUCT backend_application_deployment [
  {
    deployment_name: target-depl,
    application_name: target-app
    WITH backend_application_deployment_ingress {
      mountpoint: '/',
      subdomain: backend,
      tld: epl-infra.net,
    }
  }
]
"#,
    );
    assert_eq!(
        err,
        PlatformValidationError::FrontendApplicationLinkWiringExternalLinkUndefined {
            application_name: "hello-frontend".to_string(),
            deployment_name: "source".to_string(),
            undefined_link: "test_html".to_string(),
            link_wiring: "".to_string(),
        },
    );
}
