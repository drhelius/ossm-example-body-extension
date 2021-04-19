use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use std::str;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(BodyReplaceRootContext{
            secret_word: "".to_string(),
        })
    });
}

struct BodyReplaceFilter{
    secret_word: String
}

impl Context for BodyReplaceFilter {}

impl HttpContext for BodyReplaceFilter {

    fn on_http_response_headers(&mut self, _: usize) -> Action {
        // If there is a Content-Length header and we change the length of
        // the body later, then clients will break. So remove it.
        // We must do this here, because once we exit this function we
        // can no longer modify the response headers.
        self.set_http_response_header("content-length", None);
        Action::Continue
    }

    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            // Wait -- we'll be called again when the complete body is buffered
            // at the host side.
            return Action::Pause;
        }

        // Replace the message body if it contains the text in the config field.
        // Since we returned "Pause" previuously, this will return the whole body.
        if let Some(body_bytes) = self.get_http_response_body(0, body_size) {
            let body_str = String::from_utf8(body_bytes).unwrap();
            if body_str.contains(self.secret_word.as_str()) {
                let new_body = format!("Secret word found!! Original message body ({} bytes) omitted.", body_size);
                self.set_http_response_body(0, body_size, &new_body.into_bytes());
            }
        }
        Action::Continue
    }
}

struct BodyReplaceRootContext {
    secret_word: String
}

impl Context for BodyReplaceRootContext {}

impl RootContext for BodyReplaceRootContext {
    
    fn on_vm_start(&mut self, _vm_configuration_size: usize) -> bool {
        true
    }

    fn on_configure(&mut self, _plugin_configuration_size: usize) -> bool {
        if let Some(config_bytes) = self.get_configuration() {
            self.secret_word = str::from_utf8(config_bytes.as_ref()).unwrap().to_owned()
        }
        true
    }

    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(BodyReplaceFilter{
            secret_word: self.secret_word.clone(),
        }))
    
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}
