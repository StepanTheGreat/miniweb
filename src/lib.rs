#![no_std]

// Only for tests
#[cfg(not(target_family = "wasm"))]
extern crate std;

mod alloc;
mod app;
use core::{alloc::Layout, ptr::null_mut, str};

pub use app::*;

mod js;
pub use js::*;

use crate::alloc::ConstVec;

// use crate::alloc2::alloc;

// mod gl;
// pub use gl::*;

struct App;
impl AppHandler for App {
    fn draw(&mut self) {
        
    }
}

type ByteVec = ConstVec<u8, 2048>;
static mut DATA: *mut ByteVec = null_mut();

fn main() -> App {    
    // Allocate a pointer for our string

    unsafe {
        let ptr = alloc::alloc(Layout::from_size_align(
            size_of::<ByteVec>(), 
            align_of::<ByteVec>()
        ).unwrap()) as *mut ByteVec;

        ptr.write(ByteVec::new());

        DATA = ptr;
    }


    let _ = unsafe {&mut *DATA }.push(67);
    let _ = unsafe {&mut *DATA }.push(50);
    let _ = unsafe {&mut *DATA }.push(99);

    unsafe {
        let strr = str::from_utf8_unchecked((&*DATA).as_slice());

        println(strr);
    }

    App
}

make_app!(App);