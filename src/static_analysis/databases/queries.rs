use std::collections::{BTreeMap, HashMap};

use nom::bytes::complete::take_while1;
use nom::{combinator::opt, sequence::tuple, Parser};
use nom::character::complete::{char, multispace0};

pub type TestTablesData = BTreeMap<String, Vec<BTreeMap<String, String>>>;

pub type IResult<I, O, E = nom::error::VerboseError<I>> = Result<(I, O), nom::Err<E>>;

pub fn parse_query_argument(input: &str) -> IResult<&str, (&str, Option<&str>, usize)> {
    let mut ws_count = 0;
    let (tail, (_, w1, vname, w2, vtype, _)) = tuple((
        char('{'),
        multispace0,
        valid_variable_name,
        multispace0,
        opt(tuple((char(':'), multispace0, valid_type_name, multispace0))),
        char('}'),
    ))
    .parse(input)?;

    ws_count += w1.len();
    ws_count += w2.len();

    let vtype = vtype.map(|(_, w3, vt, w4)| {
        ws_count += w3.len();
        ws_count += w4.len();
        vt
    });

    Ok((tail, (vname, vtype, ws_count)))
}

pub fn valid_variable_name(input: &str) -> IResult<&str, &str> {
    let (tail, tname) = take_while1(|c: char| c.is_alphanumeric() || c == '_').parse(input)?;

    Ok((tail, tname))
}

pub fn valid_type_name(input: &str) -> IResult<&str, &str> {
    let (tail, tname) = take_while1(|c: char| c.is_alphanumeric()).parse(input)?;

    Ok((tail, tname))
}

pub fn deserialize_test_dataset(
    input_data: &str,
) -> Result<TestTablesData, serde_yaml::Error> {
    serde_yaml::from_str::<TestTablesData>(input_data)
}

pub fn deserialize_test_arguments(
    input_data: &str,
) -> Result<HashMap<String, String>, serde_yaml::Error> {
    serde_yaml::from_str::<HashMap<String, String>>(input_data)
}

pub fn deserialize_test_output(
    input_data: &str,
) -> Result<Vec<HashMap<String, String>>, serde_yaml::Error> {
    serde_yaml::from_str::<Vec<HashMap<String, String>>>(input_data)
}

pub fn rows_to_table(rows: &Vec<Vec<String>>) -> String {
    use prettytable::Table;
    let mut table = Table::new();

    for r in rows {
        let mut cells = Vec::with_capacity(r.len());
        for c in r {
            cells.push(prettytable::Cell::new(c));
        }
        let row = prettytable::Row::new(cells);
        table.add_row(row);
    }

    table.to_string()
}

#[test]
fn test_parse_deserialize_test_arguments() {
    assert_eq!(
        deserialize_test_arguments("{hello: 1, bois: hey}").unwrap(),
        HashMap::from_iter([
            ("hello".to_string(), "1".to_string()),
            ("bois".to_string(), "hey".to_string()),
        ])
    )
}

#[test]
fn test_parse_deserialize_test_arguments_fail() {
    assert!(deserialize_test_arguments("[]").is_err());
}

#[test]
fn test_parse_deserialize_test_outputs() {
    assert_eq!(
        deserialize_test_output("[{hello: 1, bois: hey}, {thic: boi}]").unwrap(),
        vec![
            HashMap::from_iter([
                ("hello".to_string(), "1".to_string()),
                ("bois".to_string(), "hey".to_string()),
            ]),
            HashMap::from_iter([("thic".to_string(), "boi".to_string()),]),
        ]
    )
}

#[test]
fn test_parse_deserialize_test_outputs_fail() {
    assert!(deserialize_test_output("{hello: 1, bois: hey}").is_err());
}


#[test]
fn test_deserialize_test_dataset() {
    let res = deserialize_test_dataset(
        r#"
some_table:
- hello: werld
  nice: key
- you: 1
  have: true
  false: y
other_table:
  - yo: sup
"#,
    ).unwrap();

    let expected = BTreeMap::from_iter([
        (
            "some_table".to_string(),
            vec![
                BTreeMap::from_iter([
                    ("hello".to_string(), "werld".to_string()),
                    ("nice".to_string(), "key".to_string()),
                ]),
                BTreeMap::from_iter([
                    ("you".to_string(), "1".to_string()),
                    ("have".to_string(), "true".to_string()),
                    ("false".to_string(), "y".to_string()),
                ]),
            ],
        ),
        (
            "other_table".to_string(),
            vec![BTreeMap::from_iter([("yo".to_string(), "sup".to_string())])],
        ),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn test_fail_deserialize_test_dataset_1() {
    let res = deserialize_test_dataset(
        r#"
- hello: werld
  nice: key
"#,
    );

    assert!(res.is_err());
}

#[test]
fn test_fail_deserialize_test_dataset_2() {
    let res = deserialize_test_dataset(r#"- invalid: schema"#);

    assert!(res.is_err());
}
