
use yew::prelude::*;
use yew_router::prelude::*;


#[derive(Clone, PartialEq, Routable)]
pub enum AppRoute {
    #[at("/")]
    Home,
    #[at("/single/:id")]
    SingleArg { id: i64 },
    #[at("/next/:id/:farg")]
    NextPage { id: i64, farg: f64 },
    #[at("/targ/:text")]
    TextArg { text: String },
    #[at("/rest_test/")]
    RestTest,
    #[at("/links_test/")]
    LinksTest,
}

#[derive(Clone, PartialEq)]
pub enum AppRouteExt {
    Home,
    SingleArg { id: i64 },
    NextPage { id: i64, farg: f64, arg_a: Option<bool>, arg_b: Vec<bool>, arg_c: Option<i64>, arg_d: Vec<i64>, arg_e: Option<f64>, arg_f: Vec<f64>, arg_g: Option<String>, arg_h: Vec<String> },
    TextArg { text: String },
    RestTest,
    LinksTest,
}

impl Routable for AppRouteExt {
    fn from_path(_path: &str, _params: &std::collections::HashMap<&str, &str>) -> Option<Self> {
        None
    }

    fn to_path(&self) -> String {
        match self {
            AppRouteExt::Home => {
                format!("/")
            },
            AppRouteExt::SingleArg { id } => {
                format!("/single/{}", id)
            },
            AppRouteExt::NextPage { id, farg, arg_a, arg_b, arg_c, arg_d, arg_e, arg_f, arg_g, arg_h } => {
                let mut res = format!("/next/{}/{}", id, farg);
                let mut arg_added = false;
                let mut append = |i: &str| {
                    if arg_added {
                        res += "&";
                    } else {
                        res += "?";
                        arg_added = true;
                    }
                    res += i;
                };
                arg_a.iter().for_each(|arg| append(&format!("arg_a={arg}")));
                arg_b.iter().for_each(|arg| append(&format!("arg_b={arg}")));
                arg_c.iter().for_each(|arg| append(&format!("arg_c={arg}")));
                arg_d.iter().for_each(|arg| append(&format!("arg_d={arg}")));
                arg_e.iter().for_each(|arg| append(&format!("arg_e={arg}")));
                arg_f.iter().for_each(|arg| append(&format!("arg_f={arg}")));
                arg_g.iter().for_each(|arg| append(&format!("arg_g={}", ::urlencoding::encode(arg))));
                arg_h.iter().for_each(|arg| append(&format!("arg_h={}", ::urlencoding::encode(arg))));
                res
            },
            AppRouteExt::TextArg { text } => {
                format!("/targ/{}", ::urlencoding::encode(text))
            },
            AppRouteExt::RestTest => {
                format!("/rest_test/")
            },
            AppRouteExt::LinksTest => {
                format!("/links_test/")
            },

        }
    }

    fn routes() -> Vec<&'static str> {
        vec![]
    }

    fn not_found_route() -> Option<Self> {
        None
    }

    fn recognize(_pathname: &str) -> Option<Self> {
        None
    }
}

#[function_component(Main)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<AppRoute> render={switch} />
        </BrowserRouter>
    }
}

fn switch(route: AppRoute) -> Html {
    let route_ext: AppRouteExt = match route {
        AppRoute::Home => {
            AppRouteExt::Home
        },
        AppRoute::SingleArg { id } => {
            AppRouteExt::SingleArg { id }
        },
        AppRoute::NextPage { id, farg } => {
            let current_url_args = current_url_args();
            let arg_a = current_url_args.get("arg_a").map(|v| v.parse::<bool>().ok()).flatten();
            let arg_b = current_url_args.get_all("arg_b").iter().filter_map(|i| i.as_string().map(|j| j.parse::<bool>().ok()).flatten()).collect::<Vec<_>>();
            let arg_c = current_url_args.get("arg_c").map(|v| v.parse::<i64>().ok()).flatten();
            let arg_d = current_url_args.get_all("arg_d").iter().filter_map(|i| i.as_string().map(|j| j.parse::<i64>().ok()).flatten()).collect::<Vec<_>>();
            let arg_e = current_url_args.get("arg_e").map(|v| v.parse::<f64>().ok()).flatten();
            let arg_f = current_url_args.get_all("arg_f").iter().filter_map(|i| i.as_string().map(|j| j.parse::<f64>().ok()).flatten()).collect::<Vec<_>>();
            let arg_g = current_url_args.get("arg_g");
            let arg_h = current_url_args.get_all("arg_h").iter().filter_map(|i| i.as_string()).collect::<Vec<_>>();
            AppRouteExt::NextPage { id, farg, arg_a, arg_b, arg_c, arg_d, arg_e, arg_f, arg_g, arg_h }
        },
        AppRoute::TextArg { text } => {
            AppRouteExt::TextArg { text }
        },
        AppRoute::RestTest => {
            AppRouteExt::RestTest
        },
        AppRoute::LinksTest => {
            AppRouteExt::LinksTest
        },
    };
    crate::implementation::switch(route_ext)
}

fn current_url_args() -> web_sys::UrlSearchParams {
    web_sys::UrlSearchParams::new_with_str(
        &gloo::utils::document().location().unwrap().search().unwrap()
    ).unwrap()
}

pub struct ApiFirstArgs {
    pub input_body: BwTypeTestVtype,
}

#[derive(Default)]
pub struct ApiFirstOptArgs {
    pub floatv_arg: Vec<f64>,
    pub int_arg: Option<i64>,
}

pub async fn api_first(required_args: ApiFirstArgs, optional_args: ApiFirstOptArgs) -> Result<BwTypeTestVtype, Box<dyn ::std::error::Error>> {
    let mut url = get_endpoint_mapping("eplEndpointMapping", "first").unwrap_or_else(|| "/".to_string());
    url += "dummy";
    let mut query_args = Vec::new();
    for arg in &optional_args.floatv_arg {
        query_args.push(("floatv_arg", format!("{}", arg)));
    }
    if let Some(arg) = &optional_args.int_arg {
        query_args.push(("int_arg", format!("{}", arg)));
    }
    let fetch = ::gloo_net::http::Request::post(&url)
        .query(query_args)
        .header("Content-Type", "application/json")
        .body(bw_type_test_vtype_serialize_json(&required_args.input_body))?
        .send()
        .await?
        .binary()
        .await?;
    let deser = bw_type_test_vtype_deserialize_json(&fetch)?;
    Ok(deser)
}

#[allow(dead_code)]
pub fn apicl_first(required_args: ApiFirstArgs, optional_args: ApiFirstOptArgs, closure: impl FnOnce(Result<BwTypeTestVtype, Box<dyn ::std::error::Error>>) + 'static) {
    wasm_bindgen_futures::spawn_local(async move {
        closure(api_first(required_args, optional_args).await);
    })
}

#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
pub struct BwTypeTestVtypeV1 {
    pub some_field: i64,
}
#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
pub struct BwTypeTestVtypeV2 {
    pub other_field: f64,
    pub some_field: i64,
}
#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
pub struct BwTypeTestVtypeV3Aux1 {
    pub x: f64,
    pub y: f64,
}
#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
pub struct BwTypeTestVtypeV3 {
    pub coordinates: Option<BwTypeTestVtypeV3Aux1>,
    pub other_field: f64,
    pub some_field: i64,
}
#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
pub struct BwTypeTestVtypeV4Aux1 {
    pub x: f64,
    pub y: f64,
}
#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
pub struct BwTypeTestVtypeV4 {
    pub coordinates: Option<BwTypeTestVtypeV4Aux1>,
    pub is_good: bool,
    pub nickname: String,
    pub other_field: f64,
    pub some_field: i64,
}
#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
pub struct BwTypeTestVtypeV5Aux1 {
    pub x: f64,
    pub y: f64,
}
#[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
pub struct BwTypeTestVtypeV5 {
    pub coordinates: Option<BwTypeTestVtypeV5Aux1>,
    pub is_nice: bool,
    pub other_field: f64,
    pub some_field: i64,
}
pub type BwTypeTestVtype = BwTypeTestVtypeV5;
fn bw_type_test_vtype_deserialize_json(input: &[u8]) -> Result<BwTypeTestVtype, JsonDeserializationError> {
    if let Ok(the_val) = ::serde_json::from_slice::<BwTypeTestVtypeV5>(input) {
        return Ok(the_val);
    }
    if let Ok(the_val) = ::serde_json::from_slice::<BwTypeTestVtypeV4>(input) {
        let the_val = bw_type_test_vtype_v4_to_v5(the_val);
        return Ok(the_val);
    }
    if let Ok(the_val) = ::serde_json::from_slice::<BwTypeTestVtypeV3>(input) {
        let the_val = bw_type_test_vtype_v3_to_v4(the_val);
        let the_val = bw_type_test_vtype_v4_to_v5(the_val);
        return Ok(the_val);
    }
    if let Ok(the_val) = ::serde_json::from_slice::<BwTypeTestVtypeV2>(input) {
        let the_val = bw_type_test_vtype_v2_to_v3(the_val);
        let the_val = bw_type_test_vtype_v3_to_v4(the_val);
        let the_val = bw_type_test_vtype_v4_to_v5(the_val);
        return Ok(the_val);
    }
    if let Ok(the_val) = ::serde_json::from_slice::<BwTypeTestVtypeV1>(input) {
        let the_val = bw_type_test_vtype_v1_to_v2(the_val);
        let the_val = bw_type_test_vtype_v2_to_v3(the_val);
        let the_val = bw_type_test_vtype_v3_to_v4(the_val);
        let the_val = bw_type_test_vtype_v4_to_v5(the_val);
        return Ok(the_val);
    }
    return Err(JsonDeserializationError::UnknownType);
}

fn bw_type_test_vtype_v1_to_v2(input: BwTypeTestVtypeV1) -> BwTypeTestVtypeV2 {
    BwTypeTestVtypeV2 {
        other_field: 1.23,
        some_field: input.some_field,
    }
}
fn bw_type_test_vtype_v2_to_v3(input: BwTypeTestVtypeV2) -> BwTypeTestVtypeV3 {
    BwTypeTestVtypeV3 {
        coordinates: None,
        other_field: input.other_field,
        some_field: input.some_field,
    }
}
fn bw_type_test_vtype_v3_to_v4(input: BwTypeTestVtypeV3) -> BwTypeTestVtypeV4 {
    BwTypeTestVtypeV4 {
        coordinates: input.coordinates.map(|input| BwTypeTestVtypeV4Aux1 {
            x: input.x,
            y: input.y,
        }),
        is_good: true,
        nickname: r#"who knows"#.to_string(),
        other_field: input.other_field,
        some_field: input.some_field,
    }
}
fn bw_type_test_vtype_v4_to_v5(input: BwTypeTestVtypeV4) -> BwTypeTestVtypeV5 {
    BwTypeTestVtypeV5 {
        coordinates: input.coordinates.map(|input| BwTypeTestVtypeV5Aux1 {
            x: input.x,
            y: input.y,
        }),
        is_nice: input.is_good,
        other_field: input.other_field,
        some_field: input.some_field,
    }
}
fn bw_type_test_vtype_serialize_json(input: &BwTypeTestVtype) -> String {
    ::serde_json::to_string(input).expect("should never happen")
}


#[derive(Debug)]
pub enum JsonDeserializationError {
    UnknownType,
}

impl std::fmt::Display for JsonDeserializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "json deserialization error: {:?}", self)
    }
}

impl std::error::Error for JsonDeserializationError {}

pub struct UrlPFeAllArgArgs {
    pub id: i64,
    pub name: String,
}

#[derive(Default)]
pub struct UrlPFeAllArgOptArgs {
    pub opt_i: Option<i64>,
    pub opt_m: Vec<f64>,
}

pub fn page_link_fe_all_arg(required_args: UrlPFeAllArgArgs, optional_args: UrlPFeAllArgOptArgs) -> EplUrl {
    let mut url = get_endpoint_mapping("eplExtPagesMapping", "fe_all_arg").unwrap_or_else(|| "/".to_string());
    url += "all";
    url += "/";
    url += &::urlencoding::encode(&format!("{}", required_args.id));
    url += "/";
    url += &::urlencoding::encode(&required_args.name);
    let mut opt_added = false;
    if let Some(arg) = &optional_args.opt_i {
        #[allow(unused_assignments)]
        if opt_added { url += "&" } else { opt_added = true; url += "?"; }
        url += "opt_i=";
        url += &::urlencoding::encode(&format!("{}", arg));
    }
    for arg in &optional_args.opt_m {
        #[allow(unused_assignments)]
        if opt_added { url += "&" } else { opt_added = true; url += "?"; }
        url += "opt_m=";
        url += &::urlencoding::encode(&format!("{}", arg));
    }
    EplUrl { url }
}

pub struct UrlLBeHelloWorldArgs {
    pub arg: String,
    pub more: bool,
}

#[derive(Default)]
pub struct UrlLBeHelloWorldOptArgs {
    pub floot: Vec<f64>,
    pub other: Option<i64>,
}

pub fn backend_link_be_hello_world(required_args: UrlLBeHelloWorldArgs, optional_args: UrlLBeHelloWorldOptArgs) -> EplUrl {
    let mut url = get_endpoint_mapping("eplExtLinksMapping", "be_hello_world").unwrap_or_else(|| "/".to_string());
    url += "hello_world";
    url += "/";
    url += &::urlencoding::encode(&required_args.arg);
    url += "/";
    url += &::urlencoding::encode(&format!("{}", required_args.more));
    let mut opt_added = false;
    for arg in &optional_args.floot {
        #[allow(unused_assignments)]
        if opt_added { url += "&" } else { opt_added = true; url += "?"; }
        url += "floot=";
        url += &::urlencoding::encode(&format!("{}", arg));
    }
    if let Some(arg) = &optional_args.other {
        #[allow(unused_assignments)]
        if opt_added { url += "&" } else { opt_added = true; url += "?"; }
        url += "other=";
        url += &::urlencoding::encode(&format!("{}", arg));
    }
    EplUrl { url }
}


pub struct EplUrl {
    url: String,
}

impl EplUrl {
    /// Navigate in the same tab
    #[allow(dead_code)]
    pub fn navigate(&self) {
        let _ = ::gloo::utils::window().location().set_href(self.url.as_str());
    }

    /// Navigate in the new tab
    #[allow(dead_code)]
    pub fn navigate_new_tab(&self) {
        let _ = ::gloo::utils::window().open_with_url_and_target(self.url.as_str(), "_blank");
    }

    /// Get raw url
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        self.url.as_str()
    }
}

impl yew::html::IntoPropValue<AttrValue> for EplUrl {
    fn into_prop_value(self) -> AttrValue {
        AttrValue::Rc(std::rc::Rc::from(self.url.clone()))
    }
}

impl yew::html::IntoPropValue<Option<AttrValue>> for EplUrl {
    fn into_prop_value(self) -> Option<AttrValue> {
        Some(AttrValue::Rc(std::rc::Rc::from(self.url.clone())))
    }
}

fn get_endpoint_mapping(glob_var: &str, endpoint: &str) -> Option<String> {
    let mapping: ::js_sys::Object = gloo::utils::window().get(glob_var)
        .unwrap()
        .try_into()
        .unwrap();

    ::js_sys::Reflect::get(&mapping, &::wasm_bindgen::JsValue::from_str(endpoint)).ok().map(|i| i.as_string()).flatten()
}
