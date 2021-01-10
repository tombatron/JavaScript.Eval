use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use v8facade::{FunctionParameter, Output, V8Facade};

mod v8facade;

#[repr(C)]
#[derive(Debug)]
pub struct Primitive {
    pub number_value: f64,
    pub number_value_set: bool,

    pub bigint_value: i64,
    pub bigint_value_set: bool,

    pub bool_value: bool,
    pub bool_value_set: bool,

    pub string_value: *mut c_char,
    pub symbol_value: *mut c_char,
}

// http://jakegoulding.com/rust-ffi-omnibus/objects/
#[no_mangle]
pub extern "C" fn get_v8() -> *mut V8Facade {
    Box::into_raw(Box::new(V8Facade::new()))
}

#[no_mangle]
pub extern "C" fn free_v8(v8_facade_ptr: *mut V8Facade) {
    if v8_facade_ptr.is_null() {
        return;
    }

    unsafe {
        Box::from_raw(v8_facade_ptr);
    }
}

// TODO: Going to want to put in some kind of struct here so that we can better describe the result.
#[no_mangle]
pub unsafe extern "C" fn exec(
    v8_facade_ptr: *mut V8Facade,
    script: *const c_char,
) -> *const c_char {
    let script = CStr::from_ptr(script).to_string_lossy().into_owned();

    let instance = {
        assert!(!v8_facade_ptr.is_null());
        &mut *v8_facade_ptr
    };

    let result = instance.run(script).unwrap();

    match result {
        Output::Result(r) => CString::new(r).unwrap().into_raw(),
        Output::Error(e) => CString::new(e).unwrap().into_raw(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn call(
    v8_facade_ptr: *mut V8Facade,
    func_name: *const c_char,
    parameters: *const Primitive,
    parameter_count: usize,
) -> *const c_char {
    let func_name = CStr::from_ptr(func_name).to_string_lossy().into_owned();

    let parameters: &[Primitive] = std::slice::from_raw_parts(parameters, parameter_count);
    let parameters = parameters.iter().map(|p| FunctionParameter::from(p)).collect();

    let instance = {
        assert!(!v8_facade_ptr.is_null());
        &mut *v8_facade_ptr
    };

    let result = instance.call(func_name, parameters).unwrap();

    match result {
        Output::Result(r) => CString::new(r).unwrap().into_raw(),
        Output::Error(e) => CString::new(e).unwrap().into_raw(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn free_string(string_ptr: *mut c_char) {
    if string_ptr.is_null() {
        return;
    }

    CString::from_raw(string_ptr); // This should be OK because the string that we are freeing was created "by Rust". 
}
