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

unsafe extern "C" {
    static __heap_base: usize;
}

struct App;
impl AppHandler for App {
    fn draw(&mut self) {
        
    }
}

fn main() -> App {
    println("This works...??? How...?");
    println("No way! This is impossible! There's no actual way!");

    unsafe {
        glClearColor(0.2, 0.8, 1.0, 1.0);
        glClear(GL_COLOR_BUFFER_BIT);
    }

    App
}

make_app!(App);