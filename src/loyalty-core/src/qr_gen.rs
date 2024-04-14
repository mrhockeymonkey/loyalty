use qrcode::QrCode;
use qrcode::render::Renderer;
pub use qrcode::render::svg::Color;
use rand::distributions::Alphanumeric;
use rand::Rng;
use crate::qr_gen;

pub fn rand_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect::<String>()
}

pub struct CustomerQrCode {
    pub code: String, // TODO does this mean its setable?
    qr: QrCode,
    used: bool
}

impl CustomerQrCode {
    pub fn new() -> Self {
        let code = rand_string(12);
        Self {
            code: code.clone(),
            qr: QrCode::new(code.as_bytes()).unwrap(),
            used: false
        }
    }
    
    pub fn render(&self) -> Renderer<Color> {
        self.qr.render()
    }
    
    pub fn is_used(&self) -> bool {
        self.used
    }
}

impl From<String> for CustomerQrCode {
    fn from(value: String) -> Self {
        Self {
            code: value.clone(),
            qr: QrCode::new(value.as_bytes()).unwrap(),
            used: false
        }
    }
}
