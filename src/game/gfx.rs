use core::{ffi::CStr, ptr::null};

use crate::gl::*;

enum BufferKind {
    Vertex,
    Index
}

#[repr(u32)]
#[derive(Clone, Copy)]
enum BufferUsage {
    Static = GL_STATIC_DRAW,
    Dynamic = GL_DYNAMIC_DRAW,
    Stream = GL_STREAM_DRAW,
}

#[derive(Clone, Copy)]
struct BufferId(u32);

struct Buffer {
    id: BufferId,
    kind: BufferKind,
    usage: BufferUsage,
    length: usize,
}

struct Vertex {
    position: [f32; 2],
    color: [u8; 4],

}

fn bind_buffer(id: BufferId) {
    unsafe {
        glBindBuffer(GL_ARRAY_BUFFER, id.0);
    }
}

/// Put data into the buffer (if it's bound)
fn put_buffer_data(buffer: &Buffer, data: &[u8]) {
    assert!(data.len() <= buffer.length);

    unsafe {
        glBufferData(
            buffer.id.0, 
            data.len() as _, 
            data.as_ptr() as _, 
            buffer.usage as _
        );
    }
}

fn make_buffer(kind: BufferKind, usage: BufferUsage, length: usize) -> Buffer {
    let buffer_id = {
        let mut buffer_id: u32 = 0;

        unsafe {
            glGenBuffers(1, &mut buffer_id as _);
        }

        BufferId(buffer_id)
    };

    bind_buffer(buffer_id);

    Buffer { 
        id: buffer_id, 
        kind, 
        usage, 
        length 
    }
}


#[derive(Clone, Copy)]
struct ShaderId(u32);

#[repr(u32)]
#[derive(Clone, Copy)]
enum ShaderKind {
    Vertex = GL_VERTEX_SHADER,
    Fragment = GL_FRAGMENT_SHADER
}

struct Shader {
    id: ShaderId,
    kind: ShaderKind
}

fn make_shader(src: &CStr, kind: ShaderKind) -> Shader {
    fn make_shader_id(kind: ShaderKind) -> ShaderId {
        let id = unsafe { glCreateShader(kind as _) };
        ShaderId(id)
    }

    // Generate a new shader object
    let id = make_shader_id(kind);

    // To simplify we're going to only use C-strings for shaders
    let src_ptr = &(src.as_ptr()) as *const *const i8;

    unsafe {
        // Add our shader source
        glShaderSource(id.0, 1, src_ptr, null());
        
        // Compile
        glCompileShader(id.0);
    }

    // Now, we would like to check for its compilation status
    let mut status: i32 = 0;
    unsafe {
        glGetShaderiv(id.0, GL_COMPILE_STATUS, &mut status as _);
    }

    if status == 0 {
        const ERR_SIZE: usize = 256;

        let mut message = [0u8; ERR_SIZE];
        let mut message_len: i32 = 0;
        unsafe {
            glGetShaderInfoLog(
                id.0, 
                ERR_SIZE as _, 
                &mut message_len as _, 
                message.as_mut_ptr() as _
            );
        }

        let chr_slice = &message[0..(message_len) as usize];
        let s = str::from_utf8(chr_slice).unwrap();
        
        // For now we're going to panic
        panic!("{s}");
    }

    Shader {
        id,
        kind
    }
}