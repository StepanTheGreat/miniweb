use core::{alloc::{Layout, GlobalAlloc}, fmt::Display, ptr, slice};

#[global_allocator]
static ALLOCATOR: wee::WeeAlloc = wee::WeeAlloc::INIT;

/// Allocate some memory
pub unsafe fn malloc(layout: Layout) -> *mut u8 {
    unsafe { ALLOCATOR.alloc(layout) }
}

/// Free some memory
pub unsafe fn free(ptr: *mut u8, layout: Layout) {
    unsafe { ALLOCATOR.dealloc(ptr, layout) }
}

pub struct String {
    ptr: *mut u8,
    len: usize
}

impl String {
    pub fn new(s: &str) -> Self {
        
        let len = s.len();
        let s_ptr = s.as_ptr();

        let ptr = unsafe { malloc(Layout::from_size_align(    
            size_of::<u8>() * len,
            size_of::<u8>()
        ).unwrap()) };

        unsafe { ptr::copy_nonoverlapping(s_ptr, ptr, len) };

        Self {
            ptr,
            len
        }
    }

    pub fn as_str(&self) -> &str {
        
        let chars = unsafe { slice::from_raw_parts(
            self.ptr,
            self.len
        ) };

        str::from_utf8(chars).unwrap()
    }
}

impl Drop for String {
    fn drop(&mut self) {
        unsafe {
            free(
                self.ptr,
                Layout::from_size_align(    
                    size_of::<u8>() * self.len,
                    size_of::<u8>()
                ).unwrap()
            );
        }
    }
}

impl Display for String {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}