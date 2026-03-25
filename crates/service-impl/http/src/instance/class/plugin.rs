use super::registry::ClassRegistry;

pub type PluginRegisterFn = unsafe extern "Rust" fn(&mut ClassRegistry, api_verison: &'static str);
pub const API_VERSION: &str = "0.1";
impl ClassRegistry {
    pub fn register_rust_plugin(&mut self, register_fn: PluginRegisterFn) {
        unsafe { register_fn(self, API_VERSION) }
    }
}

#[cfg(feature = "service-impl")]
impl ClassRegistry {
    pub fn load_dynamic_lib(&mut self, lib: &libloading::Library) -> Result<(), libloading::Error> {
        unsafe {
            let register_fn = lib.get::<PluginRegisterFn>(b"register\0")?;
            self.register_rust_plugin(*register_fn);
        };
        Ok(())
    }
}
