//! A module containing functions and utilities useful for synchronizing state.

use crate::io::irq::{IrqEnableSetting, IME};

mod locks;
mod statics;

pub use locks::*;
pub use statics::*;

/// Marks that a pointer is read without actually reading from this.
///
/// This uses an [`asm!`] instruction that marks the parameter as being read,
/// requiring the compiler to treat this function as if anything could be
/// done to it.
#[inline(always)]
pub fn memory_read_hint<T>(val: *const T) {
  unsafe { asm!("/* {0} */", in(reg) val, options(readonly, nostack)) }
}

/// Marks that a pointer is read or written to without actually writing to it.
///
/// This uses an [`asm!`] instruction that marks the parameter as being read
/// and written, requiring the compiler to treat this function as if anything
/// could be done to it.
#[inline(always)]
pub fn memory_write_hint<T>(val: *mut T) {
  unsafe { asm!("/* {0} */", in(reg) val, options(nostack)) }
}

/// An internal function used as a temporary hack to get `compiler_fence`
/// working. While this call is not properly inlined, working is better than
/// not working at all.
///
/// This seems to be a problem caused by Rust issue #62256:
/// <https://github.com/rust-lang/rust/issues/62256>
///
/// Not public API, obviously.
#[doc(hidden)]
#[deprecated]
#[allow(dead_code)]
#[no_mangle]
#[inline(always)]
pub unsafe extern "C" fn __sync_synchronize() {}

/// Runs a function with IRQs disabled.
///
/// This should not be done without good reason, as IRQs are usually important
/// for game functionality.
pub fn with_irqs_disabled<T>(mut func: impl FnOnce() -> T) -> T {
  let current_ime = IME.read();
  IME.write(IrqEnableSetting::IRQ_NO);
  // prevents the contents of the function from being reordered before IME is disabled.
  memory_write_hint(&mut func);

  let mut result = func();

  // prevents the contents of the function from being reordered after IME is reenabled.
  memory_write_hint(&mut result);
  IME.write(current_ime);

  result
}
