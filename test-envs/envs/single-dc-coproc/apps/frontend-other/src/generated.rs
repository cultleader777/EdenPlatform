
use yew::prelude::*;
use yew_router::prelude::*;


#[derive(Clone, PartialEq, Routable)]
pub enum AppRoute {
    #[at("/")]
    Home,
    #[at("/single/:id")]
    SingleArg { id: i64 },
    #[at("/other/")]
    OtherPage,
    #[at("/all/:id/:name")]
    AllArg { id: i64, name: String },
}

#[derive(Clone, PartialEq)]
pub enum AppRouteExt {
    Home,
    SingleArg { id: i64 },
    OtherPage,
    AllArg { id: i64, name: String, opt_i: Option<i64>, opt_m: Vec<f64> },
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
            AppRouteExt::OtherPage => {
                format!("/other/")
            },
            AppRouteExt::AllArg { id, name, opt_i, opt_m } => {
                let mut res = format!("/all/{}/{}", id, ::urlencoding::encode(name));
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
                opt_i.iter().for_each(|arg| append(&format!("opt_i={arg}")));
                opt_m.iter().for_each(|arg| append(&format!("opt_m={arg}")));
                res
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
        AppRoute::OtherPage => {
            AppRouteExt::OtherPage
        },
        AppRoute::AllArg { id, name } => {
            let current_url_args = current_url_args();
            let opt_i = current_url_args.get("opt_i").map(|v| v.parse::<i64>().ok()).flatten();
            let opt_m = current_url_args.get_all("opt_m").iter().filter_map(|i| i.as_string().map(|j| j.parse::<f64>().ok()).flatten()).collect::<Vec<_>>();
            AppRouteExt::AllArg { id, name, opt_i, opt_m }
        },
    };
    crate::implementation::switch(route_ext)
}

fn current_url_args() -> web_sys::UrlSearchParams {
    web_sys::UrlSearchParams::new_with_str(
        &gloo::utils::document().location().unwrap().search().unwrap()
    ).unwrap()
}
