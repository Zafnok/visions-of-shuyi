//! The global allocator: the SDK's `realloc` (ADR-0070 §1), with padding
//! for alignments above its own ([`crate::heap`]).

use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

use super::pd_realloc;
use crate::heap;

struct SdkAlloc;

#[global_allocator]
static ALLOC: SdkAlloc = SdkAlloc;

// SAFETY: blocks come from the SDK's allocator, which returns
// `heap::MIN_ALIGN`-aligned blocks or null; bigger alignments are padded
// and keep the block's own address just before the pointer handed out.
unsafe impl GlobalAlloc for SdkAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !heap::needs_padding(layout.align()) {
            return unsafe { pd_realloc(ptr::null_mut(), layout.size()) }.cast();
        }
        let Some(size) = heap::padded_size(layout.size(), layout.align()) else {
            return ptr::null_mut();
        };
        let raw: *mut u8 = unsafe { pd_realloc(ptr::null_mut(), size) }.cast();
        if raw.is_null() {
            return raw;
        }
        let aligned = unsafe { raw.add(heap::offset(raw.addr(), layout.align())) };
        // `offset` leaves at least a pointer's room before `aligned`, which
        // is aligned to more than a pointer.
        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            aligned.cast::<*mut u8>().sub(1).write(raw);
        };
        aligned
    }

    unsafe fn dealloc(&self, block: *mut u8, layout: Layout) {
        let raw = if heap::needs_padding(layout.align()) {
            #[allow(clippy::cast_ptr_alignment)]
            unsafe {
                block.cast::<*mut u8>().sub(1).read()
            }
        } else {
            block
        };
        unsafe { pd_realloc(raw.cast(), 0) };
    }

    unsafe fn realloc(&self, block: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if !heap::needs_padding(layout.align()) {
            return unsafe { pd_realloc(block.cast(), new_size) }.cast();
        }
        // SAFETY: the caller guarantees `new_size`, rounded up to the
        // alignment, does not overflow.
        let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };
        let new = unsafe { self.alloc(new_layout) };
        if !new.is_null() {
            unsafe { ptr::copy_nonoverlapping(block, new, layout.size().min(new_size)) };
            unsafe { self.dealloc(block, layout) };
        }
        new
    }
}
