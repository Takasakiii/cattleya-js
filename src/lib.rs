#![allow(clippy::unused_unit)]

use std::sync::Arc;

use cattleya::request::{VioletLogData, VioletRequest};
use js_sys::{Promise, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[repr(C)]
pub enum ErrorsLevel {
    Severe,
    Error,
    Warning,
    Info,
    Debug,
    Verbose,
}

impl From<ErrorsLevel> for String {
    fn from(err: ErrorsLevel) -> Self {
        match err {
            ErrorsLevel::Severe => "Severe".to_string(),
            ErrorsLevel::Error => "Error".to_string(),
            ErrorsLevel::Warning => "Warning".to_string(),
            ErrorsLevel::Info => "Info".to_string(),
            ErrorsLevel::Debug => "Debug".to_string(),
            ErrorsLevel::Verbose => "Verbose".to_string(),
        }
    }
}

#[wasm_bindgen]
pub struct Cattleya {
    client: Arc<VioletRequest>,
}

#[wasm_bindgen]
impl Cattleya {
    #[wasm_bindgen(constructor)]
    pub fn new(base_url: &str, token: &str) -> Result<Cattleya, JsValue> {
        wasm_logger::init(wasm_logger::Config::default());

        log::info!("Initializing Cattleya on {}", base_url);

        console_error_panic_hook::set_once();
        let client = VioletRequest::new(token, base_url.to_owned())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(Cattleya {
            client: Arc::new(client),
        })
    }

    #[wasm_bindgen(js_name = customError)]
    pub fn custom_error(
        &self,
        err_level: String,
        message: String,
        stack_trace: Option<String>,
    ) -> Promise {
        let log_data = VioletLogData {
            error_level: err_level,
            message,
            stack_trace,
        };

        let client = self.client.clone();

        future_to_promise(async move {
            client
                .send_log(log_data)
                .await
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            Ok(JsValue::NULL)
        })
    }

    #[wasm_bindgen(js_name = fromError)]
    pub fn from_error(&self, err: &JsValue) -> Promise {
        let message = Reflect::get(err, &JsValue::from_str("message"));
        let stack_trace = Reflect::get(err, &JsValue::from_str("stack"));

        if let Ok(message) = message {
            let log_data = VioletLogData {
                error_level: "Error".to_owned(),
                message: message.as_string().unwrap_or_else(|| "".to_owned()),
                stack_trace: stack_trace
                    .ok()
                    .map(|s| s.as_string().unwrap_or_else(|| "".to_owned())),
            };

            let client = self.client.clone();

            future_to_promise(async move {
                client
                    .send_log(log_data)
                    .await
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;

                Ok(JsValue::NULL)
            })
        } else {
            Promise::reject(&JsValue::from_str("Could not get error message"))
        }
    }

    pub fn log(&self, level: ErrorsLevel, message: String, stack_trace: Option<String>) -> Promise {
        Self::custom_error(self, level.into(), message, stack_trace)
    }
}
