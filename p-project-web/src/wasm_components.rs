use p_project_core::{models::User, utils::shorten_wallet_address};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct WebUser {
    inner: User,
}

#[wasm_bindgen]
impl WebUser {
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, username: String, wallet_address: String) -> WebUser {
        WebUser {
            inner: User {
                id,
                username,
                wallet_address,
                created_at: chrono::Utc::now().naive_utc(),
            },
        }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn username(&self) -> String {
        self.inner.username.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn wallet_address(&self) -> String {
        self.inner.wallet_address.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn short_wallet_address(&self) -> String {
        shorten_wallet_address(&self.inner.wallet_address)
    }

    #[wasm_bindgen]
    pub fn to_string(&self) -> String {
        format!(
            "User({}, {})",
            self.inner.username,
            self.short_wallet_address()
        )
    }
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to P-Project!", name)
}

#[wasm_bindgen]
pub fn initialize_app() {
    log("P-Project Web App Initialized");
}
