
use yew_router::prelude::*;
use yew::prelude::*;
use crate::generated::*;

pub fn switch(route: AppRouteExt) -> Html {
    match route {
        AppRouteExt::Home => html! { <TheHome/> },
        AppRouteExt::SingleArg { id } => html! {
            <>
            <h1>{"Single Arg id:"}{id}</h1>
            <Link<AppRoute> classes="button_c" to={AppRoute::Home}>{"go home"}</Link<AppRoute>>
            </>
        },
        AppRouteExt::NextPage { id, farg, arg_a, arg_b, arg_c, arg_d, arg_e, arg_f, arg_g, arg_h } => {
            html! {
                <>
                <h1>{"Next Page id:"}{id}{" farg:"}{farg}</h1>
                <Link<AppRoute> classes="button_d" to={AppRoute::Home}>{"go home"}</Link<AppRoute>>
                <ul>
                    <li>{"arg_a:"}{format!("{:?}", arg_a)}</li>
                    <li>{"arg_b:"}{format!("{:?}", arg_b)}</li>
                    <li>{"arg_c:"}{format!("{:?}", arg_c)}</li>
                    <li>{"arg_d:"}{format!("{:?}", arg_d)}</li>
                    <li>{"arg_e:"}{format!("{:?}", arg_e)}</li>
                    <li>{"arg_f:"}{format!("{:?}", arg_f)}</li>
                    <li>{"arg_g:"}{format!("{:?}", arg_g)}</li>
                    <li>{"arg_h:"}{format!("{:?}", arg_h)}</li>
                </ul>
                </>
            }
        },
        AppRouteExt::TextArg { text } => {
            html! {
                <h1>{"Hello text: "}{text}</h1>
            }
        },
        AppRouteExt::RestTest => html! { <RestTest/> },
        AppRouteExt::LinksTest => {
            let url_backend = backend_link_be_hello_world(
                UrlLBeHelloWorldArgs {
                    arg: "hello".to_string(),
                    more: true,
                },
                UrlLBeHelloWorldOptArgs {
                    floot: vec![0.7, 7.7],
                    other: Some(777),
                },
            );
            let url_frontend = page_link_fe_all_arg(
                UrlPFeAllArgArgs {
                    id: 7,
                    name: "some name".to_string(),
                },
                UrlPFeAllArgOptArgs {
                    opt_i: Some(17),
                    opt_m: vec![0.7, 7.7],
                },
            );
            html! {
                <>
                    <h1>{"Links test"}</h1>
                    <a class="to_backend" href={url_backend}>{"to backend"}</a>
                    <a class="to_frontend" href={url_frontend}>{"to frontend"}</a>
                </>
            }
        }
    }
}

#[function_component(RestTest)]
fn rest_test() -> Html {
    let vtype = use_state(|| BwTypeTestVtype {
        coordinates: None,
        is_nice: false,
        other_field: -1.23,
        some_field: -123,
    });
    let onclick_button = {
        let vtype = vtype.clone();
        Callback::from(move |_| {
            let vtype = vtype.clone();
            let args = ApiFirstArgs {
                input_body: BwTypeTestVtype {
                    coordinates: None,
                    is_nice: true,
                    other_field: 7.7,
                    some_field: 777,
                },
            };
            let oargs = ApiFirstOptArgs {
                // These are added to other_field
                floatv_arg: vec![3.0, 2.0, 1.7],
                // these are added to some_field
                int_arg: Some(17),
            };
            apicl_first(args, oargs, move |res| {
                if let Ok(res) = res {
                    vtype.set(res);
                }
            })
        })
    };

    html! {
        <>
        <h1>
            {"Rest test"}
        </h1>
        <p>{"is nice: "}{vtype.is_nice}{", of: "}{vtype.other_field}{", sf: "}{vtype.some_field}</p>
        <button onclick={onclick_button}>{"do the thing"}</button>
        </>
    }
}


#[function_component(TheHome)]
fn the_home() -> Html {
    let navigator = use_navigator().unwrap();
    let onclick_a = Callback::from(move |_| navigator.push(&AppRouteExt::SingleArg { id: 123 }));
    let navigator = use_navigator().unwrap();
    let onclick_b = Callback::from(move |_| navigator.push(&AppRouteExt::NextPage {
        id: 321, farg: 7.77,
        arg_a: Some(true),
        arg_b: Vec::new(),
        arg_c: Some(42),
        arg_d: Vec::new(),
        arg_e: Some(0.0000000000001),
        arg_f: Vec::new(),
        arg_g: None,
        arg_h: vec!["salookie".to_string(), "dookie".to_string(), "хелло".to_string(), "#*#@#@!$)@#*%#^)_".to_string()],
    }));

    html! {
        <>
        <h1>
            {"Hello world!"}
        </h1>
        <button class="button_a" onclick={onclick_a}>{"go single arg"}</button>
        <button class="button_b" onclick={onclick_b}>{"go all args"}</button>
        </>
    }
}
