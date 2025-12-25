//! To minimize the binary size, the strategy I'm thinking of is using a bump-allocator to 
//! pre-allocate a lot of static arrays on the heap, after which reuse them all.
//! 
//! Because putting statics directly into the binary increases its own size - this size allows us
//! to "outsource" this weight onto the heap.
//! 
//! This however does mean that every allocation is permanent, so we should use a lot of pooling
//! for rapid allocations.

use core::{alloc::Layout, cell::UnsafeCell};

mod constvec;
pub use constvec::ConstVec;
    
use crate::request_pages;

const PAGE_SIZE: usize = 2usize.pow(16);

unsafe extern "C" {
    static __heap_base: u8;
}

/// Get the heap base (the address from which the heap starts) aligned to [PAGE_SIZE]
pub fn heap_base() -> *const u8 {

    unsafe { &__heap_base as *const u8 }
} 

/// Ge the initial amount of pages allocated by rust
pub fn initial_pages() -> usize {
    (heap_base() as usize).div_ceil(PAGE_SIZE)
}

/// Get the amount of space wasted on the first heap page
pub fn heap_waste() -> usize {
    let available = (initial_pages() * PAGE_SIZE) - heap_base() as usize;
    PAGE_SIZE - available
}

struct BumpAllocator {
    cursor: *const u8,
    free_space: usize
}

impl BumpAllocator {
    fn new() -> Self {
        Self {
            cursor: heap_base(),
            free_space: PAGE_SIZE-heap_waste()
        }
    }

    unsafe fn alloc(&mut self, layout: core::alloc::Layout) -> *mut u8 {
        // Compute the padding to align user data
        let padding = self.cursor.align_offset(layout.align());

        // Compute the total amount of data that needs to be put
        let total = padding+layout.size();

        // If we don't have enough space
        if total > self.free_space {

            // Compute how many pages we need
            let needs_pages = (total-self.free_space).div_ceil(PAGE_SIZE);
            
            // Grow our memory
            request_pages(needs_pages);

            // Increase the amount of free space
            self.free_space += needs_pages * PAGE_SIZE;
        }

        // Get our return pointer
        let ptr = unsafe { self.cursor.add(padding) } as *mut u8;

        // Move our cursor to the right
        self.cursor = unsafe { self.cursor.add(total) };
        
        // Decrease the amount of free space we got
        self.free_space -= total;

        ptr
    }
}

struct Allocator {
    alloc: UnsafeCell<Option<BumpAllocator>>
}

// We're in wasm, so we don't care
unsafe impl Send for Allocator {}
unsafe impl Sync for Allocator {} 

impl Allocator {
    const fn new() -> Self {
        Self {
            alloc: UnsafeCell::new(None)
        }
    }

    unsafe fn get_alloc(&self) -> &mut BumpAllocator {
        let alloc = unsafe { &mut *self.alloc.get() };

        if alloc.is_none() {
            alloc.replace(BumpAllocator::new());
        }

        alloc.as_mut().unwrap()
    }

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { self.get_alloc().alloc(layout) }
    }
}

static ALLOCATOR: Allocator = Allocator::new();

/// Allocate the provided amount of memory. Note that this memory is permanent and can't
/// be deallocated.
pub unsafe fn alloc(layout: Layout) -> *mut u8 {
    unsafe { ALLOCATOR.alloc(layout) }
}