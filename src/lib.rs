#![no_std]

// Only for tests
#[cfg(test)]
extern crate std;

mod alloc;
mod app;
pub use app::*;

// mod heap;
mod js;
pub use js::*;

mod gl;
pub use gl::*;

/// Get the heap base (the address from which the heap starts) 
pub fn heap_base() -> *const u8 {

    unsafe extern "C" {
        static __heap_base: u8;
    }

    unsafe { &__heap_base as *const u8 }
} 

/// Ge the initial amount of pages allocated by rust
pub fn initial_pages() -> usize {
    (heap_base() as usize).div_ceil(2usize.pow(16))
}

struct App;
impl AppHandler for App {
    fn draw(&mut self) {
        
    }
}

fn main() -> App {
    println_number(heap_base() as usize);
    println_number((initial_pages() * 2usize.pow(16)) - heap_base() as usize);

    println_number(initial_pages() as usize);
    println_number(allocated_pages());

    unsafe {
        glClearColor(0.2, 0.8, 1.0, 1.0);
        glClear(GL_COLOR_BUFFER_BIT);
    }

    App
}

make_app!(App);