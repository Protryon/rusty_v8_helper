extern crate rusty_v8_protryon as rusty_v8;
extern crate self as rusty_v8_helper;

use proc_macro_hack::proc_macro_hack;
#[proc_macro_hack]
pub use rusty_v8_helper_derive::load_v8_ffi;
pub use rusty_v8_helper_derive::v8_ffi;

mod object_wrap;
pub use object_wrap::ObjectWrap;

mod ffi_map;
pub use ffi_map::FFICompat;
pub use ffi_map::FFICompat2;
pub use ffi_map::FFIObject;
pub mod util;
