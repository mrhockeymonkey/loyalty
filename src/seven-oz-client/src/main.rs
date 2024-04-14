use yew::prelude::*;
use web_sys::{ Window, window, console};
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use stylist::{css, Style};
use stylist::yew::Global;
use web_sys::js_sys::JsString;
use yew_router::prelude::*;
use crate::pages::collect::Collect;
use crate::pages::display::Display;
use crate::pages::stamp_card::StampCard;

mod pages;
mod components;

//const API_BASE: &str = "https://7oz-loyalty.shuttleapp.rs";
const API_BASE: &str = "http://localhost:8000";

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Display,
    #[at("/collect/:code")]
    Collect{ code: String },
    #[at("/my-stamp-card/:id")]
    StampCard{ id: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Display => html!{
            <Display />
        },
        Route::Collect {code } => html! {
            <Collect code={code} />
        },
        Route::StampCard{id} => html!{
            <StampCard id={id}/>
        },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

// #[function_component(Secure)]
// fn secure() -> Html {
//     let navigator = use_navigator().unwrap();
// 
//     let onclick = Callback::from(move |_| navigator.push(&Route::Home));
//     html! {
//         <div>
//             <h1>{ "Secure" }</h1>
//             <button {onclick}>{ "Go Home" }</button>
//         </div>
//     }
// }

// #[function_component(App)]
// fn app() -> Html {
//     html! {
//         <>
//             <Global css={css!("background-color: red;")} />
//             <div>{"Hello World!"}</div>
//         </>
//     }
// }

#[function_component(App)]
fn app() -> Html {
    //let stylesheet = Style::new("body {background-color: lightslategrey;}").unwrap();
    html! {
        <>
        <Global css={css!(r#"
        body {
            background-color: lightslategrey;
        }
        
        h1,h3,label {
            color: white
        }
        
        "#)} />
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}