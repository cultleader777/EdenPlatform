use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::io::ErrorKind;

use super::{PlatformValidationError, bench_start};
use crate::database::{Database, TableRowPointerAlert};
use crate::prom_metrics_dump::{PromSeriesDatabase, AllClusterSeriesDatabase, OutputPromMetric, merge_databases};
use crate::static_analysis::TableRowPointerAlertTriggerTest;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AlertTestSeries {
    series: String,
    values: String,
}

pub struct AlertTestCompiled {
    series: Vec<AlertTestSeries>,
    expected_labels: BTreeMap<String, String>,
    all_labels: BTreeMap<String, String>,
}

pub fn verify_alert_trigger_test_series(
    db: &Database,
    sdb: &Option<AllClusterSeriesDatabase>,
) -> Result<BTreeMap<TableRowPointerAlertTriggerTest, AlertTestCompiled>, PlatformValidationError>
{
    let mut res = BTreeMap::new();
    for a in db.alert().rows_iter() {
        if db.alert().c_children_alert_trigger_test(a).is_empty() {
            return Err(
                PlatformValidationError::AlertMustHaveAtLeastOneTriggerTest {
                    testless_alert: db.alert().c_alert_name(a).clone(),
                },
            );
        }
    }

    let mut alerts_databases: BTreeMap<TableRowPointerAlert, &PromSeriesDatabase> = BTreeMap::new();
    let mut merged_db: BTreeMap<String, OutputPromMetric> = BTreeMap::new();

    if let Some(sdb) = sdb.as_ref() {
        for mon_g in db.monitoring_cluster_alert_group().rows_iter() {
            let mon_c = db.monitoring_cluster_alert_group().c_parent(mon_g);
            let mon_c_name = db.monitoring_cluster().c_cluster_name(mon_c);
            let alert_group = db.monitoring_cluster_alert_group().c_alert_group_name(mon_g);
            let this_db = sdb.get(mon_c_name).unwrap();

            for alert in db.alert_group().c_children_alert(alert_group) {
                if !alerts_databases.contains_key(&alert) {
                    alerts_databases.insert(*alert, this_db);
                }
            }
        }


        if alerts_databases.len() < db.alert().len() {
            for existing_db in sdb.values() {
                merge_databases(&mut merged_db, existing_db);
            }

            for alert in db.alert().rows_iter() {
                if !alerts_databases.contains_key(&alert) {
                    alerts_databases.insert(alert, &merged_db);
                }
            }
        }
    }

    for i in db.alert_trigger_test().rows_iter() {
        let alert = db.alert_trigger_test().c_parent(i);
        let sdb = alerts_databases.get(&alert);
        let alert_name = db.alert().c_alert_name(alert);
        let alert_expr = db.alert().c_expr(alert);
        let test_expected_message = db.alert_trigger_test().c_expected_message(i);
        let mut expected_labels = BTreeMap::new();

        let deser: Vec<AlertTestSeries> =
            serde_yaml::from_str(db.alert_trigger_test().c_input_series(i).as_str()).map_err(
                |e| PlatformValidationError::AlertTriggerTestCantParseSeries {
                    error: e.to_string(),
                    input_data: db.alert_trigger_test().c_input_series(i).clone(),
                    alert_name: alert_name.clone(),
                    test_expected_message: test_expected_message.clone(),
                },
            )?;

        let labels_str = db.alert_trigger_test().c_expected_labels(i);
        if !labels_str.is_empty() {
            expected_labels = serde_yaml::from_str(labels_str).map_err(|e| {
                PlatformValidationError::AlertTriggerTestCantParseExpectedLabels {
                    error: e.to_string(),
                    input_data: labels_str.clone(),
                    alert_name: alert_name.clone(),
                    test_expected_message: test_expected_message.clone(),
                }
            })?;
        }

        let mut all_labels = BTreeMap::new();
        for i in &deser {
            let (_, series) = parser::parse_prometheus_series(&i.series).map_err(|e| {
                PlatformValidationError::AlertTriggerTestCantParseSeriesExpression {
                    input_data: i.series.clone(),
                    error: e.to_string(),
                    alert_name: alert_name.clone(),
                    test_expected_message: test_expected_message.clone(),
                }
            })?;

            // alert expression must include all series
            if !check_expression_contains_series(alert_expr, &series.name) {
                return Err(
                    PlatformValidationError::AlertTriggerTestSeriesIsNotFoundInExpression {
                        alert_name: alert_name.clone(),
                        alert_expression: alert_expr.clone(),
                        series_not_found: series.name,
                        series_expression: i.series.clone(),
                        input_data: i.values.clone(),
                        test_expected_message: test_expected_message.clone(),
                    },
                );
            }

            let mut uniq_labels: HashSet<String> = HashSet::new();
            for (lk, _) in &series.labels {
                if !uniq_labels.insert(lk.clone()) {
                    return Err(
                        PlatformValidationError::AlertTriggerTestSeriesDuplicateLabel {
                            input_data: i.series.clone(),
                            duplicate_label_name: lk.clone(),
                            alert_name: alert_name.clone(),
                            test_expected_message: test_expected_message.clone(),
                        },
                    );
                }
            }

            // TODO: in the future maybe somehow validate validity of label value?
            if let Some(sdb) = sdb {
                match sdb.get(&series.name) {
                    None =>
                    {
                        return Err(
                            PlatformValidationError::AlertTriggerTestMetricDoesntExistInDb {
                                series_expression: i.series.clone(),
                                missing_metric: series.name.clone(),
                                alert_name: alert_name.clone(),
                                test_expected_message: test_expected_message.clone(),
                            },
                        );
                    }
                    Some(db_metric) => {
                        for (lk, lv) in &series.labels {
                            if db_metric.labels.get(lk).is_none() {
                                return Err(PlatformValidationError::AlertTriggerTestMetricLabelDoesntExistInDb {
                                    series_expression: i.series.clone(),
                                    metric: series.name.clone(),
                                    missing_label: lk.clone(),
                                    existing_labels: db_metric.labels.iter().map(|(i, _)| i.clone()).collect::<Vec<_>>(),
                                    alert_name: alert_name.clone(),
                                    test_expected_message: test_expected_message.clone(),
                                });
                            }

                            let _ = all_labels.insert(lk.clone(), lv.clone());
                        }
                    }
                }
            }
        }

        if sdb.is_some() {
            let compiled = AlertTestCompiled {
                series: deser,
                expected_labels,
                all_labels,
            };

            assert!(res.insert(i, compiled).is_none());
        }
    }

    Ok(res)
}

fn check_expression_contains_series(expr: &str, series_name: &str) -> bool {
    let combinations = [
        format!("[^a-zA-Z0-9_]{}[^a-zA-Z0-9_]", series_name),
        format!("^{}[^a-zA-Z0-9_]", series_name),
        format!("[^a-zA-Z0-9_]{}$", series_name),
        format!("^{}$", series_name),
    ]
    .map(|i| regex::Regex::new(&i).expect("Invalid regex"));

    for comb in combinations {
        if comb.is_match(expr) {
            return true;
        }
    }

    false
}

#[test]
fn test_expression_contains_series() {
    assert!(check_expression_contains_series("up", "up"));
    assert!(check_expression_contains_series("up ", "up"));
    assert!(check_expression_contains_series(" up ", "up"));
    assert!(check_expression_contains_series(" up", "up"));

    assert!(!check_expression_contains_series("up", "u"));
    assert!(!check_expression_contains_series(" up", "u"));
    assert!(!check_expression_contains_series("up ", "u"));
    assert!(!check_expression_contains_series(" up ", "u"));

    assert!(check_expression_contains_series("up{}", "up"));
    assert!(check_expression_contains_series("up{} ", "up"));
    assert!(check_expression_contains_series(" up{} ", "up"));
    assert!(check_expression_contains_series(" up{}", "up"));
}

pub struct PromtoolTestSuite {
    pub rules_file: String,
    pub tests_file: String,
    pub test_count: usize,
}

pub fn generate_promtool_tests(
    db: &Database,
    proj: &BTreeMap<TableRowPointerAlertTriggerTest, AlertTestCompiled>,
) -> PromtoolTestSuite {
    let mut res = PromtoolTestSuite {
        rules_file: String::new(),
        tests_file: String::new(),
        test_count: proj.len(),
    };

    if proj.is_empty() {
        return res;
    }

    res.rules_file += r#"
groups:
- name: Alerts
  rules:
"#;
    for a in db.alert().rows_iter() {
        res.rules_file += &generate_alert_rule(db, a);
    }

    res.tests_file += r#"
rule_files:
- alert_rules.yml

tests:
"#;

    for at in db.alert_trigger_test().rows_iter() {
        let alert = db.alert_trigger_test().c_parent(at);
        let alert_name = db.alert().c_alert_name(alert);
        res.tests_file += "- interval: ";
        res.tests_file += db.alert_trigger_test().c_interval(at);
        res.tests_file += "\n";

        res.tests_file += "  input_series:\n";

        let proj = proj.get(&at).unwrap();
        for s in &proj.series {
            res.tests_file += "    - series: '";
            res.tests_file += &s.series;
            res.tests_file += "'\n";
            res.tests_file += "      values: '";
            res.tests_file += &s.values;
            res.tests_file += "'\n";
        }

        res.tests_file += "  alert_rule_test:\n";
        res.tests_file += "    - eval_time: ";
        res.tests_file += db.alert_trigger_test().c_eval_time(at);
        res.tests_file += "\n";
        res.tests_file += "      alertname: ";
        res.tests_file += alert_name;
        res.tests_file += "\n";
        res.tests_file += "      exp_alerts:\n";
        res.tests_file += "        - exp_labels:\n";
        res.tests_file += &format!("            severity: {}\n", db.alert().c_severity(alert));
        let expected_labels = if proj.expected_labels.is_empty() {
            // inferred labels
            &proj.all_labels
        } else {
            // explicit labels
            &proj.expected_labels
        };
        for (k, v) in expected_labels {
            assert!(!v.contains('\"'));
            res.tests_file += &format!("            {}: \"{}\"\n", k, v);
        }
        res.tests_file += "          exp_annotations:\n";
        res.tests_file += "            description: \"";
        res.tests_file += db.alert_trigger_test().c_expected_message(at);
        res.tests_file += "\"\n";
    }

    res
}

pub fn generate_alert_rule(db: &Database, a: TableRowPointerAlert) -> String {
    let time = db.alert().c_for_time(a);
    let expr = db.alert().c_expr(a).replace('\n', " ");
    let expr = expr.trim();
    let name = db.alert().c_alert_name(a);
    let severity = db.alert().c_severity(a);
    let description = db.alert().c_description(a);
    format!(
        r#"
  - alert: {name}
    expr: {expr}
    for: {time}
    labels:
      severity: {severity}
    annotations:
      description: "{description}"
"#
    )
}

pub fn run_sandboxed_promtool_tests(
    db: &Database,
    suite: &PromtoolTestSuite,
) -> Result<(), PlatformValidationError> {
    if db.alert_trigger_test().len() == 0 || suite.test_count == 0 {
        return Ok(());
    }

    let tdir = tempfile::tempdir().expect("Cannot create temp dir for testing promtool tests");
    std::fs::write(tdir.path().join("alert_rules.yml"), &suite.rules_file)
        .expect("Can't write alert rules file");
    std::fs::write(tdir.path().join("tests.yml"), &suite.tests_file)
        .expect("Can't write tests file");

    let output = std::process::Command::new("promtool")
        .current_dir(tdir.path())
        .arg("test")
        .arg("rules")
        .arg("tests.yml")
        .output()
        .expect("Cannot run shell command");

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr).expect("Can't promtool parse output?");
        return Err(PlatformValidationError::AlertTriggerTestsFailed { output: stderr });
    }

    Ok(())
}

pub fn try_read_prometheus_series_databases() -> Result<Option<AllClusterSeriesDatabase>, PlatformValidationError> {
    let b = bench_start("Metrics db read and deserialize");
    // allow overriding for tezting
    let filename = std::env::var("EPL_METRICS_DB").unwrap_or_else(|_| "metrics_db.yml".to_string());
    match std::fs::read(&filename) {
        Ok(bytes) => {
            let res: AllClusterSeriesDatabase =
                serde_yaml::from_slice(bytes.as_slice()).map_err(|e| {
                    PlatformValidationError::CantParsePrometheusSeriesDatabase {
                        filename,
                        error: e.to_string(),
                    }
                })?;
            b.end();
            return Ok(Some(res));
        }
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                eprintln!("WARNING: Metrics database not found at [{filename}], make sure it is initialized with 'make refresh-metrics-db' to test that your alerts are working against your actual infrastructure.");
                return Ok(None);
            } else {
                return Err(PlatformValidationError::CantReadPrometheusSeriesDatabase {
                    filename,
                    error: e.to_string(),
                });
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct PrometheusSeries {
    name: String,
    labels: Vec<(String, String)>,
}

mod parser {
    use nom::Parser;
    use nom::{
        bytes::complete::take_while1,
        character::complete::{char, multispace0},
        combinator::opt,
        error::{VerboseError, VerboseErrorKind},
        multi::separated_list0,
        sequence::tuple,
    };

    use super::PrometheusSeries;

    pub type IResult<I, O, E = nom::error::VerboseError<I>> = Result<(I, O), nom::Err<E>>;

    pub(super) fn parse_prometheus_series(input: &str) -> IResult<&str, PrometheusSeries> {
        let (tail, btype) = tuple((
            multispace0,
            valid_field_name,
            multispace0,
            opt(tuple((
                char('{'),
                multispace0,
                separated_list0(
                    tuple((multispace0, char(','), multispace0)),
                    tuple((
                        valid_field_name,
                        multispace0,
                        char('='),
                        multispace0,
                        char('"'),
                        take_while1(|c: char| c != '"'),
                        char('"'),
                    )),
                ),
                multispace0,
                char('}'),
            ))),
            multispace0,
        ))
        .parse(input)?;

        if !tail.is_empty() {
            return Err(nom::Err::Error(VerboseError {
                errors: vec![(
                    tail,
                    VerboseErrorKind::Context(
                        "Bad input, nothing extra should be left after parsing prometheus metric",
                    ),
                )],
            }));
        }

        let (_, name, _, opt_labels, _) = btype;

        let mut res = PrometheusSeries {
            name: name.to_string(),
            labels: Vec::new(),
        };

        if let Some((_, _, labels, _, _)) = opt_labels {
            for (label_name, _, _, _, _, label_value, _) in labels {
                res.labels
                    .push((label_name.to_string(), label_value.to_string()));
            }
        }

        Ok((tail, res))
    }

    fn valid_field_name(input: &str) -> IResult<&str, &str> {
        let (tail, tname) = take_while1(|c: char| c.is_alphanumeric() || c == '_').parse(input)?;

        Ok((tail, tname))
    }

    #[test]
    fn test_parse_prometheus_series_simple() {
        assert_eq!(
            parse_prometheus_series(" henlo "),
            Ok((
                "",
                PrometheusSeries {
                    name: "henlo".to_string(),
                    labels: vec![],
                }
            ))
        );
        assert_eq!(
            parse_prometheus_series("henlo "),
            Ok((
                "",
                PrometheusSeries {
                    name: "henlo".to_string(),
                    labels: vec![],
                }
            ))
        );
        assert_eq!(
            parse_prometheus_series("henlo"),
            Ok((
                "",
                PrometheusSeries {
                    name: "henlo".to_string(),
                    labels: vec![],
                }
            ))
        );
        assert_eq!(
            parse_prometheus_series(" henlo"),
            Ok((
                "",
                PrometheusSeries {
                    name: "henlo".to_string(),
                    labels: vec![],
                }
            ))
        );
        assert_eq!(
            parse_prometheus_series(" henlo{}"),
            Ok((
                "",
                PrometheusSeries {
                    name: "henlo".to_string(),
                    labels: vec![],
                }
            ))
        );
        assert_eq!(
            parse_prometheus_series(" henlo {}"),
            Ok((
                "",
                PrometheusSeries {
                    name: "henlo".to_string(),
                    labels: vec![],
                }
            ))
        );
        assert_eq!(
            parse_prometheus_series(" henlo {} "),
            Ok((
                "",
                PrometheusSeries {
                    name: "henlo".to_string(),
                    labels: vec![],
                }
            ))
        );
    }

    #[test]
    fn test_parse_prometheus_series_with_labels() {
        assert_eq!(
            parse_prometheus_series(" henlo {a=\"123\"}"),
            Ok((
                "",
                PrometheusSeries {
                    name: "henlo".to_string(),
                    labels: vec![("a".to_string(), "123".to_string())],
                }
            ))
        );
        assert_eq!(
            parse_prometheus_series(" henlo {a=\"123\" , a=\"321\"}"),
            Ok((
                "",
                PrometheusSeries {
                    name: "henlo".to_string(),
                    labels: vec![
                        ("a".to_string(), "123".to_string()),
                        ("a".to_string(), "321".to_string()),
                    ],
                }
            ))
        );
        assert_eq!(
            parse_prometheus_series(" henlo { a=\"123\",other=\"foo\" }"),
            Ok((
                "",
                PrometheusSeries {
                    name: "henlo".to_string(),
                    labels: vec![
                        ("a".to_string(), "123".to_string()),
                        ("other".to_string(), "foo".to_string()),
                    ],
                }
            ))
        );
    }

    #[test]
    fn test_parse_prometheus_series_errors() {
        println!("{:?}", parse_prometheus_series(" henlo[] "));
        assert!(parse_prometheus_series(" henlo[] ").is_err());
        assert!(parse_prometheus_series(" henlo {_name_ = 123} ").is_err());
        assert!(parse_prometheus_series(" henlo {_name_ \\= \"\"} ").is_err());
        assert!(parse_prometheus_series(" henlo {_name_ : \"\"} ").is_err());
        assert!(parse_prometheus_series(" henlo {_name_ = \"\"; other = \"meow\"} ").is_err());
    }
}
