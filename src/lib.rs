use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use std::str;
use log::trace;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(BodyReplaceRootContext{
            header_content: "".to_string(),
        })
    });
}

struct BodyReplaceFilter{
    header_content: String
}

impl Context for BodyReplaceFilter {}

impl HttpContext for BodyReplaceFilter {

    fn on_http_request_headers(&mut self, _num_headers: usize) -> Action {

        for (name, value) in &self.get_http_request_headers() {
            trace!("{}: {}", name, value);
            if name.eq("custom-header") {
                self.add_http_request_header("Custom-WASM-Header", value);
                self.add_http_request_header("Custom-WASM-Config-Header", self.header_content.as_str());
                self.set_http_request_header("custom-header", None);
            }
        }

        Action::Continue
    }
}

struct BodyReplaceRootContext {
    header_content: String
}

impl Context for BodyReplaceRootContext {}

impl RootContext for BodyReplaceRootContext {
    
    fn on_vm_start(&mut self, _vm_configuration_size: usize) -> bool {
        true
    }

    fn on_configure(&mut self, _plugin_configuration_size: usize) -> bool {
        if let Some(config_bytes) = self.get_configuration() {
            self.header_content = str::from_utf8(config_bytes.as_ref()).unwrap().to_owned()
        }
        true
    }

    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(BodyReplaceFilter{
            header_content: self.header_content.clone(),
        }))
    
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

}
