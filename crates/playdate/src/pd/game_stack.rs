//! The game's own stack (ADR-0070 §4, [`crate::stack`]): [`stack::SIZE`]
//! bytes from the heap, entered with a few lines of assembly around each
//! call into the game ([`run`]).
//!
//! Only the device switches stacks. The Simulator runs the game natively
//! on the PC's stack, which is big, so there [`run`] is a plain call and
//! nothing is measured.

use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use crate::stack;

/// The stack's lowest byte; null until [`init`] (and always off the
/// device).
static BASE: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());

/// Whether the game is running on the stack now, so a [`run`] inside a
/// [`run`] does not start again from the top.
static INSIDE: AtomicBool = AtomicBool::new(false);

/// Takes the stack from the heap and paints it. Called once, at start.
#[cfg(target_os = "none")]
pub fn init() {
    let base: *mut u8 = unsafe { super::pd_realloc(core::ptr::null_mut(), stack::SIZE) }.cast();
    assert!(!base.is_null(), "no memory for the game's stack");
    unsafe { core::ptr::write_bytes(base, stack::PAINT, stack::SIZE) };
    BASE.store(base, Ordering::Relaxed);
}

/// Nothing to take off the device.
#[cfg(not(target_os = "none"))]
pub fn init() {}

/// The bytes of the stack the game has ever used, or `None` where it is
/// not measured (the Simulator).
pub fn high_water() -> Option<usize> {
    let base = BASE.load(Ordering::Relaxed);
    if base.is_null() {
        return None;
    }
    // SAFETY: `init` took these bytes for the stack and never frees them.
    let bytes = unsafe { core::slice::from_raw_parts(base, stack::SIZE) };
    Some(stack::high_water(bytes, stack::PAINT))
}

/// Runs `f` on the game's stack and returns what it returns. Every call
/// from the update callback into game code goes through here.
pub fn run<R>(f: impl FnOnce() -> R) -> R {
    let base = BASE.load(Ordering::Relaxed);
    if base.is_null() || INSIDE.load(Ordering::Relaxed) {
        return f();
    }
    INSIDE.store(true, Ordering::Relaxed);
    let mut f = Some(f);
    let mut out = None;
    let mut call = || out = f.take().map(|f| f());
    let mut call: &mut dyn FnMut() = &mut call;
    // SAFETY: the stack is `stack::SIZE` bytes from `init`, unused while
    // `INSIDE` is false.
    unsafe {
        switch(stack::top(base.addr(), stack::SIZE), (&raw mut call).cast());
    }
    INSIDE.store(false, Ordering::Relaxed);
    let used = high_water().unwrap_or(0);
    assert!(
        !stack::overflowed(used, stack::SIZE),
        "the game's stack overflowed ({used} of {} bytes)",
        stack::SIZE
    );
    match out {
        Some(out) => out,
        None => panic!("the game's stack returned without running the call"),
    }
}

/// Calls the `&mut dyn FnMut()` behind `call`: the first function on the
/// game's stack.
extern "C" fn trampoline(call: *mut core::ffi::c_void) {
    // SAFETY: `run` passes a pointer to its `&mut dyn FnMut()`, alive for
    // the whole switch.
    let call = unsafe { &mut *call.cast::<&mut dyn FnMut()>() };
    call();
}

/// Moves the stack pointer to `top`, calls `trampoline(call)`, and moves
/// it back: Thumb, on the device.
#[cfg(target_os = "none")]
unsafe fn switch(top: usize, call: *mut core::ffi::c_void) {
    // The registers a C call may change, written out: `clobber_abi("C")`
    // would also name d16–d31, which the single-precision build (`-fp64`)
    // does not have.
    let first: extern "C" fn(*mut core::ffi::c_void) = trampoline;
    unsafe {
        core::arch::asm!(
            // r4 is callee-saved: `trampoline` gives it back unchanged.
            "mov r4, sp",
            "mov sp, r1",
            "blx r2",
            "mov sp, r4",
            inout("r0") call => _,
            inout("r1") top => _,
            inout("r2") first => _,
            out("r3") _,
            out("r4") _,
            out("r12") _,
            out("lr") _,
            out("s0") _, out("s1") _, out("s2") _, out("s3") _,
            out("s4") _, out("s5") _, out("s6") _, out("s7") _,
            out("s8") _, out("s9") _, out("s10") _, out("s11") _,
            out("s12") _, out("s13") _, out("s14") _, out("s15") _,
        );
    }
}

/// Off the device `BASE` stays null, so `run` never gets here; a plain
/// call keeps the build whole.
#[cfg(not(target_os = "none"))]
unsafe fn switch(_top: usize, call: *mut core::ffi::c_void) {
    trampoline(call);
}
