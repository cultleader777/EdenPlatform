#[cfg(test)]
use crate::static_analysis::PlatformValidationError;

#[cfg(test)]
use super::common;

#[test]
fn test_alert_tests_invalid_input_series_syntax() {
    temp_env::with_var_unset("EPL_METRICS_DB", || {
        let err = common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: HelloAlerts,
        expr: 'up',
        description: 'whatever',
        WITH alert_trigger_test [
            {
                expected_message: whatever,
                eval_time: 10m,
                input_series: '
                    this is invalid and unexpected yaml
                '
            }
        ]
    }
  ]
}
"#,
        );
        assert_eq!(err, PlatformValidationError::AlertTriggerTestCantParseSeries {
            alert_name: "HelloAlerts".to_string(),
            test_expected_message: "whatever".to_string(),
            error: "invalid type: string \"this is invalid and unexpected yaml\", expected a sequence at line 2 column 21".to_string(),
            input_data: "
                    this is invalid and unexpected yaml
                ".to_string()
        });
    });
}

#[test]
fn test_alert_unknown_field() {
    temp_env::with_var_unset("EPL_METRICS_DB", || {
        let err = common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: HelloAlerts,
        expr: 'up',
        description: 'whatever',
        WITH alert_trigger_test [
            {
                expected_message: whatever,
                eval_time: 10m,
                input_series: '
                    - series: "invalid{series;format}"
                      values: "10+10x7"
                      unknown_field: 777
                '
            }
        ]
    }
  ]
}
"#,
        );
        assert_eq!(err, PlatformValidationError::AlertTriggerTestCantParseSeries {
            alert_name: "HelloAlerts".to_string(),
            test_expected_message: "whatever".to_string(),
            input_data: "\n                    - series: \"invalid{series;format}\"\n                      values: \"10+10x7\"\n                      unknown_field: 777\n                ".to_string(),
            error: ".[0]: unknown field `unknown_field`, expected `series` or `values` at line 4 column 23".to_string(),
        });
    });
}

#[test]
fn test_alert_tests_not_found_series() {
    temp_env::with_var("EPL_METRICS_DB", Some("misc/good-metrics-db.yml"), || {
        let err = common::assert_platform_validation_error_wcustom_data(
            r#"

DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: HelloAlerts,
        expr: 'this_series_is_not_a_thing',
        description: 'whatever',
        WITH alert_trigger_test [
            {
                expected_message: whatever,
                eval_time: 10m,
                input_series: '
                    - series: "this_series_is_not_a_thing"
                      values: "10+10x7"
                '
            }
        ]
    }
  ]
}
"#,
        );
        assert_eq!(err, PlatformValidationError::AlertTriggerTestMetricDoesntExistInDb {
            alert_name: "HelloAlerts".to_string(),
            missing_metric: "this_series_is_not_a_thing".to_string(),
            series_expression: "this_series_is_not_a_thing".to_string(),
            test_expected_message: "whatever".to_string(),
        });
    });
}

#[test]
fn test_alert_tests_bad_schema_syntax() {
    temp_env::with_var("EPL_METRICS_DB", Some("misc/bad-metrics-db.yml"), || {
        let err = common::assert_platform_validation_error_wcustom_data(
            r#"
"#,
        );
        assert_eq!(err, PlatformValidationError::CantParsePrometheusSeriesDatabase {
            filename: "misc/bad-metrics-db.yml".to_string(),
            error: "invalid type: string \"this schema is totally wrong for metrics db file\", expected a map".to_string(),
        });
    });
}

#[test]
fn test_alert_tests_invalid_input_series_name_syntax() {
    temp_env::with_var_unset("EPL_METRICS_DB", || {
        let err = common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: HelloAlerts,
        expr: 'up',
        description: 'whatever',
        WITH alert_trigger_test [
            {
                expected_message: whatever,
                eval_time: 10m,
                input_series: '
                    - series: "invalid{series;format}"
                      values: "10+10x7"
                '
            }
        ]
    }
  ]
}
"#,
        );
        assert_eq!(err, PlatformValidationError::AlertTriggerTestCantParseSeriesExpression {
            alert_name: "HelloAlerts".to_string(),
            test_expected_message: "whatever".to_string(),
            input_data: "invalid{series;format}".to_string(),
            error: "Parsing Error: VerboseError { errors: [(\"{series;format}\", Context(\"Bad input, nothing extra should be left after parsing prometheus metric\"))] }".to_string(),
        });
    });
}

#[test]
fn test_alert_tests_invalid_input_series_duplicate_label() {
    temp_env::with_var_unset("EPL_METRICS_DB", || {
        let err = common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: HelloAlerts,
        expr: 'invalid',
        description: 'whatever',
        WITH alert_trigger_test [
            {
                expected_message: whatever,
                eval_time: 10m,
                input_series: '
                    - series: invalid{a="some",a="other"}
                      values: 10+10x7
                '
            }
        ]
    }
  ]
}
"#,
        );
        assert_eq!(
            err,
            PlatformValidationError::AlertTriggerTestSeriesDuplicateLabel {
                alert_name: "HelloAlerts".to_string(),
                test_expected_message: "whatever".to_string(),
                input_data: "invalid{a=\"some\",a=\"other\"}".to_string(),
                duplicate_label_name: "a".to_string()
            }
        );
    });
}

#[test]
fn test_alert_non_existing_series() {
    temp_env::with_var("EPL_METRICS_DB", Some("misc/good-metrics-db.yml"), || {

        let err = common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: HelloAlerts,
        expr: 'non_existing_series',
        description: 'whatever',
        WITH alert_trigger_test [
            {
                expected_message: whatever,
                eval_time: 10m,
                input_series: '
                    - series: non_existing_series{a="some"}
                      values: 10+10x7
                '
            }
        ]
    }
  ]
}
"#,
        );
        assert_eq!(
            err,
            PlatformValidationError::AlertTriggerTestMetricDoesntExistInDb {
                alert_name: "HelloAlerts".to_string(),
                test_expected_message: "whatever".to_string(),
                series_expression: "non_existing_series{a=\"some\"}".to_string(),
                missing_metric: "non_existing_series".to_string(),
            }
        );
    });
}

#[test]
fn test_alert_non_existing_label() {
    temp_env::with_var("EPL_METRICS_DB", Some("misc/good-metrics-db.yml"), || {

        let err = common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: HelloAlerts,
        expr: 'node_cpu_seconds_total',
        description: 'whatever',
        WITH alert_trigger_test [
            {
                expected_message: whatever,
                eval_time: 10m,
                input_series: '
                    - series: node_cpu_seconds_total{unicorn_label="some"}
                      values: 10+10x7
                '
            }
        ]
    }
  ]
}
"#,
        );
        assert_eq!(
            err,
            PlatformValidationError::AlertTriggerTestMetricLabelDoesntExistInDb {
                alert_name: "HelloAlerts".to_string(),
                test_expected_message: "whatever".to_string(),
                series_expression: "node_cpu_seconds_total{unicorn_label=\"some\"}".to_string(),
                metric: "node_cpu_seconds_total".to_string(),
                missing_label: "unicorn_label".to_string(),
                existing_labels: vec![
                    "cpu".to_string(),
                    "instance".to_string(),
                    "job".to_string(),
                    "mode".to_string()
                ],
            }
        );
    });
}

#[test]
fn test_disallow_testless_alerts() {
    temp_env::with_var_unset("EPL_METRICS_DB", || {
        let err = common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: HelloAlerts,
        expr: 'up',
        description: 'whatever',
    }
  ]
}
"#,
        );
        assert_eq!(
            err,
            PlatformValidationError::AlertMustHaveAtLeastOneTriggerTest {
                testless_alert: "HelloAlerts".to_string(),
            }
        );
    });
}

#[test]
fn test_alert_invalid_labels() {
    temp_env::with_var_unset("EPL_METRICS_DB", || {
        let err = common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: HelloAlerts,
        expr: 'up',
        description: 'whatever',
        WITH alert_trigger_test [
            {
                expected_message: whatever,
                expected_labels: '
                  invalid labels yaml
                ',
                eval_time: 10m,
                input_series: '
                    - series: "good"
                      values: "10+10x7"
                '
            }
        ]
    }
  ]
}
"#,
        );
        assert_eq!(
            err,
            PlatformValidationError::AlertTriggerTestCantParseExpectedLabels {
                alert_name: "HelloAlerts".to_string(),
                test_expected_message: "whatever".to_string(),
                input_data: "
                  invalid labels yaml
                "
                    .to_string(),
                error:
                "invalid type: string \"invalid labels yaml\", expected a map at line 2 column 19"
                    .to_string(),
            }
        );
    });
}

#[test]
fn test_alert_invalid_syntax() {
    temp_env::with_var("EPL_METRICS_DB", Some("misc/good-metrics-db.yml"), || {
        let err = common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: InvalidSyntaxAlert,
        expr: 'round(node_cpu_seconds_total',
        description: 'whatever',
        WITH alert_trigger_test [
            {
                expected_message: whatever,
                eval_time: 10m,
                input_series: '
                    - series: node_cpu_seconds_total{job="some"}
                      values: "10+10x7"
                '
            }
        ]
    }
  ]
}
"#,
        );
        assert_eq!(err, PlatformValidationError::AlertTriggerTestsFailed {
            output: "  FAILED:\nalert_rules.yml: 7:11: group \"Alerts\", rule 1, \"InvalidSyntaxAlert\": could not parse expression: 1:29: parse error: unclosed left parenthesis\n".to_string(),
        });
    });
}

#[test]
fn test_alert_test_didnt_trigger() {
    temp_env::with_var("EPL_METRICS_DB", Some("misc/good-metrics-db.yml"), || {

        let err = common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: FilesystemSpaceLow,
        expr: '
          round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100
          / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
        ',
        description: 'Filesystem {{ $labels.device }} at {{ $labels.instance }} has less than 20% disk space remaining',
        WITH alert_trigger_test [
            {
                expected_message: 'Filesystem /mookie at some-server:9090 has less than 20% disk space remaining',
                eval_time: 10m,
                input_series: '
                    - series: node_filesystem_free_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 8 8 8 8 8 8 8 8 8 8
                    - series: node_filesystem_size_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 9 9 9 9 9 9 9 9 9 9
                '
            }
        ]
    }
  ]
}
"#,
        );
        assert_eq!(err, PlatformValidationError::AlertTriggerTestsFailed {
            output: "  FAILED:\n    alertname: FilesystemSpaceLow, time: 10m, \n        exp:[\n            0:\n              Labels:{alertname=\"FilesystemSpaceLow\", device=\"/mookie\", fstype=\"zfs\", instance=\"some-server:9090\", severity=\"50\"}\n              Annotations:{description=\"Filesystem /mookie at some-server:9090 has less than 20% disk space remaining\"}\n            ], \n        got:[]\n".to_string(),
        });
    });
}

#[test]
fn test_alert_did_trigger() {
    temp_env::with_var("EPL_METRICS_DB", Some("misc/good-metrics-db.yml"), || {

        let _ = common::assert_platform_validation_success(
            r#"
DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: FilesystemSpaceLow,
        expr: '
          round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100
          / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
        ',
        description: 'Filesystem {{ $labels.device }} at {{ $labels.instance }} has less than 20% disk space remaining',
        WITH alert_trigger_test [
            {
                expected_message: 'Filesystem /mookie at some-server:9090 has less than 20% disk space remaining',
                eval_time: 10m,
                input_series: '
                    - series: node_filesystem_free_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 1 1 1 1 1 1 1 1 1 1
                    - series: node_filesystem_size_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 9 9 9 9 9 9 9 9 9 9
                '
            }
        ]
    }
  ]
}
"#,
        );
    });
}

#[test]
fn test_alert_explicit_labels_needed_fail() {
    temp_env::with_var("EPL_METRICS_DB", Some("misc/good-metrics-db.yml"), || {

        let err = common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: FilesystemSpaceLow,
        expr: '
          max by (instance) (
            round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100
            / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
          )
        ',
        description: 'Filesystem {{ $labels.device }} at {{ $labels.instance }} has less than 20% disk space remaining',
        WITH alert_trigger_test [
            {
                expected_message: 'Filesystem /mookie at some-server:9090 has less than 20% disk space remaining',
                eval_time: 10m,
                input_series: '
                    - series: node_filesystem_free_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 1 1 1 1 1 1 1 1 1 1
                    - series: node_filesystem_size_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 9 9 9 9 9 9 9 9 9 9
                '
            }
        ]
    }
  ]
}
"#,
        );
        assert_eq!(err, PlatformValidationError::AlertTriggerTestsFailed {
            output: "  FAILED:\n    alertname: FilesystemSpaceLow, time: 10m, \n        exp:[\n            0:\n              Labels:{alertname=\"FilesystemSpaceLow\", device=\"/mookie\", fstype=\"zfs\", instance=\"some-server:9090\", severity=\"50\"}\n              Annotations:{description=\"Filesystem /mookie at some-server:9090 has less than 20% disk space remaining\"}\n            ], \n        got:[\n            0:\n              Labels:{alertname=\"FilesystemSpaceLow\", instance=\"some-server:9090\", severity=\"50\"}\n              Annotations:{description=\"Filesystem  at some-server:9090 has less than 20% disk space remaining\"}\n            ]\n".to_string(),
        });
    });
}

#[test]
fn test_alert_series_are_not_in_expression() {
    temp_env::with_var_unset("EPL_METRICS_DB", || {
        let err = common::assert_platform_validation_error_wcustom_data(
            r#"
DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: FilesystemSpaceLow,
        expr: 'up',
        description: 'Filesystem {{ $labels.device }} at {{ $labels.instance }} has less than 20% disk space remaining',
        WITH alert_trigger_test [
            {
                expected_message: 'Filesystem /mookie at some-server:9090 has less than 20% disk space remaining',
                eval_time: 10m,
                input_series: '
                    - series: node_filesystem_free_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 1 1 1 1 1 1 1 1 1 1
                '
            }
        ]
    }
  ]
}
"#,
        );
        assert_eq!(err, PlatformValidationError::AlertTriggerTestSeriesIsNotFoundInExpression {
            alert_name: "FilesystemSpaceLow".to_string(),
            alert_expression: "up".to_string(),
            series_not_found: "node_filesystem_free_bytes".to_string(),
            series_expression: "node_filesystem_free_bytes{device=\"/mookie\",instance=\"some-server:9090\",fstype=\"zfs\"}".to_string(),
            input_data: "1 1 1 1 1 1 1 1 1 1".to_string(),
            test_expected_message: "Filesystem /mookie at some-server:9090 has less than 20% disk space remaining".to_string(),
        });
    });
}

#[test]
fn test_alert_explicit_labels_needed_success() {
    temp_env::with_var("EPL_METRICS_DB", Some("misc/good-metrics-db.yml"), || {
        let _ = common::assert_platform_validation_success(
            r#"
DATA STRUCT alert_group {
  alert_group_name: Default
  WITH alert [
    {
        alert_name: FilesystemSpaceLow,
        expr: '
          max by (instance) (
            round((node_filesystem_free_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"} * 100
            / node_filesystem_size_bytes{fstype=~"(btrfs|zfs|xfs|ext4)"}), 0.1) < 20
          )
        ',
        description: 'Filesystem {{ $labels.device }} at {{ $labels.instance }} has less than 20% disk space remaining',
        WITH alert_trigger_test [
            {
                expected_message: 'Filesystem  at some-server:9090 has less than 20% disk space remaining',
                expected_labels: '
                  instance: some-server:9090
                ',
                eval_time: 10m,
                input_series: '
                    - series: node_filesystem_free_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 1 1 1 1 1 1 1 1 1 1
                    - series: node_filesystem_size_bytes{device="/mookie",instance="some-server:9090",fstype="zfs"}
                      values: 9 9 9 9 9 9 9 9 9 9
                '
            }
        ]
    }
  ]
}
"#,
        );
    });
}
