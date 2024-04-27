use std::rc::Rc;
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::wasm_bindgen::JsValue;
use web_sys::{console, HtmlInputElement, window};
use yew::prelude::*;
use yew::InputEvent;
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
                    // Use the input value as needed
                    //println!("Input value: {}", input_value);

                    console::log_1(&input_value.clone().into());

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
    
                        <div>
                            <div class="mb-3">
                                <label for="phone_number" class="form-label">{"Phone Number"}</label>
                                <input type="tel"
                                    class="form-control"
                                    id="phone_number"
                                    name="phone_number"
                                    aria-describedby="phone_number_help"
                                    pattern="[0-9]{11}"
                                    ref={&self.input_ref}/>
                                <div id="phone_number_help" class="form-text">{"We'll never share your phone number with anyone else."}</div>
                            </div>
    
                            <button type="button" class="btn btn-primary" onclick={ctx.link().callback(|_| CollectMsg::Submit)}>{"Stamp"}</button>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}





async fn post_claim(claim: Claim) -> Result<(), u16> {
    let json = serde_json::to_string(&claim).unwrap();
    let api_base = get_api_base();
    let endpoint = format!("{}/api/claim", api_base);
    let resp = Request::post(&endpoint)
        .body(json)
        .header("Content-Type", "application/json")
        .send().await.unwrap();
    
    match resp.status() {
        200 => Ok(()),
        _ => Err(resp.status())
    }
}
// 
// #[function_component(Collect1)]
// pub fn collect(props: &CollectProps) -> Html {
//     let api_base = "http://localhost:8000";
//     let collect_endpoint = format!("{}/api/qr", api_base);
//     //let input_ref = use_mut_ref(|| NodeRef::default());
//     let node_ref = NodeRef::default();
//     let node_refc = node_ref.clone();
//     let button_click2 = {
//         //let input_ref = Rc::clone(&node_ref);
//         Callback::from(move |me: MouseEvent| {
//             // Get the raw DOM element from the ref
//             if let Some(input) = &node_refc.cast::<HtmlInputElement>() {
//                 // Access the value of the input element
//                 let input_value = input.value();
//                 // Use the input value as needed
//                 //println!("Input value: {}", input_value);
// 
//                 web_sys::console::log_1(&input_value.clone().into());
// 
//                 let body_json = serde_json::to_string(&Claim {
//                     id: input_value,
//                     code: props.code.clone()
//                 }).unwrap();
// 
//                 wasm_bindgen_futures::spawn_local(async move {
//                     let result = Request::post(&collect_endpoint).body(body_json).send().await.unwrap();
//                 });
// 
// 
//             }
//         })
//     };
// 
// 
// 
// 
//     let input_value= use_state(|| String::new());
// 
//     let input_change = {
//         let input_value_c = input_value.clone();
//         Callback::from(move |e: InputEvent| {
//             let d = e.data().unwrap();
//             let dd = d.clone();
//             web_sys::console::log_1(&d.into());
//             input_value_c.set(dd);
//         })
//     };
//     let foo = "foo";
//     //let user_phone = use_state_eq(|| "".to_string());
//     //let user_phone_clone = user_phone.clone();  // TODO
//     //let oninput = Callback::from(move |e: InputEvent| user_phone_clone.set(e.data().unwrap()));
//     // let onclick = Callback::from(move |_| {
//     //     let iii = input_value.clone();
//     //     let greeting = ;
//     //     web_sys::console::log_1(&greeting.into());
//     // });
// 
//     // let button_click = {
//     //     let input_value = input_value.clone();
//     //     Callback::from(move |_| {
//     //         // Handle button click event
//     //         // You can access the value of the input field from input_value
//     //         dbg!(input_value);
//     //         //web_sys::console::log_1(&input_value.into());
//     //     })
//     // };
//     
//     html! {
//         <div class="container text-center">
//             <div class="row">
//                 <div class="col">
//                     <h1>{"Stamp Your Digital Card"}</h1>
//                     <h3>{&props.code}</h3>
//         
//                     <div>
//                         <div class="mb-3">
//                             <label for="phone_number" class="form-label">{"Phone Number"}</label>
//                             <input type="tel" class="form-control" id="phone_number" name="phone_number" aria-describedby="phone_number_help" pattern="[0-9]{11}"
//                                 ref={node_ref.clone()}
//                                 oninput={input_change}/>
//                             <div id="phone_number_help" class="form-text">{"We'll never share your phone number with anyone else."}</div>
//                         </div>
// 
//                         <button type="button" class="btn btn-primary" onclick={button_click2}>{"Stamp"}</button>
//                     </div>
//                 </div>
//             </div>
//         </div>
//     }
// }