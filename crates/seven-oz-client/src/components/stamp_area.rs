use stylist::Style;
use yew::{function_component, html, Html, Properties};

const STYLE: &str = include_str!("stamp_area.css");

#[derive(Properties, PartialEq)]
pub struct StampAreaProps {
    pub is_stamped: bool
}

#[function_component]
pub fn StampArea(props: &StampAreaProps) -> Html {
    let stylesheet = Style::new(STYLE).unwrap();
    
    html! {
        <div class={ stylesheet }>
            <div>
                { 
                    if props.is_stamped {
                        html!{
                            <p>{ "7oz" }</p>
                        }
                    }
                    else {
                        html!{}
                    }
                }
            </div>
        </div>
    }
}