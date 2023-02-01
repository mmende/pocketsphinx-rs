use std::error::Error;

pub struct PsConfig {
    inner: *mut pocketsphinx_sys::ps_config_t,
}

impl PsConfig {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let config = unsafe { pocketsphinx_sys::ps_config_init(std::ptr::null_mut()) };

        if config.is_null() {
            Err("Failed to initialize config".into())
        } else {
            Ok(PsConfig { inner: config })
        }
    }

    pub fn get_bool(&self, name: &str) -> Result<bool, Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let value = unsafe { pocketsphinx_sys::ps_config_bool(self.inner, c_name.as_ptr()) };

        if value == -1 {
            Err("Failed to get config value".into())
        } else {
            Ok(value == 1)
        }
    }

    pub fn set_bool(&mut self, name: &str, value: bool) -> Result<(), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let value = if value { 1 } else { 0 };

        let _result =
            unsafe { pocketsphinx_sys::ps_config_set_bool(self.inner, c_name.as_ptr(), value) };

        Ok(())
    }

    pub fn get_int(&self, name: &str) -> Result<i64, Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let value = unsafe { pocketsphinx_sys::ps_config_int(self.inner, c_name.as_ptr()) };

        if value == -1 {
            Err("Failed to get config value".into())
        } else {
            Ok(value)
        }
    }

    pub fn set_int(&mut self, name: &str, value: i64) -> Result<(), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;

        let _result =
            unsafe { pocketsphinx_sys::ps_config_set_int(self.inner, c_name.as_ptr(), value) };

        Ok(())
    }

    pub fn get_float(&self, name: &str) -> Result<f64, Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let value = unsafe { pocketsphinx_sys::ps_config_float(self.inner, c_name.as_ptr()) };

        Ok(value)
    }

    pub fn set_float(&mut self, name: &str, value: f64) -> Result<(), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;

        let _result =
            unsafe { pocketsphinx_sys::ps_config_set_float(self.inner, c_name.as_ptr(), value) };

        Ok(())
    }

    pub fn get_str(&self, name: &str) -> Result<String, Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let value = unsafe { pocketsphinx_sys::ps_config_str(self.inner, c_name.as_ptr()) };

        if value.is_null() {
            Err("Failed to get config value".into())
        } else {
            Ok(unsafe { std::ffi::CStr::from_ptr(value) }
                .to_str()
                .unwrap()
                .to_string())
        }
    }

    pub fn set_str(&mut self, name: &str, value: &str) -> Result<(), Box<dyn Error>> {
        let c_name = std::ffi::CString::new(name)?;
        let c_value = std::ffi::CString::new(value)?;

        let _result = unsafe {
            pocketsphinx_sys::ps_config_set_str(self.inner, c_name.as_ptr(), c_value.as_ptr())
        };

        Ok(())
    }

    pub fn get_inner(&self) -> *mut pocketsphinx_sys::ps_config_t {
        self.inner
    }
}

impl Drop for PsConfig {
    fn drop(&mut self) {
        unsafe {
            pocketsphinx_sys::ps_config_free(self.inner);
        }
    }
}
