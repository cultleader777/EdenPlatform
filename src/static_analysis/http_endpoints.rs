use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::database::{
    TableRowPointerBackendApplication, TableRowPointerBackendApplicationDeploymentIngress,
    TableRowPointerBackendHttpEndpoint, TableRowPointerFrontendApplication,
    TableRowPointerFrontendPage, TableRowPointerHttpEndpointDataType, TableRowPointerHttpMethods,
    TableRowPointerVersionedType,
};

use super::{
    projections::{Index, Projection},
    PlatformValidationError,
};

pub struct PathArgs {
    pub original_path: String,
    pub required_args: Vec<CorePathSegment>,
    pub opt_query_args: Vec<(String, HttpArgumentType)>,
}

pub struct CheckedHttpEndpoint {
    pub path_args: PathArgs,
    pub is_raw_input_body: bool,
    pub receive_body_as_stream: bool,
    pub input_body_type: Option<TableRowPointerVersionedType>,
    pub output_body_type: Option<TableRowPointerVersionedType>,
    pub expected_input_body_content_type: Option<&'static str>,
}

impl CheckedHttpEndpoint {
    pub fn has_required_arguments(&self) -> bool {
        for pa in &self.path_args.required_args {
            if matches!(pa, CorePathSegment::Argument(..)) {
                return true;
            }
        }

        if self.input_body_type.is_some() {
            return true;
        }

        false
    }

    pub fn has_optional_arguments(&self) -> bool {
        !self.path_args.opt_query_args.is_empty()
    }
}

lazy_static! {
    static ref ARGUMENT_REGEX: regex::Regex =
        regex::Regex::new(r#"^\{([a-z0-9_]+):([A-Z0-9]+)(\[\])?\}$"#).unwrap();
    static ref HTTP_SEGMENT_REGEX: regex::Regex = regex::Regex::new(r#"^[a-z0-9_-]+$"#).unwrap();
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ValidHttpPrimitiveType {
    Int,
    Float,
    Bool,
    Text,
}

pub struct HttpArgumentType {
    pub the_type: ValidHttpPrimitiveType,
    pub is_multiple: bool,
}

#[derive(PartialEq, Eq, Debug)]
pub enum CorePathSegment {
    Text(String),
    Argument(String, ValidHttpPrimitiveType),
    LastSlash,
}

pub fn check_http_endpoint(
    db: &crate::database::Database,
    endpoint: TableRowPointerBackendHttpEndpoint,
    bwtypes: &Index<String, TableRowPointerVersionedType>,
    valid_http_methods: &Index<String, TableRowPointerHttpMethods>,
    valid_http_data_types: &Index<String, TableRowPointerHttpEndpointDataType>,
) -> Result<CheckedHttpEndpoint, PlatformValidationError> {
    let path = db.backend_http_endpoint().c_path(endpoint);
    let mut expected_input_body_content_type = None;

    let endpoint_data_type = db.backend_http_endpoint().c_data_type(endpoint);
    let endpoint_data_type_str = db.http_endpoint_data_type().c_http_endpoint_data_type(endpoint_data_type);
    let is_json_endpoint = endpoint_data_type == valid_http_data_types.values(&"json".to_string())[0];
    let is_raw_endpoint = endpoint_data_type == valid_http_data_types.values(&"raw".to_string())[0];
    let is_get_method = db.backend_http_endpoint().c_http_method(endpoint)
        == valid_http_methods.values(&"GET".to_string())[0];
    let is_post_method = db.backend_http_endpoint().c_http_method(endpoint)
        == valid_http_methods.values(&"POST".to_string())[0];
    let is_put_method = db.backend_http_endpoint().c_http_method(endpoint)
        == valid_http_methods.values(&"PUT".to_string())[0];
    let receive_body_as_stream = db.backend_http_endpoint().c_receive_body_as_stream(endpoint);
    let needs_input_body = is_put_method || is_post_method;
    if needs_input_body && !is_json_endpoint && !is_raw_endpoint {
        // not a bug, think of how to deal with this.
        // application form data is useless html leftover, nobody cares about that
        // we have binary format for efficiency and json for readability
        panic!("Only json/raw post/put bodies are supported so far")
    }

    let str_input_body_type = db.backend_http_endpoint().c_input_body_type(endpoint);
    let has_input_body_type = !str_input_body_type.is_empty();
    let is_raw_input_body = str_input_body_type == "raw";

    let str_output_body_type = db.backend_http_endpoint().c_output_body_type(endpoint);
    let has_output_body_type = !str_output_body_type.is_empty();

    if is_get_method && has_input_body_type {
        return Err(
            PlatformValidationError::HttpEndpointGetMethodCannotHaveInputBody {
                application_name: db
                    .backend_application()
                    .c_application_name(db.backend_http_endpoint().c_parent(endpoint))
                    .clone(),
                endpoint_name: db
                    .backend_http_endpoint()
                    .c_http_endpoint_name(endpoint)
                    .clone(),
                full_path: path.clone(),
                input_body_type: db
                    .backend_http_endpoint()
                    .c_input_body_type(endpoint)
                    .clone(),
            },
        );
    }

    let path_args = check_http_path(path)?;

    let body_type = db.backend_http_endpoint().c_input_body_type(endpoint);
    let mut input_body_type = None;
    if !body_type.is_empty() {
        let types = bwtypes.values(body_type);
        if types.is_empty() && !(is_raw_endpoint && is_raw_input_body) {
            return Err(PlatformValidationError::HttpPathInputBwTypeNotFound {
                application_name: db
                    .backend_application()
                    .c_application_name(db.backend_http_endpoint().c_parent(endpoint))
                    .clone(),
                endpoint_name: db
                    .backend_http_endpoint()
                    .c_http_endpoint_name(endpoint)
                    .clone(),
                full_path: path.clone(),
                input_body_type: body_type.clone(),
            });
        }

        if !is_raw_endpoint {
            assert!(
                is_json_endpoint,
                "only json for posts/puts supported so far"
            );
            expected_input_body_content_type = Some("application/json");
            assert_eq!(types.len(), 1);
            input_body_type = Some(types[0]);
        }
    }

    let body_type = db.backend_http_endpoint().c_output_body_type(endpoint);
    let mut output_body_type = None;
    if !body_type.is_empty() {
        let types = bwtypes.values(body_type);
        if types.is_empty() {
            return Err(PlatformValidationError::HttpPathOutputBwTypeNotFound {
                application_name: db
                    .backend_application()
                    .c_application_name(db.backend_http_endpoint().c_parent(endpoint))
                    .clone(),
                endpoint_name: db
                    .backend_http_endpoint()
                    .c_http_endpoint_name(endpoint)
                    .clone(),
                full_path: path.clone(),
                output_body_type: body_type.clone(),
            });
        }

        assert_eq!(types.len(), 1);
        output_body_type = Some(types[0]);
    }

    // if output is json it must have output body type
    if is_json_endpoint && output_body_type.is_none() {
        return Err(
            PlatformValidationError::HttpEndpointBodyTypeIsJsonButOutputBwTypeIsUnspecified {
                application_name: db
                    .backend_application()
                    .c_application_name(db.backend_http_endpoint().c_parent(endpoint))
                    .clone(),
                endpoint_name: db
                    .backend_http_endpoint()
                    .c_http_endpoint_name(endpoint)
                    .clone(),
                full_path: path.clone(),
            },
        );
    }

    if is_json_endpoint && (is_put_method || is_post_method) && input_body_type.is_none() {
        return Err(PlatformValidationError::HttpEndpointBodyTypeIsJsonAndPostPutMethodButInputBwTypeIsUnspecified {
            application_name: db.backend_application().c_application_name(db.backend_http_endpoint().c_parent(endpoint)).clone(),
            endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(endpoint).clone(),
            full_path: path.clone(),
            http_method: db.http_methods().c_http_method_name(db.backend_http_endpoint().c_http_method(endpoint)).clone(),
        });
    }

    if is_raw_endpoint {
        if has_input_body_type {
            if !is_raw_input_body {
                return Err(PlatformValidationError::HttpEndpointDataTypeIsRawButInputBodyTypeIsSpecified {
                    application_name: db.backend_application().c_application_name(db.backend_http_endpoint().c_parent(endpoint)).clone(),
                    endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(endpoint).clone(),
                    full_path: path.clone(),
                    http_method: db.http_methods().c_http_method_name(db.backend_http_endpoint().c_http_method(endpoint)).clone(),
                    input_body_type: str_input_body_type.clone(),
                });
            }
        }

        if has_output_body_type {
            return Err(PlatformValidationError::HttpEndpointDataTypeIsRawButOutputBodyTypeIsSpecified {
                application_name: db.backend_application().c_application_name(db.backend_http_endpoint().c_parent(endpoint)).clone(),
                endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(endpoint).clone(),
                full_path: path.clone(),
                http_method: db.http_methods().c_http_method_name(db.backend_http_endpoint().c_http_method(endpoint)).clone(),
                output_body_type: str_output_body_type.clone(),
            });
        }
    }

    if receive_body_as_stream {
        if !is_raw_endpoint || !is_raw_input_body {
            return Err(PlatformValidationError::HttpEndpointReceiveBodyAsStreamIsOnlySupportedForRawEndpointsWithRawInputBodyType {
                application_name: db.backend_application().c_application_name(db.backend_http_endpoint().c_parent(endpoint)).clone(),
                endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(endpoint).clone(),
                full_path: path.clone(),
                http_method: db.http_methods().c_http_method_name(db.backend_http_endpoint().c_http_method(endpoint)).clone(),
                input_body_type: str_input_body_type.clone(),
                endpoint_data_type: endpoint_data_type_str.clone(),
                expected_endpoint_data_type: "raw".to_string(),
                expected_input_body_type: "raw".to_string(),
            });
        }
    }

    Ok(CheckedHttpEndpoint {
        path_args,
        is_raw_input_body,
        receive_body_as_stream,
        input_body_type,
        output_body_type,
        expected_input_body_content_type,
    })
}

pub fn check_http_path(path: &str) -> Result<PathArgs, PlatformValidationError> {
    let mut optional_arguments = Vec::new();
    let mut mandatory_segments = Vec::new();

    let res = path.split('?').collect::<Vec<_>>();
    if res.len() > 2 {
        return Err(PlatformValidationError::HttpPathBadFormat {
            explanation: "More than one ? (question mark) found in path".to_string(),
            path: path.to_owned(),
        });
    }

    if res.len() == 2 {
        let query_part = res[1];

        for arg in query_part.split('&') {
            optional_arguments.push(try_parse_http_arg(path, arg)?);
        }
    }

    let core_path = res[0];
    if core_path.is_empty() {
        return Err(PlatformValidationError::HttpEmptyCorePath {
            full_path: path.to_owned(),
        });
    }

    if !core_path.starts_with('/') {
        return Err(PlatformValidationError::HttpCorePathMustStartWithSlash {
            full_path: path.to_owned(),
        });
    }

    let core_split = core_path.split('/').skip(1).collect::<Vec<_>>();

    for (idx, core_segment) in core_split.iter().enumerate() {
        let is_last = idx == core_split.len() - 1;
        if core_segment.is_empty() {
            if is_last {
                mandatory_segments.push(CorePathSegment::LastSlash);
                break;
            } else {
                return Err(
                    PlatformValidationError::HttpMultipleConsecutiveSlashesNotAllowed {
                        full_path: path.to_owned(),
                    },
                );
            }
        }

        match try_parse_http_arg(path, core_segment) {
            Ok((name, t)) => {
                if t.is_multiple {
                    return Err(
                        PlatformValidationError::HttpMultipleArgumentsOnlyAllowedInQuery {
                            bad_argument_name: name,
                            full_path: path.to_owned(),
                        },
                    );
                }

                mandatory_segments.push(CorePathSegment::Argument(name, t.the_type));
            }
            Err(e) => {
                if let PlatformValidationError::HttpInvalidArgumentType { .. } = &e {
                    return Err(e);
                }

                if HTTP_SEGMENT_REGEX.is_match(core_segment) {
                    mandatory_segments.push(CorePathSegment::Text(core_segment.to_string()));
                } else {
                    return Err(PlatformValidationError::HttpInvalidCoreSegment {
                        full_path: path.to_owned(),
                        segment: core_segment.to_lowercase(),
                        explanation:
                            "Either argument or alphanumeric string is allowed in core path segment",
                    });
                }
            }
        }
    }

    let mut vars: HashSet<String> = HashSet::new();

    for ma in &mandatory_segments {
        if let CorePathSegment::Argument(an, _) = ma {
            if !vars.insert(an.clone()) {
                return Err(PlatformValidationError::HttpPathDuplicateArgumentName {
                    full_path: path.to_owned(),
                    duplicate_arg_name: an.clone(),
                });
            }
        }
    }

    for (oa, _) in &optional_arguments {
        if !vars.insert(oa.clone()) {
            return Err(PlatformValidationError::HttpPathDuplicateArgumentName {
                full_path: path.to_owned(),
                duplicate_arg_name: oa.clone(),
            });
        }
    }

    for uniq_name in &vars {
        for reserved in ["input_body", "headers"] {
            if reserved == uniq_name.as_str() {
                return Err(PlatformValidationError::HttpPathReservedArgumentName {
                    full_path: path.to_owned(),
                    reserved_arg_name: uniq_name.clone(),
                });
            }
        }
    }

    optional_arguments.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(PathArgs {
        original_path: path.to_owned(),
        required_args: mandatory_segments,
        opt_query_args: optional_arguments,
    })
}

impl ToString for ValidHttpPrimitiveType {
    fn to_string(&self) -> String {
        match self {
            ValidHttpPrimitiveType::Int => "INT",
            ValidHttpPrimitiveType::Float => "FLOAT",
            ValidHttpPrimitiveType::Bool => "BOOL",
            ValidHttpPrimitiveType::Text => "TEXT",
        }
        .to_string()
    }
}

pub fn check_app_duplicate_paths(
    db: &crate::database::Database,
    app: TableRowPointerBackendApplication,
    proj: &Projection<TableRowPointerBackendHttpEndpoint, CheckedHttpEndpoint>,
) -> Result<HttpPathTree<TableRowPointerBackendHttpEndpoint>, PlatformValidationError> {
    let mut tree = HttpPathTree::root();

    for endpoint in db
        .backend_application()
        .c_children_backend_http_endpoint(app)
    {
        let checked = proj.value(*endpoint);

        let method = match db
            .http_methods()
            .c_http_method_name(db.backend_http_endpoint().c_http_method(*endpoint))
            .as_str()
        {
            "GET" => PageMethod::GET,
            "POST" => PageMethod::POST,
            "PUT" => PageMethod::PUT,
            other => panic!("Unknown HTTP method: {other}, this should be checked earlier!"),
        };

        let mut tree_ref = &mut tree;

        for (idx, seg) in checked.path_args.required_args.iter().enumerate() {
            let is_last = idx == checked.path_args.required_args.len() - 1;
            match seg {
                CorePathSegment::Text(txt) => {
                    if is_last {
                        tree_ref.add_named_page(txt.as_str(), method.clone(), *endpoint).map_err(|e| {
                            match e {
                                HttpPathTreeCheckerErrors::DuplicateNamedPage { prev_page, duplicate_page, page_method, .. } => {
                                    PlatformValidationError::AppHttpTreeErrorDuplicatePagePath {
                                        application_name: db.backend_application().c_application_name(db.backend_http_endpoint().c_parent(*endpoint)).clone(),
                                        previous_endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(prev_page).clone(),
                                        previous_endpoint_path: db.backend_http_endpoint().c_path(prev_page).clone(),
                                        duplicate_endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(duplicate_page).clone(),
                                        duplicate_endpoint_path: db.backend_http_endpoint().c_path(duplicate_page).clone(),
                                        page_method: page_method.to_string(),
                                    }
                                },
                                HttpPathTreeCheckerErrors::PathSegmentCanBeVariableOrStaticPage { variable_page, static_page } => {
                                    PlatformValidationError::AppHttpTreeErrorArgPageMixedWithStaticPage {
                                        application_name: db.backend_application().c_application_name(db.backend_http_endpoint().c_parent(*endpoint)).clone(),
                                        a_endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(variable_page).clone(),
                                        a_endpoint_path: db.backend_http_endpoint().c_path(variable_page).clone(),
                                        b_endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(static_page).clone(),
                                        b_endpoint_path: db.backend_http_endpoint().c_path(static_page).clone(),
                                    }
                                },
                                e => panic!("Unexpected error during path checks: {:?}", e)
                            }
                        })?;
                    } else {
                        // slash or other page MUST follow this segment, change ref
                        tree_ref = tree_ref.fetch_level(txt.as_str(), endpoint).map_err(|e| {
                            match e {
                                HttpPathTreeCheckerErrors::PathSegmentCanBeVariableOrStaticPage { variable_page, static_page } => {
                                    PlatformValidationError::AppHttpTreeErrorArgPageMixedWithStaticPage {
                                        application_name: db.backend_application().c_application_name(db.backend_http_endpoint().c_parent(*endpoint)).clone(),
                                        a_endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(variable_page).clone(),
                                        a_endpoint_path: db.backend_http_endpoint().c_path(variable_page).clone(),
                                        b_endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(static_page).clone(),
                                        b_endpoint_path: db.backend_http_endpoint().c_path(static_page).clone(),
                                    }
                                },
                                e => panic!("Unexpected error during path checks: {:?}", e)
                            }
                        })?;
                    }
                }
                CorePathSegment::Argument(arg_name, arg_t) => {
                    // last argument, no slash at end
                    let arg_name = format!("{}:{}", arg_name, arg_t.to_string());
                    if is_last {
                        tree_ref.add_argument_final(arg_name.as_str(), method.clone(), *endpoint).map_err(|e| {
                            match e {
                                HttpPathTreeCheckerErrors::PathSegmentCanBeVariableOrStaticPage { variable_page, static_page } => {
                                    PlatformValidationError::AppHttpTreeErrorArgPageMixedWithStaticPage {
                                        application_name: db.backend_application().c_application_name(db.backend_http_endpoint().c_parent(*endpoint)).clone(),
                                        a_endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(variable_page).clone(),
                                        a_endpoint_path: db.backend_http_endpoint().c_path(variable_page).clone(),
                                        b_endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(static_page).clone(),
                                        b_endpoint_path: db.backend_http_endpoint().c_path(static_page).clone(),
                                    }
                                },
                                HttpPathTreeCheckerErrors::PathSegmentCanHaveOnlyOneArgumentVariable { prev_page, duplicate_page } => {
                                    PlatformValidationError::AppHttpTreeErrorArgPageSegmentMultipleNames {
                                        application_name: db.backend_application().c_application_name(db.backend_http_endpoint().c_parent(*endpoint)).clone(),
                                        arg_a_endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(prev_page).clone(),
                                        arg_a_endpoint_path: db.backend_http_endpoint().c_path(prev_page).clone(),
                                        arg_b_endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(duplicate_page).clone(),
                                        arg_b_endpoint_path: db.backend_http_endpoint().c_path(duplicate_page).clone(),
                                    }
                                },
                                HttpPathTreeCheckerErrors::ArgPageAlreadyTaken { prev_page, duplicate_page, page_method } => {
                                    PlatformValidationError::AppHttpTreeErrorArgPageTakenTwice {
                                        application_name: db.backend_application().c_application_name(db.backend_http_endpoint().c_parent(*endpoint)).clone(),
                                        arg_a_endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(prev_page).clone(),
                                        arg_a_endpoint_path: db.backend_http_endpoint().c_path(prev_page).clone(),
                                        arg_b_endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(duplicate_page).clone(),
                                        arg_b_endpoint_path: db.backend_http_endpoint().c_path(duplicate_page).clone(),
                                        page_method: page_method.to_string(),
                                    }
                                },
                                e => panic!("Unexpected error during path checks: {:?}", e)
                            }
                        })?;
                    } else {
                        // slash or other page follows this
                        tree_ref = tree_ref.add_argument_with_tree(arg_name.as_str(), endpoint).map_err(|e| {
                            match e {
                                HttpPathTreeCheckerErrors::PathSegmentCanBeVariableOrStaticPage { variable_page, static_page } => {
                                    PlatformValidationError::AppHttpTreeErrorArgPageMixedWithStaticPage {
                                        application_name: db.backend_application().c_application_name(db.backend_http_endpoint().c_parent(*endpoint)).clone(),
                                        a_endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(variable_page).clone(),
                                        a_endpoint_path: db.backend_http_endpoint().c_path(variable_page).clone(),
                                        b_endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(static_page).clone(),
                                        b_endpoint_path: db.backend_http_endpoint().c_path(static_page).clone(),
                                    }
                                },
                                HttpPathTreeCheckerErrors::PathSegmentCanHaveOnlyOneArgumentVariable { prev_page, duplicate_page } => {
                                    PlatformValidationError::AppHttpTreeErrorArgPageSegmentMultipleNames {
                                        application_name: db.backend_application().c_application_name(db.backend_http_endpoint().c_parent(*endpoint)).clone(),
                                        arg_a_endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(prev_page).clone(),
                                        arg_a_endpoint_path: db.backend_http_endpoint().c_path(prev_page).clone(),
                                        arg_b_endpoint_name: db.backend_http_endpoint().c_http_endpoint_name(duplicate_page).clone(),
                                        arg_b_endpoint_path: db.backend_http_endpoint().c_path(duplicate_page).clone(),
                                    }
                                },
                                e => panic!("Unexpected error during path checks: {:?}", e)
                            }
                        })?;
                    }
                }
                CorePathSegment::LastSlash => {
                    tree_ref
                        .lock_index(method.clone(), *endpoint)
                        .map_err(|e| match e {
                            HttpPathTreeCheckerErrors::IndexPageLockedTwice {
                                prev_page,
                                duplicate_page,
                                page_method,
                            } => PlatformValidationError::AppHttpTreeErrorDuplicatePagePath {
                                application_name: db
                                    .backend_application()
                                    .c_application_name(
                                        db.backend_http_endpoint().c_parent(*endpoint),
                                    )
                                    .clone(),
                                previous_endpoint_name: db
                                    .backend_http_endpoint()
                                    .c_http_endpoint_name(prev_page)
                                    .clone(),
                                previous_endpoint_path: db
                                    .backend_http_endpoint()
                                    .c_path(prev_page)
                                    .clone(),
                                duplicate_endpoint_name: db
                                    .backend_http_endpoint()
                                    .c_http_endpoint_name(duplicate_page)
                                    .clone(),
                                duplicate_endpoint_path: db
                                    .backend_http_endpoint()
                                    .c_path(duplicate_page)
                                    .clone(),
                                page_method: page_method.to_string(),
                            },
                            e => panic!("Unexpected error during path checks: {:?}", e),
                        })?;
                }
            }
        }
    }

    Ok(tree)
}

pub fn check_frontend_duplicate_paths(
    db: &crate::database::Database,
    app: TableRowPointerFrontendApplication,
    proj: &Projection<TableRowPointerFrontendPage, PathArgs>,
) -> Result<HttpPathTree<TableRowPointerFrontendPage>, PlatformValidationError> {
    let mut tree = HttpPathTree::root();

    let application_name = || db.frontend_application().c_application_name(app).clone();

    for endpoint in db.frontend_application().c_children_frontend_page(app) {
        let checked = proj.value(*endpoint);

        let mut tree_ref = &mut tree;
        // Frontend only servers static GET requests
        let page_method = PageMethod::GET;

        let ename = |page| db.frontend_page().c_page_name(page).clone();
        let epath = |page| db.frontend_page().c_path(page).clone();

        for (idx, seg) in checked.required_args.iter().enumerate() {
            let is_last = idx == checked.required_args.len() - 1;
            match seg {
                CorePathSegment::Text(txt) => {
                    if is_last {
                        tree_ref.add_named_page(txt.as_str(), page_method.clone(), *endpoint).map_err(|e| {
                            match e {
                                HttpPathTreeCheckerErrors::DuplicateNamedPage { prev_page, duplicate_page, page_method, .. } => {
                                    PlatformValidationError::AppHttpTreeErrorDuplicatePagePath {
                                        application_name: application_name(),
                                        previous_endpoint_name: ename(prev_page),
                                        previous_endpoint_path: epath(prev_page),
                                        duplicate_endpoint_name: ename(duplicate_page),
                                        duplicate_endpoint_path: epath(duplicate_page),
                                        page_method: page_method.to_string(),
                                    }
                                },
                                HttpPathTreeCheckerErrors::PathSegmentCanBeVariableOrStaticPage { variable_page, static_page } => {
                                    PlatformValidationError::AppHttpTreeErrorArgPageMixedWithStaticPage {
                                        application_name: application_name(),
                                        a_endpoint_name: ename(variable_page),
                                        a_endpoint_path: epath(variable_page),
                                        b_endpoint_name: ename(static_page),
                                        b_endpoint_path: epath(static_page),
                                    }
                                },
                                e => panic!("Unexpected error during path checks: {:?}", e)
                            }
                        })?;
                    } else {
                        // slash or other page MUST follow this segment, change ref
                        tree_ref = tree_ref.fetch_level(txt.as_str(), endpoint).map_err(|e| {
                            match e {
                                HttpPathTreeCheckerErrors::PathSegmentCanBeVariableOrStaticPage { variable_page, static_page } => {
                                    PlatformValidationError::AppHttpTreeErrorArgPageMixedWithStaticPage {
                                        application_name: application_name(),
                                        a_endpoint_name: ename(variable_page),
                                        a_endpoint_path: epath(variable_page),
                                        b_endpoint_name: ename(static_page),
                                        b_endpoint_path: epath(static_page),
                                    }
                                },
                                e => panic!("Unexpected error during path checks: {:?}", e)
                            }
                        })?;
                    }
                }
                CorePathSegment::Argument(arg_name, arg_t) => {
                    // last argument, no slash at end
                    let arg_name = format!("{}:{}", arg_name, arg_t.to_string());
                    if is_last {
                        tree_ref.add_argument_final(arg_name.as_str(), page_method.clone(), *endpoint).map_err(|e| {
                            match e {
                                HttpPathTreeCheckerErrors::PathSegmentCanBeVariableOrStaticPage { variable_page, static_page } => {
                                    PlatformValidationError::AppHttpTreeErrorArgPageMixedWithStaticPage {
                                        application_name: application_name(),
                                        a_endpoint_name: ename(variable_page),
                                        a_endpoint_path: epath(variable_page),
                                        b_endpoint_name: ename(static_page),
                                        b_endpoint_path: epath(static_page),
                                    }
                                },
                                HttpPathTreeCheckerErrors::PathSegmentCanHaveOnlyOneArgumentVariable { prev_page, duplicate_page } => {
                                    PlatformValidationError::AppHttpTreeErrorArgPageSegmentMultipleNames {
                                        application_name: application_name(),
                                        arg_a_endpoint_name: ename(prev_page),
                                        arg_a_endpoint_path: epath(prev_page),
                                        arg_b_endpoint_name: ename(duplicate_page),
                                        arg_b_endpoint_path: epath(duplicate_page),
                                    }
                                },
                                HttpPathTreeCheckerErrors::ArgPageAlreadyTaken { prev_page, duplicate_page, page_method } => {
                                    PlatformValidationError::AppHttpTreeErrorArgPageTakenTwice {
                                        application_name: application_name(),
                                        arg_a_endpoint_name: ename(prev_page),
                                        arg_a_endpoint_path: epath(prev_page),
                                        arg_b_endpoint_name: ename(duplicate_page),
                                        arg_b_endpoint_path: epath(duplicate_page),
                                        page_method: page_method.to_string(),
                                    }
                                },
                                e => panic!("Unexpected error during path checks: {:?}", e)
                            }
                        })?;
                    } else {
                        // slash or other page follows this
                        tree_ref = tree_ref.add_argument_with_tree(arg_name.as_str(), endpoint).map_err(|e| {
                            match e {
                                HttpPathTreeCheckerErrors::PathSegmentCanBeVariableOrStaticPage { variable_page, static_page } => {
                                    PlatformValidationError::AppHttpTreeErrorArgPageMixedWithStaticPage {
                                        application_name: application_name(),
                                        a_endpoint_name: ename(variable_page),
                                        a_endpoint_path: epath(variable_page),
                                        b_endpoint_name: ename(static_page),
                                        b_endpoint_path: epath(static_page),
                                    }
                                },
                                HttpPathTreeCheckerErrors::PathSegmentCanHaveOnlyOneArgumentVariable { prev_page, duplicate_page } => {
                                    PlatformValidationError::AppHttpTreeErrorArgPageSegmentMultipleNames {
                                        application_name: application_name(),
                                        arg_a_endpoint_name: ename(prev_page),
                                        arg_a_endpoint_path: epath(prev_page),
                                        arg_b_endpoint_name: ename(duplicate_page),
                                        arg_b_endpoint_path: epath(duplicate_page),
                                    }
                                },
                                e => panic!("Unexpected error during path checks: {:?}", e)
                            }
                        })?;
                    }
                }
                CorePathSegment::LastSlash => {
                    tree_ref
                        .lock_index(page_method.clone(), *endpoint)
                        .map_err(|e| match e {
                            HttpPathTreeCheckerErrors::IndexPageLockedTwice {
                                prev_page,
                                duplicate_page,
                                page_method,
                            } => PlatformValidationError::AppHttpTreeErrorDuplicatePagePath {
                                application_name: application_name(),
                                previous_endpoint_name: ename(prev_page),
                                previous_endpoint_path: epath(prev_page),
                                duplicate_endpoint_name: ename(duplicate_page),
                                duplicate_endpoint_path: epath(duplicate_page),
                                page_method: page_method.to_string(),
                            },
                            e => panic!("Unexpected error during path checks: {:?}", e),
                        })?;
                }
            }
        }
    }

    Ok(tree)
}

pub fn backend_ingress_endpoints(
    db: &crate::database::Database,
) -> Result<
    Projection<
        TableRowPointerBackendApplicationDeploymentIngress,
        BTreeSet<TableRowPointerBackendHttpEndpoint>,
    >,
    PlatformValidationError,
> {
    Projection::maybe_create(
        db.backend_application_deployment_ingress().rows_iter(),
        |ingress| {
            let endpoints_list = db
                .backend_application_deployment_ingress()
                .c_endpoint_list(ingress);
            let deployment = db
                .backend_application_deployment_ingress()
                .c_deployment(ingress);
            let application = db
                .backend_application_deployment()
                .c_application_name(deployment);
            let endpoints = db
                .backend_application()
                .c_children_backend_http_endpoint(application);
            let available_endpoints = || {
                db.backend_application()
                    .c_children_backend_http_endpoint(application)
                    .iter()
                    .map(|i| db.backend_http_endpoint().c_http_endpoint_name(*i).clone())
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            let mut res = BTreeSet::new();
            for line in endpoints_list
                .lines()
                .map(|i| i.trim())
                .filter(|i| !i.is_empty())
            {
                let found = endpoints.iter().find(|endpoint| {
                    db.backend_http_endpoint().c_http_endpoint_name(**endpoint) == line
                });
                if let Some(found) = found {
                    if !res.insert(*found) {
                        return Err(
                            PlatformValidationError::AppIngressDuplicateEndpointInEndpointList {
                                deployment: db
                                    .backend_application_deployment()
                                    .c_deployment_name(deployment)
                                    .clone(),
                                application_name: db
                                    .backend_application()
                                    .c_application_name(application)
                                    .clone(),
                                endpoints_list: endpoints_list.clone(),
                                duplicate_endpoint: line.to_string(),
                            },
                        );
                    }
                } else {
                    return Err(PlatformValidationError::AppIngressEndpointNotFound {
                        deployment: db
                            .backend_application_deployment()
                            .c_deployment_name(deployment)
                            .clone(),
                        application_name: db
                            .backend_application()
                            .c_application_name(application)
                            .clone(),
                        non_existing_endpoint: line.to_string(),
                        endpoints_list: endpoints_list.clone(),
                        available_endpoints: available_endpoints(),
                    });
                }
            }

            Ok(res)
        },
    )
}

fn try_parse_http_arg(
    path: &str,
    input: &str,
) -> Result<(String, HttpArgumentType), PlatformValidationError> {
    match ARGUMENT_REGEX.captures(input) {
        Some(captures) => {
            let v_name = captures.get(1).unwrap().as_str();
            let v_type = captures.get(2).unwrap().as_str();

            let vtype = match v_type {
                "INT" => ValidHttpPrimitiveType::Int,
                "FLOAT" => ValidHttpPrimitiveType::Float,
                "BOOL" => ValidHttpPrimitiveType::Bool,
                "TEXT" => ValidHttpPrimitiveType::Text,
                _ => {
                    return Err(PlatformValidationError::HttpInvalidArgumentType {
                        allowed_types: vec!["INT", "FLOAT", "BOOL", "TEXT"],
                        full_path: path.to_string(),
                        segment: input.to_string(),
                        the_type: v_type.to_string(),
                    })
                }
            };

            let vtype = HttpArgumentType {
                the_type: vtype,
                is_multiple: captures.get(3).is_some(),
            };

            Ok((v_name.to_string(), vtype))
        }
        None => Err(PlatformValidationError::HttpCantParseQueryArgument {
            full_path: path.to_string(),
            actual_segment: input.to_string(),
            expected_segment_example: "{some_variable:INT}",
        }),
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PageMethod {
    GET,
    POST,
    PUT,
}

impl ToString for PageMethod {
    fn to_string(&self) -> String {
        match self {
            PageMethod::GET => "GET",
            PageMethod::POST => "POST",
            PageMethod::PUT => "PUT",
        }
        .to_string()
    }
}

#[derive(Debug, Clone)]
struct PageSlot<T: Clone> {
    get_page: Option<T>,
    post_page: Option<T>,
    put_page: Option<T>,
}

impl<T: Clone> PageSlot<T> {
    fn empty() -> PageSlot<T> {
        Self {
            get_page: None,
            post_page: None,
            put_page: None,
        }
    }

    #[cfg(test)]
    fn try_merge(&mut self, other: &PageSlot<T>) -> bool {
        if self.get_page.is_some() && other.get_page.is_some() {
            return false;
        }

        if self.post_page.is_some() && other.post_page.is_some() {
            return false;
        }

        if self.put_page.is_some() && other.put_page.is_some() {
            return false;
        }

        if other.get_page.is_some() {
            self.get_page = other.get_page.clone();
        }

        if other.post_page.is_some() {
            self.post_page = other.post_page.clone();
        }

        if other.put_page.is_some() {
            self.put_page = other.put_page.clone();
        }

        return true;
    }

    #[cfg(test)]
    fn pages_with_methods(&self) -> Vec<(PageMethod, T)> {
        let mut res = Vec::new();

        if let Some(p) = &self.get_page {
            res.push((PageMethod::GET, p.clone()));
        }

        if let Some(p) = &self.post_page {
            res.push((PageMethod::POST, p.clone()));
        }

        if let Some(p) = &self.put_page {
            res.push((PageMethod::PUT, p.clone()));
        }

        res
    }

    #[cfg(test)]
    fn dupe_pages(&self, other: &Self) -> Vec<(PageMethod, T, T)> {
        let mut res = Vec::new();

        let va = self.pages_with_methods();
        let vb = other.pages_with_methods();

        for (am, ap) in va.iter() {
            for (bm, bp) in vb.iter() {
                if am == bm {
                    res.push((am.clone(), ap.clone(), bp.clone()));
                }
            }
        }

        res
    }

    fn any_page(&self) -> Option<&T> {
        self.get_page
            .as_ref()
            .or(self.post_page.as_ref())
            .or(self.put_page.as_ref())
    }

    fn try_set_page_slot(&mut self, method: PageMethod, page: T) -> bool {
        match method {
            PageMethod::GET => {
                if self.get_page.is_none() {
                    self.get_page = Some(page);
                    return true;
                }
            }
            PageMethod::POST => {
                if self.post_page.is_none() {
                    self.post_page = Some(page);
                    return true;
                }
            }
            PageMethod::PUT => {
                if self.put_page.is_none() {
                    self.put_page = Some(page);
                    return true;
                }
            }
        }

        false
    }

    fn get_slot(&self, method: &PageMethod) -> Option<&T> {
        match method {
            PageMethod::GET => self.get_page.as_ref(),
            PageMethod::POST => self.post_page.as_ref(),
            PageMethod::PUT => self.put_page.as_ref(),
        }
    }

    fn dump_pages(&self, current_prefix: &[ValidHttpPathSegment], res: &mut Vec<HttpPathRoute<T>>) {
        if let Some(o) = &self.get_page {
            res.push(HttpPathRoute {
                forward_all: false,
                method: PageMethod::GET,
                source_path: current_prefix.to_vec(),
                value: o.clone(),
            });
        }

        if let Some(o) = &self.post_page {
            res.push(HttpPathRoute {
                forward_all: false,
                method: PageMethod::POST,
                source_path: current_prefix.to_vec(),
                value: o.clone(),
            });
        }

        if let Some(o) = &self.put_page {
            res.push(HttpPathRoute {
                forward_all: false,
                method: PageMethod::PUT,
                source_path: current_prefix.to_vec(),
                value: o.clone(),
            });
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpPathTree<T: Clone> {
    // pages like /foo /bar
    named_pages: BTreeMap<String, PageSlot<T>>,
    // regex pages which must never clash with other pages
    prefix_pages: BTreeMap<String, PageSlot<T>>,
    // pages that nest deeper from this level like
    // /foo/...
    // /bar/...
    recursive_pages: BTreeMap<String, HttpPathTree<T>>,
    // this layer contains an argument, like
    // /123
    // /user/321/attr/123
    arg_node: Option<(String, Box<HttpPathTreeArg<T>>)>,
    // this marks root of this index
    // /
    index_page: PageSlot<T>,
    // this marks root page and the rest of the pages also forward to this page
    root_page: Option<T>,
}

#[derive(Debug, Clone)]
pub struct HttpPathTreeArg<T: Clone> {
    // if page ends here
    this_page: PageSlot<T>,
    next_level: Option<HttpPathTree<T>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum HttpPathTreeCheckerErrors<T: Clone> {
    IndexPageLockedTwice {
        prev_page: T,
        duplicate_page: T,
        page_method: PageMethod,
    },
    DuplicateNamedPage {
        page_name: String,
        prev_page: T,
        duplicate_page: T,
        page_method: PageMethod,
    },
    PathSegmentCanBeVariableOrStaticPage {
        variable_page: T,
        static_page: T,
    },
    PathSegmentCanHaveOnlyOneArgumentVariable {
        prev_page: T,
        duplicate_page: T,
    },
    #[cfg(test)]
    DifferentArgNamesInSameSegment {
        prev_name: String,
        curr_name: String,
    },
    #[cfg(test)]
    DifferentPagesAtArgName {
        page_a: T,
        page_b: T,
        page_method: PageMethod,
    },
    ArgPageAlreadyTaken {
        prev_page: T,
        duplicate_page: T,
        page_method: PageMethod,
    },
    RootAndOtherKindPageInvariantViolated {
        root_page: T,
        other_page: T,
    },
    RootPageAlreadySet {
        prev_root_page: T,
        new_root_page: T,
    },
    NamedAndPrefixPageClash {
        named_path: String,
        regex_path: String,
        named_page: T,
        regex_page: T,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidHttpPathSegment {
    StaticPath(String),
    Prefix(String),
    Argument,
    Slash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpPathRoute<T: Clone> {
    pub forward_all: bool,
    pub method: PageMethod,
    pub source_path: Vec<ValidHttpPathSegment>,
    pub value: T,
}

impl<T: Clone> HttpPathTree<T> {
    pub fn root() -> Self {
        Self {
            named_pages: Default::default(),
            prefix_pages: Default::default(),
            recursive_pages: Default::default(),
            arg_node: Default::default(),
            index_page: PageSlot::empty(),
            root_page: None,
        }
    }

    // How to represent arguments????
    pub fn flat_pages(&self) -> Vec<HttpPathRoute<T>> {
        let mut res = Vec::new();
        let segment = vec![ValidHttpPathSegment::Slash];
        self.flat_pages_int(&segment, &mut res);
        res
    }

    fn flat_pages_int(
        &self,
        source_path: &[ValidHttpPathSegment],
        res: &mut Vec<HttpPathRoute<T>>,
    ) {
        if let Some(o) = &self.root_page {
            res.push(HttpPathRoute {
                forward_all: true,
                method: PageMethod::GET,
                source_path: source_path.to_vec(),
                value: o.clone(),
            });
        }

        self.index_page.dump_pages(source_path, res);

        for (k, v) in &self.named_pages {
            let mut source_path = source_path.to_vec();
            source_path.push(ValidHttpPathSegment::StaticPath(k.clone()));
            v.dump_pages(&source_path, res);
        }

        for (k, v) in &self.prefix_pages {
            let mut source_path = source_path.to_vec();
            source_path.push(ValidHttpPathSegment::Prefix(k.clone()));
            v.dump_pages(&source_path, res);
        }

        for (k, v) in &self.recursive_pages {
            let mut source_path = source_path.to_vec();
            source_path.push(ValidHttpPathSegment::StaticPath(k.clone()));
            source_path.push(ValidHttpPathSegment::Slash);
            v.flat_pages_int(&source_path, res);
        }

        if let Some((_, arg)) = &self.arg_node {
            let mut source_path = source_path.to_vec();
            source_path.push(ValidHttpPathSegment::Argument);
            arg.this_page.dump_pages(&source_path, res);
            if let Some(nl) = &arg.next_level {
                source_path.push(ValidHttpPathSegment::Slash);
                nl.flat_pages_int(&source_path, res);
            }
        }
    }

    #[cfg(test)]
    pub fn try_merge(
        &mut self,
        other: &HttpPathTree<T>,
    ) -> Result<(), HttpPathTreeCheckerErrors<T>> {
        if !self.index_page.try_merge(&other.index_page) {
            let dupe_pages = self.index_page.dupe_pages(&other.index_page);
            let (pm, a, b) = dupe_pages
                .first()
                .expect("Must have dupes if logic is correct");
            return Err(HttpPathTreeCheckerErrors::IndexPageLockedTwice {
                prev_page: a.clone(),
                duplicate_page: b.clone(),
                page_method: pm.clone(),
            });
        }

        if let Some(root_page) = &other.root_page {
            self.lock_root(root_page.clone())?;
        }

        for (np, p) in &other.named_pages {
            for (pm, p) in p.pages_with_methods() {
                self.add_named_page(np.as_str(), pm, p.clone())?;
            }
        }

        for (np, p) in &other.prefix_pages {
            for (pm, p) in p.pages_with_methods() {
                self.add_prefix_page(np.as_str(), pm, p)?;
            }
        }

        for (an, it) in &other.arg_node {
            if self.arg_node.is_some() {
                let (n, t_arg) = self.arg_node.as_mut().unwrap();
                if n != an {
                    return Err(HttpPathTreeCheckerErrors::DifferentArgNamesInSameSegment {
                        prev_name: n.clone(),
                        curr_name: an.clone(),
                    });
                }

                if !t_arg.this_page.try_merge(&it.this_page) {
                    let dupe_pages = t_arg.this_page.dupe_pages(&it.this_page);
                    let (pm, a, b) = dupe_pages
                        .first()
                        .expect("Must have dupes if merge check failed");
                    return Err(HttpPathTreeCheckerErrors::DifferentPagesAtArgName {
                        page_a: a.clone(),
                        page_b: b.clone(),
                        page_method: pm.clone(),
                    });
                }

                if let Some(s) = t_arg.next_level.as_mut() {
                    if let Some(oth) = &it.next_level {
                        s.try_merge(oth)?;
                    }
                } else {
                    if let Some(oth) = &it.next_level {
                        t_arg.next_level = Some(oth.clone());
                    }
                }
            } else {
                self.arg_node = Some((an.clone(), it.clone()));
            }
        }

        for (k, v) in &other.recursive_pages {
            if let Some(this_v) = self.recursive_pages.get_mut(k) {
                this_v.try_merge(v)?;
            } else {
                assert!(self.recursive_pages.insert(k.clone(), v.clone()).is_none());
                self.check_arg_exclusion_violated(self.any_page().unwrap())?;
            }
        }

        Ok(())
    }

    pub fn lock_root(&mut self, this_page: T) -> Result<(), HttpPathTreeCheckerErrors<T>> {
        if self.root_page.is_some() {
            return Err(HttpPathTreeCheckerErrors::RootPageAlreadySet {
                prev_root_page: self.root_page.as_ref().unwrap().clone(),
                new_root_page: this_page,
            });
        }

        self.root_page = Some(this_page.clone());
        self.check_arg_exclusion_violated(&this_page)?;

        Ok(())
    }

    pub fn add_argument_final(
        &mut self,
        arg_name: &str,
        page_method: PageMethod,
        this_page: T,
    ) -> Result<(), HttpPathTreeCheckerErrors<T>> {
        if self.arg_node.is_some() {
            let (arg_n, tree) = self.arg_node.as_mut().unwrap();
            if arg_n != arg_name {
                return Err(
                    HttpPathTreeCheckerErrors::PathSegmentCanHaveOnlyOneArgumentVariable {
                        prev_page: tree
                            .this_page
                            .any_page()
                            .or_else(|| tree.next_level.as_ref().and_then(|i| i.any_page()))
                            .unwrap()
                            .clone(),
                        duplicate_page: this_page,
                    },
                );
            }

            if !tree
                .this_page
                .try_set_page_slot(page_method.clone(), this_page.clone())
            {
                return Err(HttpPathTreeCheckerErrors::ArgPageAlreadyTaken {
                    prev_page: tree.this_page.get_slot(&page_method).unwrap().clone(),
                    duplicate_page: this_page,
                    page_method,
                });
            }
        } else {
            let mut page_slot = PageSlot::empty();
            assert!(page_slot.try_set_page_slot(page_method, this_page.clone()));
            self.arg_node = Some((
                arg_name.to_string(),
                Box::new(HttpPathTreeArg {
                    this_page: page_slot,
                    next_level: None,
                }),
            ));
        }

        self.check_arg_exclusion_violated(&this_page)?;

        Ok(())
    }

    pub fn add_argument_with_tree(
        &mut self,
        arg_name: &str,
        failover_page: &T,
    ) -> Result<&mut HttpPathTree<T>, HttpPathTreeCheckerErrors<T>> {
        if self.arg_node.is_some() {
            if self.arg_node.as_ref().unwrap().0 != arg_name {
                let (_, tree) = self.arg_node.as_ref().unwrap();
                return Err(
                    HttpPathTreeCheckerErrors::PathSegmentCanHaveOnlyOneArgumentVariable {
                        prev_page: tree
                            .this_page
                            .any_page()
                            .or_else(|| tree.next_level.as_ref().and_then(|i| i.any_page()))
                            .unwrap()
                            .clone(),
                        duplicate_page: failover_page.clone(),
                    },
                );
            }
        } else {
            self.arg_node = Some((
                arg_name.to_string(),
                Box::new(HttpPathTreeArg {
                    this_page: PageSlot::empty(),
                    next_level: Some(HttpPathTree::root()),
                }),
            ));
        }

        if self.arg_node.as_ref().unwrap().1.next_level.is_none() {
            self.arg_node.as_mut().unwrap().1.next_level = Some(HttpPathTree::root());
        }

        self.check_arg_exclusion_violated(failover_page)?;

        Ok(self
            .arg_node
            .as_mut()
            .unwrap()
            .1
            .next_level
            .as_mut()
            .unwrap())
    }

    pub fn add_named_page(
        &mut self,
        page_name: &str,
        page_method: PageMethod,
        page: T,
    ) -> Result<(), HttpPathTreeCheckerErrors<T>> {
        assert!(
            HTTP_SEGMENT_REGEX.is_match(page_name),
            "We assume this component only reaches valid paths"
        );
        let e = self
            .named_pages
            .entry(page_name.to_string())
            .or_insert_with(|| PageSlot::empty());
        if !e.try_set_page_slot(page_method.clone(), page.clone()) {
            return Err(HttpPathTreeCheckerErrors::DuplicateNamedPage {
                page_name: page_name.to_string(),
                prev_page: e.any_page().unwrap().clone(),
                duplicate_page: page,
                page_method,
            });
        }
        self.check_arg_exclusion_violated(&page)?;
        self.perform_prefix_page_check()?;
        Ok(())
    }

    pub fn add_prefix_page(
        &mut self,
        page_prefix: &str,
        page_method: PageMethod,
        page: T,
    ) -> Result<(), HttpPathTreeCheckerErrors<T>> {
        let e = self
            .prefix_pages
            .entry(page_prefix.to_string())
            .or_insert_with(|| PageSlot::empty());
        if !e.try_set_page_slot(page_method.clone(), page.clone()) {
            return Err(HttpPathTreeCheckerErrors::DuplicateNamedPage {
                page_name: page_prefix.to_string(),
                prev_page: e.any_page().unwrap().clone(),
                duplicate_page: page,
                page_method,
            });
        }
        self.perform_prefix_page_check()?;
        Ok(())
    }

    fn perform_prefix_page_check(&self) -> Result<(), HttpPathTreeCheckerErrors<T>> {
        for (prefix, psr) in &self.prefix_pages {
            for (np, psn) in &self.named_pages {
                if np.starts_with(prefix) {
                    return Err(HttpPathTreeCheckerErrors::NamedAndPrefixPageClash {
                        named_path: np.clone(),
                        regex_path: prefix.to_string(),
                        named_page: psn.any_page().unwrap().clone(),
                        regex_page: psr.any_page().unwrap().clone(),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn fetch_level(
        &mut self,
        level_name: &str,
        failover_page: &T,
    ) -> Result<&mut HttpPathTree<T>, HttpPathTreeCheckerErrors<T>> {
        assert!(
            HTTP_SEGMENT_REGEX.is_match(level_name),
            "We assume this component only reaches valid paths"
        );
        let _ = self
            .recursive_pages
            .entry(level_name.to_string())
            .or_insert_with(|| HttpPathTree::root());
        self.check_arg_exclusion_violated(failover_page)?;
        Ok(self.recursive_pages.get_mut(level_name).unwrap())
    }

    pub fn lock_index(
        &mut self,
        page_method: PageMethod,
        input: T,
    ) -> Result<(), HttpPathTreeCheckerErrors<T>> {
        if let Some(prev_page) = self.index_page.get_slot(&page_method) {
            return Err(HttpPathTreeCheckerErrors::IndexPageLockedTwice {
                prev_page: prev_page.clone(),
                duplicate_page: input,
                page_method,
            });
        }
        assert!(
            self.index_page
                .try_set_page_slot(page_method, input.clone()),
            "Must succeed because we checked above"
        );
        self.check_arg_exclusion_violated(&input)?;
        Ok(())
    }

    fn any_page(&self) -> Option<&T> {
        self.first_arg_page().or_else(|| self.first_non_arg_page())
    }

    fn first_arg_page(&self) -> Option<&T> {
        if let Some(e) = self.arg_node.as_ref() {
            if let Some(tp) = e.1.this_page.any_page() {
                return Some(tp);
            }

            if let Some(tp) = e.1.next_level.as_ref() {
                if let Some(tp) = tp.any_page() {
                    return Some(tp);
                }
            }
        }

        None
    }

    fn first_non_arg_page(&self) -> Option<&T> {
        if let Some(r) = self.index_page.any_page() {
            return Some(r);
        }

        if let Some(e) = self.named_pages.iter().next() {
            if let Some(ap) = e.1.any_page() {
                return Some(ap);
            }
        }

        for rp in self.recursive_pages.values() {
            if let Some(tp) = rp.any_page() {
                return Some(tp);
            }
        }

        None
    }

    // /root/other
    // /root/other/123
    // /root/<arg> invalid
    // /root/ invalid
    // 1. there can only be either arg node
    // 2. or there can be recursive pages inside
    // 3. or slash pages
    // 4. arg node can only be one
    // 5. root page can be added whenever
    // and we should be good
    fn check_arg_exclusion_violated(
        &self,
        failover_page: &T,
    ) -> Result<(), HttpPathTreeCheckerErrors<T>> {
        let var_or_static_violated = self.arg_node.is_some()
            && (!self.named_pages.is_empty()
                || !self.recursive_pages.is_empty()
                || !self.prefix_pages.is_empty());

        if var_or_static_violated {
            return Err(
                HttpPathTreeCheckerErrors::PathSegmentCanBeVariableOrStaticPage {
                    variable_page: self.first_arg_page().unwrap_or(failover_page).clone(),
                    static_page: self.first_non_arg_page().unwrap_or(failover_page).clone(),
                },
            );
        }

        let root_violated = (self.arg_node.is_some()
            || !self.named_pages.is_empty()
            || !self.recursive_pages.is_empty())
            && self.root_page.is_some();

        if root_violated {
            return Err(
                HttpPathTreeCheckerErrors::RootAndOtherKindPageInvariantViolated {
                    root_page: self.root_page.as_ref().unwrap().clone(),
                    other_page: self.any_page().unwrap_or(failover_page).clone(),
                },
            );
        }

        Ok(())
    }
}

#[test]
fn test_http_path_tree_checker_double_root() {
    let mut tc = HttpPathTree::<i32>::root();

    assert_eq!(Ok(()), tc.lock_index(PageMethod::GET, 7));
    assert_eq!(
        Err(HttpPathTreeCheckerErrors::IndexPageLockedTwice {
            prev_page: 7,
            duplicate_page: 17,
            page_method: PageMethod::GET
        }),
        tc.lock_index(PageMethod::GET, 17)
    );
}

#[test]
fn test_http_path_tree_checker_double_argument() {
    let mut tc = HttpPathTree::<i32>::root();

    tc.add_argument_final("arg", PageMethod::GET, 7).unwrap();
    assert_eq!(
        HttpPathTreeCheckerErrors::PathSegmentCanHaveOnlyOneArgumentVariable {
            prev_page: 7,
            duplicate_page: 17,
        },
        tc.add_argument_with_tree("other", &17).unwrap_err()
    );
}

#[test]
fn test_http_path_tree_checker_double_argument_2() {
    let mut tc = HttpPathTree::<i32>::root();

    let a = tc.add_argument_with_tree("arg", &7).unwrap();
    a.lock_index(PageMethod::GET, 7).unwrap();
    assert_eq!(
        HttpPathTreeCheckerErrors::PathSegmentCanHaveOnlyOneArgumentVariable {
            prev_page: 7,
            duplicate_page: 17,
        },
        tc.add_argument_final("other", PageMethod::GET, 17)
            .unwrap_err()
    );
}

#[test]
fn test_http_path_tree_checker_args_with_constants() {
    let mut tc = HttpPathTree::<i32>::root();

    let r = tc.add_argument_with_tree("arg", &7).unwrap();
    r.lock_index(PageMethod::GET, 7).unwrap();
    assert_eq!(
        HttpPathTreeCheckerErrors::PathSegmentCanBeVariableOrStaticPage {
            variable_page: 7,
            static_page: 17,
        },
        tc.add_named_page("named", PageMethod::GET, 17).unwrap_err()
    );
}

#[test]
fn test_http_path_tree_checker_args_with_constants_level() {
    let mut tc = HttpPathTree::<i32>::root();

    let r = tc.add_argument_with_tree("arg", &7).unwrap();
    r.lock_index(PageMethod::GET, 7).unwrap();
    assert_eq!(
        HttpPathTreeCheckerErrors::PathSegmentCanBeVariableOrStaticPage {
            variable_page: 7,
            static_page: 17,
        },
        tc.fetch_level("named", &17).unwrap_err()
    );
}

#[test]
fn test_http_path_tree_checker_merge_double_root() {
    let mut tc = HttpPathTree::<i32>::root();
    let mut tc2 = HttpPathTree::<i32>::root();

    tc.lock_index(PageMethod::GET, 7).unwrap();
    tc2.lock_index(PageMethod::GET, 17).unwrap();

    assert_eq!(
        HttpPathTreeCheckerErrors::IndexPageLockedTwice {
            prev_page: 7,
            duplicate_page: 17,
            page_method: PageMethod::GET,
        },
        tc.try_merge(&tc2).unwrap_err()
    );
}

#[test]
fn test_http_path_tree_checker_merge_arg_exclusion() {
    let mut tc = HttpPathTree::<i32>::root();
    let mut tc2 = HttpPathTree::<i32>::root();

    tc.add_argument_final("arg", PageMethod::GET, 7).unwrap();
    tc2.add_named_page("mookie", PageMethod::GET, 17).unwrap();

    assert_eq!(
        HttpPathTreeCheckerErrors::PathSegmentCanBeVariableOrStaticPage {
            variable_page: 7,
            static_page: 17,
        },
        tc.try_merge(&tc2).unwrap_err()
    );
}

#[test]
fn test_http_path_tree_checker_merge_double_arg() {
    let mut tc = HttpPathTree::<i32>::root();
    let mut tc2 = HttpPathTree::<i32>::root();

    // ok can have two...
    // 1. none... just nested then?
    // 2. how about we transform it to fetch then?
    // So api for us locking argument page is always providing argument.
    // If we want to nest deeper we fetch nested?
    tc.add_argument_final("arg", PageMethod::GET, 7).unwrap();
    tc2.add_argument_final("arg", PageMethod::GET, 17).unwrap();

    assert_eq!(
        HttpPathTreeCheckerErrors::DifferentPagesAtArgName {
            page_a: 7,
            page_b: 17,
            page_method: PageMethod::GET,
        },
        tc.try_merge(&tc2).unwrap_err()
    );
}

#[test]
fn test_http_path_tree_checker_merge_page_and_level() {
    let mut tc = HttpPathTree::<()>::root();
    let mut tc2 = HttpPathTree::<()>::root();

    tc.add_named_page("arg", PageMethod::GET, ()).unwrap();
    let _ = tc2.fetch_level("mookie", &()).unwrap();

    assert!(tc.try_merge(&tc2).is_ok());
}

#[test]
fn test_http_path_tree_checker_merge_nested_different_double_root() {
    let mut tc = HttpPathTree::<()>::root();
    let mut tc2 = HttpPathTree::<()>::root();

    tc.fetch_level("dookie", &())
        .unwrap()
        .lock_index(PageMethod::GET, ())
        .unwrap();
    tc2.fetch_level("mookie", &())
        .unwrap()
        .lock_index(PageMethod::GET, ())
        .unwrap();

    assert!(tc.try_merge(&tc2).is_ok());
}

#[test]
fn test_http_path_tree_checker_merge_nested_same_double_root() {
    let mut tc = HttpPathTree::<i32>::root();
    let mut tc2 = HttpPathTree::<i32>::root();

    tc.fetch_level("mookie", &0)
        .unwrap()
        .lock_index(PageMethod::GET, 7)
        .unwrap();
    tc2.fetch_level("mookie", &0)
        .unwrap()
        .lock_index(PageMethod::GET, 17)
        .unwrap();

    assert_eq!(
        HttpPathTreeCheckerErrors::IndexPageLockedTwice {
            prev_page: 7,
            duplicate_page: 17,
            page_method: PageMethod::GET,
        },
        tc.try_merge(&tc2).unwrap_err()
    );
}

#[test]
fn test_http_path_tree_checker_merge_arg_same_name() {
    let mut tc = HttpPathTree::<()>::root();
    let mut tc2 = HttpPathTree::<()>::root();

    tc.add_argument_final("mookie", PageMethod::GET, ())
        .unwrap();
    tc2.add_argument_with_tree("mookie", &()).unwrap();

    assert_eq!(Ok(()), tc.try_merge(&tc2));
}

#[test]
fn test_http_path_tree_checker_merge_nested() {
    let mut tc = HttpPathTree::<i32>::root();
    let mut tc2 = HttpPathTree::<i32>::root();

    let nested = tc.add_argument_with_tree("mookie", &0).unwrap();
    nested
        .add_named_page("salookie", PageMethod::GET, 7)
        .unwrap();
    let nested = tc2.add_argument_with_tree("mookie", &0).unwrap();
    nested
        .add_named_page("salookie", PageMethod::GET, 17)
        .unwrap();

    assert_eq!(
        Err(HttpPathTreeCheckerErrors::DuplicateNamedPage {
            page_name: "salookie".to_string(),
            prev_page: 7,
            duplicate_page: 17,
            page_method: PageMethod::GET
        }),
        tc.try_merge(&tc2)
    );
}

#[test]
fn test_http_path_tree_checker_merge_root() {
    let mut tc = HttpPathTree::<i32>::root();
    let mut tc2 = HttpPathTree::<i32>::root();

    let nested = tc.add_argument_with_tree("mookie", &0).unwrap();
    nested.lock_index(PageMethod::GET, 7).unwrap();
    let nested = tc2.add_argument_with_tree("mookie", &0).unwrap();
    nested.lock_index(PageMethod::GET, 17).unwrap();

    assert_eq!(
        Err(HttpPathTreeCheckerErrors::IndexPageLockedTwice {
            prev_page: 7,
            duplicate_page: 17,
            page_method: PageMethod::GET,
        }),
        tc.try_merge(&tc2)
    );
}

#[test]
fn test_http_path_tree_checker_merge_add_finals() {
    let mut tc = HttpPathTree::<i32>::root();
    let mut tc2 = HttpPathTree::<i32>::root();

    tc.add_argument_final("mookie", PageMethod::GET, 7).unwrap();
    tc2.add_argument_final("mookie", PageMethod::GET, 17)
        .unwrap();

    assert_eq!(
        Err(HttpPathTreeCheckerErrors::DifferentPagesAtArgName {
            page_a: 7,
            page_b: 17,
            page_method: PageMethod::GET,
        }),
        tc.try_merge(&tc2)
    );
}

#[test]
fn test_http_path_root_set_twice() {
    let mut tc = HttpPathTree::<i32>::root();

    tc.lock_root(7).unwrap();
    let e = tc.lock_root(17);

    assert_eq!(
        Err(HttpPathTreeCheckerErrors::RootPageAlreadySet {
            prev_root_page: 7,
            new_root_page: 17
        }),
        e
    );
}

#[test]
fn test_http_path_root_and_named() {
    let mut tc = HttpPathTree::<i32>::root();

    tc.add_named_page("mookie", PageMethod::GET, 7).unwrap();
    let e = tc.lock_root(17);

    assert_eq!(
        Err(
            HttpPathTreeCheckerErrors::RootAndOtherKindPageInvariantViolated {
                root_page: 17,
                other_page: 7
            }
        ),
        e
    );
}

#[test]
fn test_http_path_root_and_arg() {
    let mut tc = HttpPathTree::<i32>::root();

    tc.add_argument_final("mookie", PageMethod::GET, 7).unwrap();
    let e = tc.lock_root(17);

    assert_eq!(
        Err(
            HttpPathTreeCheckerErrors::RootAndOtherKindPageInvariantViolated {
                root_page: 17,
                other_page: 7
            }
        ),
        e
    );
}

#[test]
fn test_http_path_root_and_arg_recur() {
    let mut tc = HttpPathTree::<i32>::root();

    tc.lock_root(17).unwrap();
    let e = tc.add_argument_with_tree("mookie", &7).unwrap_err();

    assert_eq!(
        HttpPathTreeCheckerErrors::RootAndOtherKindPageInvariantViolated {
            root_page: 17,
            other_page: 7
        },
        e
    );
}

#[test]
fn test_http_path_root_and_page_recur() {
    let mut tc = HttpPathTree::<i32>::root();

    tc.lock_root(17).unwrap();
    let e = tc.fetch_level("mookie", &7).unwrap_err();

    assert_eq!(
        HttpPathTreeCheckerErrors::RootAndOtherKindPageInvariantViolated {
            root_page: 17,
            other_page: 7
        },
        e
    );
}

#[test]
fn test_http_path_merge_two_roots() {
    let mut tc = HttpPathTree::<i32>::root();
    let mut tc2 = HttpPathTree::<i32>::root();

    tc.lock_root(7).unwrap();
    tc2.lock_root(17).unwrap();

    assert_eq!(
        Err(HttpPathTreeCheckerErrors::RootPageAlreadySet {
            prev_root_page: 7,
            new_root_page: 17
        }),
        tc.try_merge(&tc2)
    );
}

#[test]
fn test_http_path_merge_root_and_named() {
    let mut tc = HttpPathTree::<i32>::root();
    let mut tc2 = HttpPathTree::<i32>::root();

    tc.lock_root(7).unwrap();
    tc2.add_named_page("mookie", PageMethod::GET, 17).unwrap();

    assert_eq!(
        Err(
            HttpPathTreeCheckerErrors::RootAndOtherKindPageInvariantViolated {
                root_page: 7,
                other_page: 17
            }
        ),
        tc.try_merge(&tc2)
    );
}

#[test]
fn test_http_path_same_page_diff_methods() {
    let mut tc = HttpPathTree::<i32>::root();

    tc.add_named_page("mookie", PageMethod::GET, 7).unwrap();
    assert_eq!(Ok(()), tc.add_named_page("mookie", PageMethod::POST, 17));
}

#[test]
fn test_http_path_same_page_same_methods() {
    let mut tc = HttpPathTree::<i32>::root();

    tc.add_named_page("mookie", PageMethod::POST, 7).unwrap();
    assert_eq!(
        Err(HttpPathTreeCheckerErrors::DuplicateNamedPage {
            page_name: "mookie".to_string(),
            prev_page: 7,
            duplicate_page: 17,
            page_method: PageMethod::POST
        }),
        tc.add_named_page("mookie", PageMethod::POST, 17)
    );
}

#[test]
fn test_http_path_index_page_same_methods() {
    let mut tc = HttpPathTree::<i32>::root();

    tc.lock_index(PageMethod::PUT, 7).unwrap();
    assert_eq!(
        Err(HttpPathTreeCheckerErrors::IndexPageLockedTwice {
            prev_page: 7,
            duplicate_page: 17,
            page_method: PageMethod::PUT
        }),
        tc.lock_index(PageMethod::PUT, 17)
    );
}

#[test]
fn test_http_path_arg_page_same_methods() {
    let mut tc = HttpPathTree::<i32>::root();

    tc.add_argument_final("arg", PageMethod::PUT, 7).unwrap();
    assert_eq!(
        Err(HttpPathTreeCheckerErrors::ArgPageAlreadyTaken {
            prev_page: 7,
            duplicate_page: 17,
            page_method: PageMethod::PUT
        }),
        tc.add_argument_final("arg", PageMethod::PUT, 17)
    );
}

#[test]
fn test_http_path_same_index_diff_methods() {
    let mut tc = HttpPathTree::<i32>::root();

    tc.lock_index(PageMethod::GET, 7).unwrap();
    assert_eq!(Ok(()), tc.lock_index(PageMethod::POST, 17));
}

#[test]
fn test_http_path_same_arg_final_diff_methods() {
    let mut tc = HttpPathTree::<i32>::root();

    tc.add_argument_final("arg", PageMethod::GET, 7).unwrap();
    assert_eq!(Ok(()), tc.add_argument_final("arg", PageMethod::PUT, 17));
}

#[test]
fn test_http_path_arg_nested_diff_named_methods() {
    let mut tc = HttpPathTree::<i32>::root();

    let inner = tc.add_argument_with_tree("arg", &7).unwrap();
    let mut id = 7;
    let mut next_id = || {
        let res = id;
        id += 1;
        res
    };
    assert_eq!(Ok(()), inner.lock_index(PageMethod::GET, next_id()));
    assert_eq!(Ok(()), inner.lock_index(PageMethod::POST, next_id()));
    assert_eq!(Ok(()), inner.lock_index(PageMethod::PUT, next_id()));
    assert_eq!(
        Ok(()),
        inner.add_named_page("named", PageMethod::GET, next_id())
    );
    assert_eq!(
        Ok(()),
        inner.add_named_page("named", PageMethod::POST, next_id())
    );
    assert_eq!(
        Ok(()),
        inner.add_named_page("named", PageMethod::PUT, next_id())
    );
}

#[test]
fn test_http_path_arg_nested_diff_arg_methods() {
    let mut tc = HttpPathTree::<i32>::root();

    let inner = tc.add_argument_with_tree("arg", &7).unwrap();
    let mut id = 7;
    let mut next_id = || {
        let res = id;
        id += 1;
        res
    };
    assert_eq!(
        Ok(()),
        inner.add_argument_final("named", PageMethod::GET, next_id())
    );
    assert_eq!(
        Ok(()),
        inner.add_argument_final("named", PageMethod::POST, next_id())
    );
    assert_eq!(
        Ok(()),
        inner.add_argument_final("named", PageMethod::PUT, next_id())
    );
}

#[test]
fn test_http_path_named_nested_diff_named_methods() {
    let mut tc = HttpPathTree::<i32>::root();

    let inner = tc.fetch_level("level", &7).unwrap();
    let mut id = 7;
    let mut next_id = || {
        let res = id;
        id += 1;
        res
    };
    assert_eq!(Ok(()), inner.lock_index(PageMethod::GET, next_id()));
    assert_eq!(Ok(()), inner.lock_index(PageMethod::POST, next_id()));
    assert_eq!(Ok(()), inner.lock_index(PageMethod::PUT, next_id()));
    assert_eq!(
        Ok(()),
        inner.add_named_page("named", PageMethod::GET, next_id())
    );
    assert_eq!(
        Ok(()),
        inner.add_named_page("named", PageMethod::POST, next_id())
    );
    assert_eq!(
        Ok(()),
        inner.add_named_page("named", PageMethod::PUT, next_id())
    );
}

#[test]
fn test_http_path_named_nested_diff_arg_methods() {
    let mut tc = HttpPathTree::<i32>::root();

    let inner = tc.fetch_level("level", &7).unwrap();
    let mut id = 7;
    let mut next_id = || {
        let res = id;
        id += 1;
        res
    };
    assert_eq!(
        Ok(()),
        inner.add_argument_final("named", PageMethod::GET, next_id())
    );
    assert_eq!(
        Ok(()),
        inner.add_argument_final("named", PageMethod::POST, next_id())
    );
    assert_eq!(
        Ok(()),
        inner.add_argument_final("named", PageMethod::PUT, next_id())
    );
}

#[test]
fn test_http_path_named_dump_paths() {
    let mut tc = HttpPathTree::<i32>::root();

    let mut id = 7;
    let mut next_id = || {
        let res = id;
        id += 1;
        res
    };
    tc.lock_index(PageMethod::POST, next_id()).unwrap();
    let inner = tc.fetch_level("level", &7).unwrap();
    assert_eq!(
        Ok(()),
        inner.add_named_page("named", PageMethod::GET, next_id())
    );
    assert_eq!(
        Ok(()),
        inner.add_named_page("named", PageMethod::POST, next_id())
    );
    assert_eq!(
        Ok(()),
        inner.add_named_page("named", PageMethod::PUT, next_id())
    );

    let paths = tc.flat_pages();
    assert_eq!(
        paths,
        vec![
            HttpPathRoute {
                forward_all: false,
                method: PageMethod::POST,
                source_path: vec![ValidHttpPathSegment::Slash],
                value: 7,
            },
            HttpPathRoute {
                forward_all: false,
                method: PageMethod::GET,
                source_path: vec![
                    ValidHttpPathSegment::Slash,
                    ValidHttpPathSegment::StaticPath("level".to_string()),
                    ValidHttpPathSegment::Slash,
                    ValidHttpPathSegment::StaticPath("named".to_string()),
                ],
                value: 8,
            },
            HttpPathRoute {
                forward_all: false,
                method: PageMethod::POST,
                source_path: vec![
                    ValidHttpPathSegment::Slash,
                    ValidHttpPathSegment::StaticPath("level".to_string()),
                    ValidHttpPathSegment::Slash,
                    ValidHttpPathSegment::StaticPath("named".to_string()),
                ],
                value: 9,
            },
            HttpPathRoute {
                forward_all: false,
                method: PageMethod::PUT,
                source_path: vec![
                    ValidHttpPathSegment::Slash,
                    ValidHttpPathSegment::StaticPath("level".to_string()),
                    ValidHttpPathSegment::Slash,
                    ValidHttpPathSegment::StaticPath("named".to_string()),
                ],
                value: 10,
            },
        ]
    );
}

#[test]
fn test_http_path_arg_dump_paths() {
    let mut tc = HttpPathTree::<i32>::root();

    let mut id = 7;
    let mut next_id = || {
        let res = id;
        id += 1;
        res
    };
    tc.lock_index(PageMethod::POST, next_id()).unwrap();
    let inner = tc.fetch_level("level", &7).unwrap();
    assert_eq!(
        Ok(()),
        inner.add_argument_final("named", PageMethod::GET, next_id())
    );
    assert_eq!(
        Ok(()),
        inner.add_argument_final("named", PageMethod::POST, next_id())
    );
    assert_eq!(
        Ok(()),
        inner.add_argument_final("named", PageMethod::PUT, next_id())
    );

    let paths = tc.flat_pages();
    assert_eq!(
        paths,
        vec![
            HttpPathRoute {
                forward_all: false,
                method: PageMethod::POST,
                source_path: vec![ValidHttpPathSegment::Slash],
                value: 7,
            },
            HttpPathRoute {
                forward_all: false,
                method: PageMethod::GET,
                source_path: vec![
                    ValidHttpPathSegment::Slash,
                    ValidHttpPathSegment::StaticPath("level".to_string()),
                    ValidHttpPathSegment::Slash,
                    ValidHttpPathSegment::Argument,
                ],
                value: 8,
            },
            HttpPathRoute {
                forward_all: false,
                method: PageMethod::POST,
                source_path: vec![
                    ValidHttpPathSegment::Slash,
                    ValidHttpPathSegment::StaticPath("level".to_string()),
                    ValidHttpPathSegment::Slash,
                    ValidHttpPathSegment::Argument,
                ],
                value: 9,
            },
            HttpPathRoute {
                forward_all: false,
                method: PageMethod::PUT,
                source_path: vec![
                    ValidHttpPathSegment::Slash,
                    ValidHttpPathSegment::StaticPath("level".to_string()),
                    ValidHttpPathSegment::Slash,
                    ValidHttpPathSegment::Argument,
                ],
                value: 10,
            },
        ]
    );
}

#[test]
fn test_http_dump_paths_root() {
    let mut tc = HttpPathTree::<i32>::root();

    let mut id = 7;
    let mut next_id = || {
        let res = id;
        id += 1;
        res
    };
    tc.lock_root(next_id()).unwrap();

    let paths = tc.flat_pages();
    assert_eq!(
        paths,
        vec![HttpPathRoute {
            forward_all: true,
            method: PageMethod::GET,
            source_path: vec![ValidHttpPathSegment::Slash],
            value: 7,
        },]
    );
}

#[test]
fn test_prefix_page_clash() {
    let mut tc = HttpPathTree::<i32>::root();

    let mut id = 7;
    let mut next_id = || {
        let res = id;
        id += 1;
        res
    };
    let _ = tc
        .add_named_page("epl-app-123", PageMethod::GET, next_id())
        .unwrap();
    let err = tc
        .add_prefix_page("epl-app-", PageMethod::GET, next_id())
        .unwrap_err();

    assert_eq!(
        err,
        HttpPathTreeCheckerErrors::NamedAndPrefixPageClash {
            named_path: "epl-app-123".to_string(),
            regex_path: "epl-app-".to_string(),
            named_page: 7,
            regex_page: 8,
        }
    );
}

#[test]
fn test_prefix_page_diff_http_methods_clash() {
    let mut tc = HttpPathTree::<i32>::root();

    let mut id = 7;
    let mut next_id = || {
        let res = id;
        id += 1;
        res
    };
    let _ = tc
        .add_named_page("epl-app-123", PageMethod::GET, next_id())
        .unwrap();
    let err = tc
        .add_prefix_page("epl-app-", PageMethod::POST, next_id())
        .unwrap_err();

    assert_eq!(
        err,
        HttpPathTreeCheckerErrors::NamedAndPrefixPageClash {
            named_path: "epl-app-123".to_string(),
            regex_path: "epl-app-".to_string(),
            named_page: 7,
            regex_page: 8,
        }
    );
}

#[test]
fn test_parse_http_arguments() {
    assert!(!ARGUMENT_REGEX.is_match("{hello_1:I64} "));
    assert!(!ARGUMENT_REGEX.is_match(" {hello_1:I64}"));
    assert!(!ARGUMENT_REGEX.is_match("{ hello_1:I64}"));
    assert!(!ARGUMENT_REGEX.is_match("{hello_1 :I64}"));
    assert!(!ARGUMENT_REGEX.is_match("{hello_1: I64}"));
    assert!(!ARGUMENT_REGEX.is_match("{hello_1:I64 >"));
    let success = ARGUMENT_REGEX.captures("{hello_1:I64}").unwrap();
    assert_eq!(success.get(1).unwrap().as_str(), "hello_1");
    assert_eq!(success.get(2).unwrap().as_str(), "I64");
    assert!(success.get(3).is_none());

    let success = ARGUMENT_REGEX.captures("{hello_1:I64[]}").unwrap();
    assert_eq!(success.get(1).unwrap().as_str(), "hello_1");
    assert_eq!(success.get(2).unwrap().as_str(), "I64");
    assert!(success.get(3).is_some());
}
