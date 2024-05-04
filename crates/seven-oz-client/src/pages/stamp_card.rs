use std::time::Duration;
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use stylist::Style;
use stylist::yew::use_style;
use stylist::global_style;
use wasm_bindgen_futures::js_sys::JsString;
use web_sys::{console, window};
use yew::{AttrValue, Callback, Component, Context, Html, html, Properties};
use yew::platform::time::sleep;
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
    StampsReceived(u32)
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
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        html! {
            <div class="container text-center">
                <div class="row col">
                    <h1 class="display-1 py-3">{ "Your Loyalty Card" }</h1>
                </div>

                <div class="row">
                    <div class="col"></div>
                    <div class="col-10">
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
                            </div>
                        </div>
                        <div class="pt-5">
                            if (self.query.clone() == REDEEM_PARAM) {
                                <button type="button" class="btn btn-danger btn-lg">{ "Redeem" }</button>
                            }
                            else {
                                <QrCodeImage link={ format!("{}{}", self.location, REDEEM_PARAM) } />
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
        let endpoint = format!("{}/api/card/{}", api_base, id);

        let resp = Request::get(&endpoint)
            //.header("Access-Control-Allow-Origin", "http://localhost:8000/")
            .send()
            .await.unwrap()
            .json::<crate::pages::stamp_card::CardResponse>()
            .await.unwrap();

        console::log_1(&JsString::from(
            serde_json::to_string(&resp).unwrap()
        ));

        code_cb.emit(resp.stamps.into());
    });
}