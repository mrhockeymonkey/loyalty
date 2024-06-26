use std::time::Duration;

use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::js_sys::JsString;
use web_sys::{console, window};
use yew::{AttrValue, Callback, Component, Context, Html, html};
use yew::platform::time::sleep;

use crate::components::qrcode_image::QrCodeImage;
use crate::get_api_base;

#[derive(Serialize, Deserialize, PartialEq)]
struct QrResponse {
    code: String
}

pub struct Display {
    location: String,
    code: Option<AttrValue>
}

pub enum DisplayMsg {
    CodeReceived(AttrValue)
}

impl Component for Display {
    type Message = DisplayMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let code_cb = ctx.link().callback(DisplayMsg::CodeReceived);
        poll_code_service(code_cb);
        Self {
            location: window().unwrap().location().origin().unwrap(),
            code: None
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            DisplayMsg::CodeReceived(new_code) => {
                if self.code.as_deref().unwrap_or("") == new_code {
                    return false;
                }

                console::log_1(&JsString::from("Received new code!"));
                self.code = Some(new_code);
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
            <div class="container text-center">
                <div class="row">
                    <div class="col">
                        <h1 class="display-1 py-3">{"Scan Me"}</h1>
                            {
                                match self.code.clone() {
                                    Some(code) => html!{
                                        <div>
                                            <div>
                                                <QrCodeImage link={ format!("{}/collect/{}", self.location, code) } dim=250 module_dim=7  />
                                            </div>
                                            <div>
                                                //<a href={ format!("{}/collect/{}", self.location, code) }>{ format!("{}/collect/{}", self.location, code) }</a>
                                            </div>
                                        </div>

                                    },
                                    None => html!{
                                        <div>{ "Loading..." }</div>
                                    }
                                }
                            }
                    </div>
                </div>
            </div>
            </>
        }
    }
}

fn poll_code_service(code_cb: Callback<AttrValue>) {
    wasm_bindgen_futures::spawn_local(async move {

        let api_base = get_api_base();
        let endpoint = format!("{}/api/customercode", api_base);

        loop {
            let resp = Request::get(&endpoint)
                //.header("Access-Control-Allow-Origin", "http://localhost:8000/")
                .send()
                .await.unwrap()
                .json::<QrResponse>()
                .await.unwrap();

            console::log_1(&JsString::from(
                serde_json::to_string(&resp).unwrap()
            ));

            code_cb.emit(resp.code.into());
            sleep(Duration::from_secs(2)).await
        }

    });
}