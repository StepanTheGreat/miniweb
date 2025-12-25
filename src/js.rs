mod js {
    use core::ptr::null;

    unsafe extern "C" {        
        fn js_request_pages(pages: usize);

        fn js_allocated_pages() -> usize;

        fn js_println(start: *const u8, len: usize);

        fn js_alert(start: *const u8, len: usize);

        fn js_panic(
            err: *const u8, err_len: usize,
            file: *const u8, file_len: usize,
            line: u32
        );

        fn js_println_number(number: usize);
    }

    /// Request an amount of pages to be allocated from JS 
    pub fn request_pages(pages: usize) {
        unsafe { js_request_pages(pages) }
    }

    /// Get the current amount of allocated pages from JS
    pub fn allocated_pages() -> usize {
        unsafe { js_allocated_pages() }
    }

    pub fn println_number(number: usize) {
        unsafe { js_println_number(number) };
    }

    /// Print to the console
    pub fn println(message: &str) {
        unsafe { println_raw(message.as_ptr(), message.len()) }
    }

    pub unsafe fn println_raw(ptr: *const u8, len: usize) {
        unsafe { js_println(ptr, len);}
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