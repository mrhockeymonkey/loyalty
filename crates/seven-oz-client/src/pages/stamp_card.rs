use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::js_sys::JsString;
use wasm_bindgen_futures::wasm_bindgen::JsValue;
use web_sys::console;
use yew::{Callback, Component, Context, Html, html, Properties};
use yew_router::prelude::RouterScopeExt;

use crate::components::qrcode_image::QrCodeImage;
use crate::components::stamp_area::StampArea;
use crate::get_api_base;

const REDEEM_PARAM: &str = "?redeem=1";

pub struct StampCard {
    stamp_count: u32,
    query: String,
    location: String
}

pub enum StampCardMsg {
    StampsReceived(u32),
    StampsResetRequested,
    StampsResetOk,
    StampsResetErr(u16)
}

#[derive(Properties, PartialEq)]
pub struct StampCardProps {
    pub id: String
}

impl Component for StampCard {
    type Message = StampCardMsg;
    type Properties = StampCardProps;

    fn create(ctx: &Context<Self>) -> Self {
        let card_callback = ctx.link().callback(StampCardMsg::StampsReceived);
        get_stamp_card(card_callback, ctx.props().id.clone());
        let location = ctx.link().location().unwrap();
        let query = ctx.link().location().unwrap().query_str().to_string();
        
        Self {
            stamp_count: 0,
            query,
            location: location.path().to_string()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            StampCardMsg::StampsReceived(count) => {
                self.stamp_count = count;
                true
            },
            StampCardMsg::StampsResetRequested => {
                console::log_1(&JsValue::from("Reset requested"));
                let card_id = ctx.props().id.clone();
                ctx.link().send_future(async {
                    match reset_stamp_card(card_id).await {
                        Ok(()) => StampCardMsg::StampsResetOk,
                        Err(err) => StampCardMsg::StampsResetErr(err),
                    }
                });

                false
            },
            StampCardMsg::StampsResetOk => {
                console::log_1(&JsValue::from("Reset OK"));
                
                // TODO replace this redirect with some live polling and refresh screen with "coffee on the way" animation
                web_sys::window().unwrap().location().set_href(self.location.as_ref()).unwrap();
                false
            },
            StampCardMsg::StampsResetErr(response_code) => {
                // TODO some sort of visual feedback for error
                console::log_1(&JsValue::from(format!("Reset Error: {}", response_code)));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        html! {
            <div class="container-fluid text-center" style="height:100vh">
                <div class="row col">
                    <h1 class="display-1 py-3">{ "Your Loyalty Card" }</h1>
                </div>

                <div class="row" style="height:100vh">
                    <div class="col"></div>
                    <div class="col-10 d-flex flex-column">
                        <div class="card text-bg-light">
                            <div class="card-header">
                                { "7oz" }
                            </div>
                            <div class="card-body">
                                <div class="row col">
                                    <div class="d-flex justify-content-between">
                                        <StampArea is_stamped={self.stamp_count>=1} />
                                        <StampArea is_stamped={self.stamp_count>=2} />
                                        <StampArea is_stamped={self.stamp_count>=3} />
                                    </div>
                                </div>
                                <div class="row col py-2"></div>
                                <div class="row col">
                                    <div class="d-flex justify-content-between">
                                        <StampArea is_stamped={self.stamp_count>=4} />
                                        <StampArea is_stamped={self.stamp_count>=5} />
                                        <StampArea is_stamped={self.stamp_count>=6} />
            
                                    </div>
                                </div>
                                <div class="row col py-2"></div>
                                <div class="row col">
                                    <div class="d-flex justify-content-between">
                                        <StampArea is_stamped={self.stamp_count>=7} />
                                        <StampArea is_stamped={self.stamp_count>=8} />
                                        <StampArea is_stamped={self.stamp_count>=9} />
                                    </div>
                                </div>
                                <div class="row col py-2"></div>
                                <div class="row col">
                                    <div class="d-flex justify-content-between">
                                        <div style="visibility:hidden">
                                            <StampArea is_stamped={false} />
                                        </div>
                                        <StampArea is_stamped={self.stamp_count>=10} />
                                        <div style="visibility:hidden">
                                            <StampArea is_stamped={false} />
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="mt-auto" style="height:300px">
                            if self.query.clone() == REDEEM_PARAM {
                                <button type="button"
                                    onclick={ctx.link().callback(|_| StampCardMsg::StampsResetRequested)}
                                    class="btn btn-danger btn-lg">{ "Redeem" }</button>
                            }
                            else {
                                <QrCodeImage link={ format!("{}{}", self.location, REDEEM_PARAM)} dim={150} module_dim={4} />
                            }
                        </div>
                    </div>
                    <div class="col"></div>
                </div>
            </div>
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
struct CardResponse {
    stamps: u32
}

fn get_stamp_card(code_cb: Callback<u32>, id: String) {
    wasm_bindgen_futures::spawn_local(async move {
        let api_base = get_api_base();
        let endpoint = format!("{}/api/stampcard/{}", api_base, id);

        let resp = Request::get(&endpoint)
            //.header("Access-Control-Allow-Origin", "http://localhost:8000/")
            .send()
            .await.unwrap()
            .json::<CardResponse>()
            .await.unwrap();

        console::log_1(&JsString::from(
            serde_json::to_string(&resp).unwrap()
        ));

        code_cb.emit(resp.stamps.into());
    });
}

async fn reset_stamp_card(id: String) -> Result<(), u16> {
    let api_base = get_api_base();
    let endpoint = format!("{}/api/stampcard/{}/reset", api_base, id);

    let resp = Request::post(&endpoint)
        .send()
        .await.unwrap();

    match resp.status() {
        200 => Ok(()),
        _ => Err(resp.status())
    }
}