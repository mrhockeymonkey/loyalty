use yew::prelude::*;
use stylist::{css};
use stylist::yew::Global;
use yew_router::prelude::*;

use crate::pages::collect::Collect;
use crate::pages::display::Display;
use crate::pages::stamp_card::StampCard;

mod pages;
mod components;

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

// TODO is there a better way
fn get_api_base() -> &'static str {
    return if cfg!(feature = "prod") {
        "https://7oz-loyalty.shuttleapp.rs"
    } else {
        "http://localhost:8000"
    };
}

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