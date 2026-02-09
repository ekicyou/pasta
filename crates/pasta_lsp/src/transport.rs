//! Transport layer for the Pasta Language Server.
//!
//! Provides platform abstraction for WASM and native environments.

/// WASM entry point (only compiled for wasm32 target)
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    /// WASM-based LSP Server entry point.
    #[wasm_bindgen]
    pub struct WasmLspServer {
        // Will be populated when WASM transport is fully implemented
    }

    #[wasm_bindgen]
    impl WasmLspServer {
        /// Create a new WASM LSP server instance.
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Receive an LSP message from the JS side.
        #[wasm_bindgen]
        pub fn send(&self, _message: &str) {
            // TODO: Route message to tower-lsp server
        }

        /// Register a callback for receiving LSP responses.
        #[wasm_bindgen]
        pub fn on_message(&self, _callback: js_sys::Function) {
            // TODO: Store callback for sending responses
        }
    }
}

/// Native entry point stub (for future stdio-based transport)
#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    // Reserved for future native (stdio/TCP) transport implementation
}
