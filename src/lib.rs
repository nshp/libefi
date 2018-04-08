//! Crate for writing UEFI software using Rust


#![feature(alloc, allocator_api, compiler_builtins_lib, const_fn, global_allocator, lang_items)]
#![no_std]


extern crate alloc;
#[macro_use]
extern crate bitflags;
extern crate compiler_builtins;
extern crate rlibc;
extern crate spin;


pub mod boot_services;
#[macro_use]
pub mod console;
pub mod protocols;
pub mod runtime;
pub mod runtime_services;
mod system_table;
pub mod types;

pub use system_table::SystemTable;

/// Static instance of Allocator used as the global allocator
///
/// TODO:
///
/// * Put this in the `runtime` module once [#44113](https://github.com/rust-lang/rust/issues/44113)
///   is resolved
#[global_allocator]
static ALLOCATOR: runtime::PageAllocator = runtime::PageAllocator {};
