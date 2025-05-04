use std::hash::Hash;

use edendb::db_parser::take_until_unbalanced;
use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_while1},
    character::complete::{char, digit1, multispace0, multispace1, none_of, one_of},
    combinator::{cut, opt, all_consuming},
    multi::{many0, many1, many_m_n, separated_list1},
    sequence::{delimited, tuple},
    Parser,
};

use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

#[allow(clippy::enum_variant_names)]
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum MigrationMutatorGeneric<T> {
    AddField {
        field_path: Vec<String>,
        opt_fields: Vec<bool>,
        field_type: T,
        field_index: u32,
        default_value: Option<String>,
    },
    DropField {
        field_path: Vec<String>,
    },
    RenameField {
        from_path: Vec<String>,
        to_path: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub struct VersionedStructFieldGeneric<T> {
    pub field_type: T,
    pub field_index: u32,
    pub default_value: Option<String>,
    pub last_mutation_version: i64,
}

impl<T: PartialEq> PartialEq for VersionedStructFieldGeneric<T> {
    fn eq(&self, other: &Self) -> bool {
        self.field_type == other.field_type
            && self.field_index == other.field_index
            && self.default_value == other.default_value
    }
}

#[derive(Debug, Clone)]
pub struct VersionedStructGeneric<T> {
    pub fields: Vec<(String, VersionedStructFieldGeneric<T>)>,
    // pub order: u32,
}

#[derive(PartialEq, Debug, Clone, Hash)]
pub enum ValidVersionedStructType {
    String,
    I64,
    F64,
    Bool,
    DateTime,
    UUID,
    Option(Box<ValidVersionedStructType>),
    Array(Box<ValidVersionedStructType>),
    Struct(VersionedStructGeneric<ValidVersionedStructType>),
}

impl<T: std::hash::Hash> std::hash::Hash for VersionedStructGeneric<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // names are not included in type hash
        for (_, f) in &self.fields {
            f.field_type.hash(state);
        }
    }
}

impl<T: PartialEq> PartialEq for VersionedStructGeneric<T> {
    fn eq(&self, other: &Self) -> bool {
        self.fields == other.fields
    }
}

pub type MigrationMutatorUnvalidated = MigrationMutatorGeneric<ValidVersionedStructType>;
pub type VersionedTypeUnvalidated = VersionedStructGeneric<ValidVersionedStructType>;
pub type VersionedStructFieldUnvalidated = VersionedStructFieldGeneric<ValidVersionedStructType>;

pub type IResult<I, O, E = nom::error::VerboseError<I>> = Result<(I, O), nom::Err<E>>;

impl ValidVersionedStructType {
    pub fn is_option(&self) -> bool {
        matches!(self, ValidVersionedStructType::Option(_))
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, ValidVersionedStructType::Struct(_))
    }
}

fn valid_field_name(input: Span) -> IResult<Span, Span> {
    let (tail, tname) = take_while1(|c: char| c.is_alphanumeric() || c == '_').parse(input)?;

    Ok((tail, tname))
}

pub fn valid_base_type_name(input: Span) -> IResult<Span, ValidVersionedStructType> {
    // TODO: parse just what could be the type and say if we support it
    let (tail, btype) = alt((
        tag("String").map(|_| ValidVersionedStructType::String),
        tag("I64").map(|_| ValidVersionedStructType::I64),
        tag("F64").map(|_| ValidVersionedStructType::F64),
        tag("Bool").map(|_| ValidVersionedStructType::Bool),
        tag("DateTime").map(|_| ValidVersionedStructType::DateTime),
        tag("UUID").map(|_| ValidVersionedStructType::UUID),
    ))
    .parse(input)?;

    Ok((tail, btype))
}

pub fn valid_type_expression(input: Span) -> IResult<Span, ValidVersionedStructType> {
    let (tail, (btype, array_level, opt)) = tuple((
        alt((
            valid_base_type_name,
            parse_migration_struct.map(ValidVersionedStructType::Struct),
        )),
        many0(tag("[]")),
        opt(char('?')),
    ))
    .parse(input)?;

    let mut res = btype;

    for _ in array_level {
        res = ValidVersionedStructType::Array(Box::new(res));
    }
    if opt.is_some() {
        res = ValidVersionedStructType::Option(Box::new(res));
    }

    Ok((tail, res))
}

fn valid_field_path(input: Span) -> IResult<Span, Vec<String>> {
    let (tail, res) = many1(tuple((char('.'), valid_field_name))).parse(input)?;

    let res = res
        .into_iter()
        .map(|(_, i)| i.to_string())
        .collect::<Vec<_>>();
    Ok((tail, res))
}

fn valid_field_path_wopts(input: Span) -> IResult<Span, Vec<(String, bool)>> {
    let (tail, res) = many1(tuple((char('.'), valid_field_name, opt(char('?'))))).parse(input)?;

    let res = res
        .into_iter()
        .map(|(_, i, is_opt)| (i.to_string(), is_opt.is_some()))
        .collect::<Vec<_>>();
    Ok((tail, res))
}

fn parse_field_seq_no(input: Span) -> IResult<Span, u32> {
    let (tail, (_, digit_no)) = tuple((char('@'), many_m_n(1, 4, digit1))).parse(input)?;

    let res: String = digit_no.into_iter().map(|i| i.to_string()).collect();
    // we know will succeed as up to 4 digits
    Ok((tail, res.parse::<u32>().unwrap()))
}

#[test]
fn test_parse_seq_no() {
    let (leftover, res) = parse_field_seq_no(Span::new("@123 ")).unwrap();
    assert_eq!(*leftover, " ");
    assert_eq!(res, 123);
    assert!(parse_field_seq_no(Span::new("@-123 ")).is_err());
}

fn parse_migration_line(input: Span) -> IResult<Span, MigrationMutatorUnvalidated> {
    let (tail, res) =
        alt((
            tuple((
                tag("ADD"),
                multispace1,
                valid_field_path_wopts,
                multispace1,
                parse_field_seq_no,
                multispace1,
                valid_type_expression,
                opt(parse_default_expression),
            ))
            .map(
                |(_, _, field_path_wopt, _, field_index, _, field_type, default_value)| {
                    let mut field_path = Vec::with_capacity(field_path_wopt.len());
                    let mut opt_fields = Vec::with_capacity(field_path_wopt.len());
                    for (ps, is_opt) in field_path_wopt {
                        field_path.push(ps);
                        opt_fields.push(is_opt);
                    }
                    MigrationMutatorUnvalidated::AddField {
                        field_path,
                        opt_fields,
                        field_type,
                        field_index,
                        default_value: default_value.map(|v| v.to_string()),
                    }
                },
            ),
            tuple((tag("DROP"), multispace1, valid_field_path))
                .map(|(_, _, field_path)| MigrationMutatorUnvalidated::DropField { field_path }),
            tuple((
                tag("RENAME"),
                multispace1,
                valid_field_path,
                multispace1,
                valid_field_path,
            ))
            .map(|(_, _, from_path, _, to_path)| {
                MigrationMutatorUnvalidated::RenameField { from_path, to_path }
            }),
        ))
        .parse(input)?;

    Ok((tail, res))
}

pub(crate) fn parse_all_migration_lines(
    input: Span,
) -> IResult<Span, Vec<MigrationMutatorUnvalidated>> {
    let (tail, (_, res, _)) = tuple((
        multispace0,
        separated_list1(multispace1, parse_migration_line),
        multispace0,
    ))
    .parse(input)?;

    // TODO: better error handling in nom
    assert!(tail.is_empty());

    Ok((tail, res))
}

pub(crate) fn parse_migration_snapshot(input: Span) -> IResult<Span, VersionedTypeUnvalidated> {
    let (tail, (_, fields, _)) =
        tuple((multispace0, parse_migration_struct, multispace0)).parse(input)?;

    // TODO: better error handling in nom
    assert!(tail.is_empty());

    Ok((tail, fields))
}

fn parse_migration_struct(input: Span) -> IResult<Span, VersionedTypeUnvalidated> {
    let (tail, res) = curly_braces_expression.parse(input)?;

    // TODO: nice error instead of cryptic stuff
    let (le_tail, fields) = all_consuming(parse_migration_fields)(res)?;

    assert!(le_tail.is_empty(), "Should have been empty");

    Ok((tail, VersionedTypeUnvalidated { fields }))
}

fn parse_migration_fields(
    input: Span,
) -> IResult<Span, Vec<(String, VersionedStructFieldUnvalidated)>> {
    let (tail, (_, res, ..)) = tuple((
        multispace1,
        separated_list1(
            tuple((multispace0, char(','), multispace0)),
            parse_type_field,
        ),
        opt(tuple((multispace0, char(',')))),
        multispace0,
    ))
    .parse(input)?;

    Ok((tail, res))
}

fn parse_default_expression(input: Span) -> IResult<Span, Span> {
    let (tail, (_, _, _, dp)) =
        tuple((multispace1, tag("DEFAULT"), multispace1, parse_quoted_text)).parse(input)?;

    Ok((tail, dp))
}

fn parse_quoted_text(input: Span) -> IResult<Span, Span> {
    match parse_quoted_text_custom(input, '"') {
        Err(_) => parse_quoted_text_custom(input, '\''),
        Ok(ok) => Ok(ok),
    }
}

fn parse_quoted_text_custom(input: Span, quote_char: char) -> IResult<Span, Span> {
    let none_of_chars = format!("{quote_char}");
    let escapable = format!("\\{quote_char}");
    let (tail, res) = delimited(
        char(quote_char),
        cut(escaped(
            many0(none_of(none_of_chars.as_str())),
            '\\',
            one_of(escapable.as_str()),
        )),
        char(quote_char),
    )
    .parse(input)?;

    Ok((tail, res))
}

fn parse_type_field(input: Span) -> IResult<Span, (String, VersionedStructFieldUnvalidated)> {
    let (tail, (vf, _, field_index, _, _, _, expr, maybe_default)) = tuple((
        valid_field_name,
        multispace1,
        parse_field_seq_no,
        multispace0,
        char(':'),
        multispace0,
        valid_type_expression,
        opt(parse_default_expression),
    ))
    .parse(input)?;

    let field = VersionedStructFieldUnvalidated {
        field_type: expr,
        default_value: maybe_default.map(|dp| dp.to_string()),
        field_index,
        last_mutation_version: -1,
    };

    Ok((tail, (vf.to_string(), field)))
}

fn curly_braces_expression(input: Span) -> IResult<Span, Span> {
    let (tail, (_, content, _)) =
        tuple((char('{'), take_until_unbalanced('{', '}'), char('}'))).parse(input)?;

    Ok((tail, content))
}

#[test]
fn test_parse_quoted_text() {
    let (leftover, res) = parse_quoted_text(Span::new(r#""henlo bois""#)).unwrap();
    assert_eq!((*leftover, *res), ("", "henlo bois"));

    let (leftover, res) = parse_quoted_text(Span::new(r#"'henlo bois'"#)).unwrap();
    assert_eq!((*leftover, *res), ("", "henlo bois"));

    let (leftover, res) = parse_quoted_text(Span::new(r#"'henlo bois'ayo"#)).unwrap();

    assert_eq!((*leftover, *res), ("ayo", "henlo bois"));

    assert!(parse_quoted_text(Span::new(r#" 'henlo bois'"#)).is_err());
}

#[test]
fn test_basic_migration_line_parse() {
    let (span, res) = parse_all_migration_lines(Span::new(
        r#"
        ADD .hey @7 String DEFAULT "henlo"
        DROP .booyah
        ADD .gonzo.boo @8 I64
        RENAME .rekt .rookt
    "#)).unwrap();

    assert_eq!(
        (*span, res),
        (
            "",
            vec![
                MigrationMutatorUnvalidated::AddField {
                    opt_fields: vec![false],
                    field_path: vec!["hey".to_string()],
                    field_type: ValidVersionedStructType::String,
                    default_value: Some("henlo".to_string()),
                    field_index: 7,
                },
                MigrationMutatorUnvalidated::DropField {
                    field_path: vec!["booyah".to_string()],
                },
                MigrationMutatorUnvalidated::AddField {
                    opt_fields: vec![false, false],
                    field_path: vec!["gonzo".to_string(), "boo".to_string()],
                    field_type: ValidVersionedStructType::I64,
                    default_value: None,
                    field_index: 8,
                },
                MigrationMutatorUnvalidated::RenameField {
                    from_path: vec!["rekt".to_string()],
                    to_path: vec!["rookt".to_string()],
                },
            ]
        )
    )
}

#[test]
fn test_basic_migration_snapshot_parse() {
    let (span, res) = parse_migration_snapshot(Span::new(
        r#"
        {
            rookt @0 :I64,
            hey @1 :String[]?,
            gonzo @7 :{
                boo @2 :I64,
                is_something @3 :Bool,
                lozo @4 :{
                    mozo @5 :F64?,
                },
            }
        }
    "#,
    )).unwrap();

    assert_eq!((*span, res), ("", VersionedTypeUnvalidated {
        fields: vec![
            ("rookt".to_string(), VersionedStructFieldUnvalidated {
                field_type: ValidVersionedStructType::I64,
                default_value: None,
                field_index: 0,
                last_mutation_version: -1,
            }),
            ("hey".to_string(), VersionedStructFieldUnvalidated {
                field_type: ValidVersionedStructType::Option(Box::new(ValidVersionedStructType::Array(Box::new(ValidVersionedStructType::String)))),
                default_value: None,
                field_index: 1,
                last_mutation_version: -1,
            }),
            ("gonzo".to_string(), VersionedStructFieldUnvalidated {
                field_type: ValidVersionedStructType::Struct(VersionedTypeUnvalidated {
                    fields: vec![
                        ("boo".to_string(), VersionedStructFieldUnvalidated {
                            field_type: ValidVersionedStructType::I64,
                            default_value: None,
                            field_index: 2,
                            last_mutation_version: -1,
                        }),
                        ("is_something".to_string(), VersionedStructFieldUnvalidated {
                            field_type: ValidVersionedStructType::Bool,
                            default_value: None,
                            field_index: 3,
                            last_mutation_version: -1,
                        }),
                        ("lozo".to_string(), VersionedStructFieldUnvalidated {
                            field_type: ValidVersionedStructType::Struct(VersionedTypeUnvalidated {
                                fields: vec![
                                    ("mozo".to_string(), VersionedStructFieldUnvalidated {
                                        field_type: ValidVersionedStructType::Option(Box::new(ValidVersionedStructType::F64)),
                                        default_value: None,
                                        field_index: 5,
                                        last_mutation_version: -1,
                                    }),
                                ]
                            }),
                            default_value: None,
                            field_index: 4,
                            last_mutation_version: -1,
                        }),
                    ]
                }),
                default_value: None,
                field_index: 7,
                last_mutation_version: -1,
            }),
        ]
    }));
}

#[test]
fn test_bw_type_expected_hash() {
    let test_val = VersionedTypeUnvalidated {
        fields: vec![
            (
                "string".to_string(),
                VersionedStructFieldUnvalidated {
                    field_type: ValidVersionedStructType::String,
                    field_index: 0,
                    default_value: None,
                    last_mutation_version: -1,
                },
            ),
            (
                "i64".to_string(),
                VersionedStructFieldUnvalidated {
                    field_type: ValidVersionedStructType::I64,
                    field_index: 1,
                    default_value: None,
                    last_mutation_version: -1,
                },
            ),
            (
                "f64".to_string(),
                VersionedStructFieldUnvalidated {
                    field_type: ValidVersionedStructType::I64,
                    field_index: 2,
                    default_value: None,
                    last_mutation_version: -1,
                },
            ),
            (
                "bool".to_string(),
                VersionedStructFieldUnvalidated {
                    field_type: ValidVersionedStructType::Bool,
                    field_index: 3,
                    default_value: None,
                    last_mutation_version: -1,
                },
            ),
            (
                "opt".to_string(),
                VersionedStructFieldUnvalidated {
                    field_type: ValidVersionedStructType::Option(Box::new(
                        ValidVersionedStructType::I64,
                    )),
                    field_index: 4,
                    default_value: None,
                    last_mutation_version: -1,
                },
            ),
            (
                "arr".to_string(),
                VersionedStructFieldUnvalidated {
                    field_type: ValidVersionedStructType::Array(Box::new(
                        ValidVersionedStructType::I64,
                    )),
                    field_index: 5,
                    default_value: None,
                    last_mutation_version: -1,
                },
            ),
        ],
    };

    let mut test_val =
        crate::static_analysis::bw_compat_types::ComputedType::new(0x1234, test_val, Vec::new());

    let res = test_val.version_hash();

    // hash should only depend on type values
    test_val
        .the_type
        .fields
        .iter_mut()
        .for_each(|(fname, fval)| {
            *fname = "a".to_string();
            fval.field_index = 777;
        });
    let res2 = test_val.version_hash();

    assert_eq!(res, res2);
    assert_eq!(0x1234EBA350308B0E, res);
}
