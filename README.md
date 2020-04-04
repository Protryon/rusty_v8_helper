# Rusty V8 Helper

The purpose of this project is to add more complex external functionality to my (Protryon) fork of rusty_v8.

## Exports

* There are some utlity functions like `make_str` and `run_script` in `::util`.
* `::object_wrap` provides `ObjectWrap` which allows the wrapped of an owned rust object inside a V8 object with weak deallocation within V8.
* Importing `::ffi_map::*` Provides the `v8_ffi` macro and `load_v8_ffi` macro. See tests in `::ffi_map` for details.
    * In general, the purpose of this module is to allow the near-transparent mapping of idiomatic rust functions to JS code via macro. This drastically cuts down on development overhead for FFI implementations.