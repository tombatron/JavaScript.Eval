use std::os::raw::c_char;
use std::{
    ffi::{CStr, CString},
    ptr,
};

use v8facade::{FunctionParameter, JavaScriptResult, Output, V8Facade};

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

#[repr(C)]
#[derive(Debug)]
pub struct PrimitiveResult {
    pub number_value: f64,
    pub number_value_set: bool,

    pub bigint_value: i64,
    pub bigint_value_set: bool,

    pub bool_value: bool,
    pub bool_value_set: bool,

    pub string_value: *mut c_char,
    pub array_value: *mut c_char,
    pub object_value: *mut c_char,
}

impl PrimitiveResult {
    pub fn create_for_number(number: f64) -> PrimitiveResult {
        PrimitiveResult {
            number_value: number,
            number_value_set: true,

            bigint_value: 0,
            bigint_value_set: false,

            bool_value: false,
            bool_value_set: false,

            string_value: ptr::null_mut(),
            array_value: ptr::null_mut(),
            object_value: ptr::null_mut(),
        }
    }

    pub fn create_for_bigint(bigint: i64) -> PrimitiveResult {
        PrimitiveResult {
            number_value: 0.0,
            number_value_set: false,

            bigint_value: bigint,
            bigint_value_set: true,

            bool_value: false,
            bool_value_set: false,

            string_value: ptr::null_mut(),
            array_value: ptr::null_mut(),
            object_value: ptr::null_mut(),
        }
    }

    pub fn create_for_bool(boolean: bool) -> PrimitiveResult {
        PrimitiveResult {
            number_value: 0.0,
            number_value_set: false,

            bigint_value: 0,
            bigint_value_set: false,

            bool_value: boolean,
            bool_value_set: true,

            string_value: ptr::null_mut(),
            array_value: ptr::null_mut(),
            object_value: ptr::null_mut(),
        }
    }

    pub fn create_for_string(string: String) -> PrimitiveResult {
        PrimitiveResult {
            number_value: 0.0,
            number_value_set: false,

            bigint_value: 0,
            bigint_value_set: false,

            bool_value: false,
            bool_value_set: false,

            string_value: CString::new(string).unwrap().into_raw(),
            array_value: ptr::null_mut(),
            object_value: ptr::null_mut(),
        }
    }

    pub fn create_for_array(array: String) -> PrimitiveResult {
        PrimitiveResult {
            number_value: 0.0,
            number_value_set: false,

            bigint_value: 0,
            bigint_value_set: false,

            bool_value: false,
            bool_value_set: false,

            string_value: ptr::null_mut(),
            array_value: CString::new(array).unwrap().into_raw(),
            object_value: ptr::null_mut(),
        }
    }

    pub fn create_for_object(object: String) -> PrimitiveResult {
        PrimitiveResult {
            number_value: 0.0,
            number_value_set: false,

            bigint_value: 0,
            bigint_value_set: false,

            bool_value: false,
            bool_value_set: false,

            string_value: ptr::null_mut(),
            array_value: ptr::null_mut(),
            object_value: CString::new(object).unwrap().into_raw(),
        }
    }

    pub fn create_for_error(string: String) -> PrimitiveResult {
        // TODO: Need some first class error stuff here.

        PrimitiveResult {
            number_value: 0.0,
            number_value_set: false,

            bigint_value: 0,
            bigint_value_set: false,

            bool_value: false,
            bool_value_set: false,

            string_value: CString::new(string).unwrap().into_raw(),
            array_value: ptr::null_mut(),
            object_value: ptr::null_mut(),
        }
    }
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
        Output::Result(r) => match r {
            JavaScriptResult::StringValue(s) => CString::new(s).unwrap().into_raw(),
            _ => CString::new("complete this").unwrap().into_raw(),
        },
        Output::Error(e) => CString::new(e).unwrap().into_raw(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn call(
    v8_facade_ptr: *mut V8Facade,
    func_name: *const c_char,
    parameters: *const Primitive,
    parameter_count: usize,
) -> *mut PrimitiveResult {
    let func_name = CStr::from_ptr(func_name).to_string_lossy().into_owned();

    let parameters: &[Primitive] = std::slice::from_raw_parts(parameters, parameter_count);
    let parameters = parameters
        .iter()
        .map(|p| FunctionParameter::from(p))
        .collect();

    let instance = {
        assert!(!v8_facade_ptr.is_null());
        &mut *v8_facade_ptr
    };

    let result = instance.call(func_name, parameters).unwrap();

    match result {
        Output::Result(r) => match r {
            JavaScriptResult::StringValue(s) => {
                Box::into_raw(Box::new(PrimitiveResult::create_for_string(s)))
            }
            JavaScriptResult::NumberValue(n) => {
                Box::into_raw(Box::new(PrimitiveResult::create_for_number(n)))
            }
            JavaScriptResult::BigIntValue(i) => {
                Box::into_raw(Box::new(PrimitiveResult::create_for_bigint(i)))
            }
            JavaScriptResult::BoolValue(b) => {
                Box::into_raw(Box::new(PrimitiveResult::create_for_bool(b)))
            }
            JavaScriptResult::ArrayValue(av) => {
                Box::into_raw(Box::new(PrimitiveResult::create_for_array(av)))
            }
            JavaScriptResult::ObjectValue(ov) => {
                Box::into_raw(Box::new(PrimitiveResult::create_for_object(ov)))
            }
        },
        Output::Error(e) => Box::into_raw(Box::new(PrimitiveResult::create_for_error(e))),
    }
}

#[no_mangle]
pub unsafe extern "C" fn free_string(string_ptr: *mut c_char) {
    if string_ptr.is_null() {
        return;
    }

    CString::from_raw(string_ptr); // This should be OK because the string that we are freeing was created "by Rust".
}

#[no_mangle]
pub unsafe extern "C" fn free_primitive_result(primitive_result_ptr: *mut PrimitiveResult) {
    let primitive_result = Box::from_raw(primitive_result_ptr);

    if !primitive_result.string_value.is_null() {
        CString::from_raw(primitive_result.string_value);
    }

    if !primitive_result.array_value.is_null() {
        CString::from_raw(primitive_result.array_value);
    }

    if !primitive_result.object_value.is_null() {
        CString::from_raw(primitive_result.object_value);
    }
}
