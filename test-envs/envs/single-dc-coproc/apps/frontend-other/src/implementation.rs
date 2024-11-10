
use yew::prelude::*;
use crate::generated::AppRouteExt;

pub fn switch(route: AppRouteExt) -> Html {
    match route {
        AppRouteExt::Home => html! {
            <h1>{"Other app home"}</h1>
        },
        AppRouteExt::SingleArg { id } => html! {
            <h1>{"Other app"}{id}</h1>
        },
        AppRouteExt::OtherPage => html! {
            <h1>{"Other dummy page"}</h1>
        },
        AppRouteExt::AllArg { id, name, opt_i, opt_m } => html! {
            <h1>{"All arg id:"}{id}{" name:"}{name}{" opt_i:"}{opt_i}{" opt_m:"}{opt_m}</h1>
        },
    }
}
