use regex::Regex;
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::wasm_bindgen::JsValue;
use web_sys::{console, HtmlInputElement};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{get_api_base, Route};

#[derive(Properties, PartialEq)]
pub struct CollectProps {
    pub code: String,
}

pub enum CollectMsg {
    Submit,
    Claiming,
    ClaimOk(String),
    ClaimFail
}

#[derive(Deserialize, Serialize)]
struct Claim {
    id: String,
    code: String
}

pub struct Collect {
    input_ref: NodeRef
}

impl Component for Collect {
    type Message = CollectMsg;
    type Properties = CollectProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            input_ref: NodeRef::default()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            CollectMsg::Submit => {
                if let Some(input) = self.input_ref.cast::<HtmlInputElement>() {
                    // Access the value of the input element
                    let input_value = input.value();
                    console::log_1(&input_value.clone().into());

                    // check validity
                    let r = Regex::new(r"^07\d{9}$").unwrap();
                    if (!r.is_match(input_value.as_ref())){
                        input.class_list().add_1("is-invalid");
                        return false;
                    }
                    
                    let claim = Claim {
                        id:  input_value.clone(),
                        code: ctx.props().code.clone()
                    };
                    
                    ctx.link().send_future(async {
                        match post_claim(claim).await {
                            Ok(md) => CollectMsg::ClaimOk(input_value),
                            Err(err) => CollectMsg::ClaimFail,
                        }
                    });
                    ctx.link().send_message(CollectMsg::Claiming);
                }
                false
            },
            CollectMsg::Claiming => {
                console::log_1(&JsValue::from("Claiming"));
                false
            },
            CollectMsg::ClaimOk(id) => {
                console::log_1(&JsValue::from("ClaimOk"));
                let navigator = ctx.link().navigator().unwrap();
                navigator.push(&Route::StampCard{id});
                false
            },
            CollectMsg::ClaimFail => {
                console::log_1(&JsValue::from("ClaimFail"));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="container text-center">
                <div class="row">
                    <div class="col">
                        <h1 class="display-1 py-3">{"Collect a Stamp"}</h1>
    
                        <form novalidate=true>
                            <div class="mb-3">
                                <label for="phone_number" class="form-label">{"Phone Number"}</label>
                                <input type="tel"
                                    class="form-control"
                                    id="phone_number"
                                    name="phone_number"
                                    aria-describedby="phone_number_help"
                                    ref={&self.input_ref} 
                                    placeholder="07715559999"/>
                                <div class="invalid-feedback">
                                    { "Please enter a valid UK mobile number without spaces" }
                                </div>
                            </div>
            
                            <button type="button"
                                class="btn btn-primary"
                                onclick={ctx.link().callback(|_| CollectMsg::Submit)}>
                                {"Stamp"}
                            </button>
                        </form>
                    </div>
                </div>
            </div>
        }
    }
}

async fn post_claim(claim: Claim) -> Result<(), u16> {
    let json = serde_json::to_string(&claim).unwrap();
    let api_base = get_api_base();
    let endpoint = format!("{}/api/customercode/claim", api_base);
    let resp = Request::post(&endpoint)
        .body(json)
        .header("Content-Type", "application/json")
        .send().await.unwrap();
    
    match resp.status() {
        200 => Ok(()),
        _ => Err(resp.status())
    }
}
