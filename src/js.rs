mod js {
    use core::ptr::null;

    unsafe extern "C" {
        fn js_println(start: *const u8, len: usize);

        fn js_alert(start: *const u8, len: usize);

        fn js_panic(
            err: *const u8, err_len: usize,
            file: *const u8, file_len: usize,
            line: u32
        );
    }

    /// Print to the console
    pub fn println(message: &str) {
        unsafe { js_println(message.as_ptr(), message.len()) }
    }

    /// Alert!!
    pub fn alert(message: &str) {
        unsafe { js_alert(message.as_ptr(), message.len()) }
    }

    pub fn panic(message: Option<&str>, file: &str, line: u32) {
        let (err_ptr, err_len) = match message {
            Some(s) => {
                (s.as_ptr(), s.len())
            },
            None => {
                (null(), 0)
            }
        };

        unsafe { js_panic(err_ptr, err_len, file.as_ptr(), file.len(), line) }
    }
}

pub use js::*;