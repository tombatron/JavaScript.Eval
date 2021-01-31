use std::{ffi::CString, os::raw::c_char, ptr};

use crate::v8facade::JavaScriptError;

#[repr(C)]
#[derive(Debug)]
pub struct UnsafeJavaScriptError {
    pub exception: *mut c_char,
    pub stack_trace: *mut c_char,
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

    pub error: *mut UnsafeJavaScriptError,
}

impl PrimitiveResult {

    pub fn blank() -> PrimitiveResult {
        PrimitiveResult {
            number_value: 0.0,
            number_value_set: false,
            bigint_value: 0,
            bigint_value_set: false,
            bool_value: false,
            bool_value_set: false,
            string_value: ptr::null_mut(),
            array_value: ptr::null_mut(),
            object_value: ptr::null_mut(),
            error: ptr::null_mut(),
        }
    }

    pub fn create_for_number(number: f64) -> PrimitiveResult {
        let blank_result = PrimitiveResult::blank();

        PrimitiveResult {
            number_value: number,
            number_value_set: true,
            ..blank_result
        }
    }

    pub fn create_for_bigint(bigint: i64) -> PrimitiveResult {
        let blank_result = PrimitiveResult::blank();

        PrimitiveResult {
            bigint_value: bigint,
            bigint_value_set: true,
            ..blank_result
        }
    }

    pub fn create_for_bool(boolean: bool) -> PrimitiveResult {
        let blank_result = PrimitiveResult::blank();

        PrimitiveResult {
            bool_value: boolean,
            bool_value_set: true,
            ..blank_result
        }
    }

    pub fn create_for_string(string: String) -> PrimitiveResult {
        let blank_result = PrimitiveResult::blank();

        PrimitiveResult {
            string_value: CString::new(string).unwrap().into_raw(),
            ..blank_result
        }
    }

    pub fn create_for_array(array: String) -> PrimitiveResult {
        let blank_result = PrimitiveResult::blank();

        PrimitiveResult {
            array_value: CString::new(array).unwrap().into_raw(),
            ..blank_result
        }
    }

    pub fn create_for_object(object: String) -> PrimitiveResult {
        let blank_result = PrimitiveResult::blank();

        PrimitiveResult {
            object_value: CString::new(object).unwrap().into_raw(),
            ..blank_result
        }
    }

    pub fn create_for_error(javascript_error: JavaScriptError) -> PrimitiveResult {
        let exception = CString::new(javascript_error.exception).unwrap().into_raw();
        let stack_trace = CString::new(javascript_error.stack_trace)
            .unwrap()
            .into_raw();

        let unsafe_error = UnsafeJavaScriptError {
            exception,
            stack_trace,
        };

        let unsafe_error = Box::into_raw(Box::new(unsafe_error));

        let blank_result = PrimitiveResult::blank();

        PrimitiveResult {
            error: unsafe_error,
            ..blank_result
        }
    }
}