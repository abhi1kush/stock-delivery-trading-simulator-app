//! Browser entry point. The trading model lives in `simulator-core`.

use simulator_core::execute;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Run a delivery backtest.
///
/// `prices_csv` is OHLCV text (`date,open,high,low,close,volume`).
/// `policy_json` is a JSON object, or YAML when the text does not start with `{`.
/// Returns the serialized [`simulator_core::RunOutput`](simulator_core::RunOutput).
#[wasm_bindgen]
pub fn run_backtest(prices_csv: &str, policy_json: &str) -> Result<String, JsValue> {
    match execute(prices_csv, policy_json) {
        Ok(output) => serde_json::to_string(&output).map_err(|e| JsValue::from_str(&e.to_string())),
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}
