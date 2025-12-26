#![no_std]

// Only for tests
#[cfg(not(target_family = "wasm"))]
extern crate std;

mod cell;
pub use cell::*;

mod alloc;
mod app;

pub use app::*;

mod js;
pub use js::*;

// use crate::alloc2::alloc;

// mod gl;
// pub use gl::*;

struct App;
impl AppHandler for App {
    fn draw(&mut self) {
        
    }
}

fn main() -> App {    

    App
}

make_app!(App);