use std::{ffi::CString, os::raw::c_char, ptr};

use crate::v8facade::{JavaScriptError, JavaScriptResult, Output};

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

    pub fn from_output(output: Output) -> PrimitiveResult {
        match output {
            Output::Result(r) => match r {
                JavaScriptResult::StringValue(s) => {
                    PrimitiveResult::create_for_string(s)
                }
                JavaScriptResult::NumberValue(n) => {
                    PrimitiveResult::create_for_number(n)
                }
                JavaScriptResult::BigIntValue(i) => {
                    PrimitiveResult::create_for_bigint(i)
                }
                JavaScriptResult::BoolValue(b) => {
                    PrimitiveResult::create_for_bool(b)
                }
                JavaScriptResult::ArrayValue(av) => {
                    PrimitiveResult::create_for_array(av)
                }
                JavaScriptResult::ObjectValue(ov) => {
                    PrimitiveResult::create_for_object(ov)
                }
            },
    
            Output::Error(e) => PrimitiveResult::create_for_error(e),
    
            // You can't get heap statistics out of V8 by invoking script so this result is impossible.
            Output::HeapStatistics(_) => unreachable!(),
        }        
    }

    pub fn from_javascriptresult(result: JavaScriptResult) -> PrimitiveResult {
        match result {
            JavaScriptResult::ArrayValue(v) => {
                PrimitiveResult::create_for_array(v)
            }
            JavaScriptResult::StringValue(v) => {
                PrimitiveResult::create_for_string(v)
            }
            JavaScriptResult::NumberValue(v) => {
                PrimitiveResult::create_for_number(v)
            }
            JavaScriptResult::BigIntValue(v) => {
                PrimitiveResult::create_for_bigint(v)
            }
            JavaScriptResult::BoolValue(v) => {
                PrimitiveResult::create_for_bool(v)
            }
            JavaScriptResult::ObjectValue(v) => {
                PrimitiveResult::create_for_object(v)
            }
        }
    }

    pub fn into_raw(self: Self) -> *mut PrimitiveResult {
        Box::into_raw(Box::new(self))
    }

    pub unsafe fn free_raw(raw_prim_result: *mut PrimitiveResult) {
        let primitive_result = Box::from_raw(raw_prim_result);

        if !primitive_result.string_value.is_null() {
            CString::from_raw(primitive_result.string_value);
        }
    
        if !primitive_result.array_value.is_null() {
            CString::from_raw(primitive_result.array_value);
        }
    
        if !primitive_result.object_value.is_null() {
            CString::from_raw(primitive_result.object_value);
        }
    
        if !primitive_result.error.is_null() {
            let error = primitive_result.error;
    
            CString::from_raw((*error).exception);
            CString::from_raw((*error).stack_trace);
    
            Box::from_raw(primitive_result.error);
        }       
    }
}