use std::{os::raw::c_char, unreachable};
use std::{
    ffi::{CStr, CString},
    ptr,
};

use v8facade::{FunctionParameter, JavaScriptError, JavaScriptResult, Output, V8Facade};

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
    pub object_value: *mut c_char,
}

#[repr(C)]
#[derive(Debug)]
pub struct UnsafeJavaScriptError {
    pub exception: *mut c_char,
    pub stack_trace: *mut c_char,
}

#[repr(C)]
#[derive(Debug)]
pub struct V8HeapStatistics {
    pub total_heap_size: usize,
    pub total_heap_size_executable: usize,
    pub total_physical_size: usize,
    pub total_available_size: usize,
    pub used_heap_size: usize,
    pub heap_size_limit: usize,
    pub malloced_memory: usize,
    pub does_zap_garbage: usize,
    pub number_of_native_contexts: usize,
    pub number_of_detached_contexts: usize,
    pub peak_malloced_memory: usize,
    pub used_global_handles_size: usize,
    pub total_global_handles_size: usize,
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
) -> *mut PrimitiveResult {
    let script = CStr::from_ptr(script).to_string_lossy().into_owned();

    println!("This is what script Rust is seeing: {}", script);

    let instance = {
        assert!(!v8_facade_ptr.is_null());
        &mut *v8_facade_ptr
    };

    let result = instance.run(script).unwrap();

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

        // You can't get heap statistics out of V8 by invoking script so this result is impossible.
        Output::HeapStatistics(_) => unreachable!()
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

    let result = match instance.call(func_name, parameters) {
        Ok(o) => o,
        Err(e) => Output::Error(JavaScriptError{exception : e, stack_trace:String::from("")}),
    };

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

        // You can't get heap statistics by invoking a JavaScript function so this would be impossible. 
        Output::HeapStatistics(_) => unreachable!()
    }
}

#[no_mangle]
pub unsafe extern "C" fn get_heap_statistics(v8_facade_ptr: *mut V8Facade) -> *mut V8HeapStatistics {
    let instance = {
        assert!(!v8_facade_ptr.is_null());
        &mut *v8_facade_ptr
    };

    let heap_stats = instance.get_heap_statistics().unwrap();

    Box::into_raw(Box::new(heap_stats))
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

    if !primitive_result.error.is_null() {
        let error = primitive_result.error;

        CString::from_raw((*error).exception);
        CString::from_raw((*error).stack_trace);

        Box::from_raw(primitive_result.error);
    }
}

#[no_mangle]
pub unsafe extern "C" fn free_heap_stats(heap_stats_ptr: *mut V8HeapStatistics) {
    if !heap_stats_ptr.is_null() {
        Box::from_raw(heap_stats_ptr);
    }
}