use yew::{AttrValue, function_component, Html, html, Properties};
use loyalty_core::qr_gen;
use loyalty_core::qr_gen::CustomerQrCode;

#[derive(Properties, PartialEq)]
pub struct QrCodeImageProps {
    pub link: AttrValue,
    pub dim: u32,
    pub module_dim: u32
}

// TODO disable onclick in production
#[function_component]
pub fn QrCodeImage(props: &QrCodeImageProps) -> Html {

    let qr: CustomerQrCode = String::from(props.link.as_str()).into();
    
    let image = qr
        .render()
        .min_dimensions(props.dim, props.dim)
        .module_dimensions(props.module_dim, props.module_dim)
        .dark_color(qr_gen::Color("#ffffff"))
        .light_color(qr_gen::Color("#778899"))
        .build();

    html! {
        <a href={ props.link.clone() }>
            <div>
                { Html::from_html_unchecked(AttrValue::from(image))}
            </div>
        </a>
    }
}