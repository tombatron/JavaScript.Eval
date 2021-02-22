use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use function_parameter::FunctionParameter;
use primitive_result::PrimitiveResult;
use v8facade::{JavaScriptError, Output, V8Facade};

pub mod function_parameter;
pub mod primitive_result;
pub mod v8facade;

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

#[no_mangle]
pub unsafe extern "C" fn exec(
    v8_facade_ptr: *mut V8Facade,
    script: *const c_char,
) -> *mut PrimitiveResult {
    let script = CStr::from_ptr(script).to_string_lossy().into_owned();

    let instance = {
        assert!(!v8_facade_ptr.is_null());
        &mut *v8_facade_ptr
    };

    let result = instance.run(script).unwrap();

    PrimitiveResult::from_output(result).into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn begin_exec(
    v8_facade_ptr: *mut V8Facade,
    script: *const c_char,
    on_complete: extern "C" fn(*mut PrimitiveResult),
) {
    let script = CStr::from_ptr(script).to_string_lossy().into_owned();

    let instance = {
        assert!(!v8_facade_ptr.is_null());
        &mut *v8_facade_ptr
    };

    instance
        .begin_run(script, move |output| {
            let result = PrimitiveResult::from_output(output);

            on_complete(result.into_raw());
        })
        .unwrap();
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
        Err(e) => Output::Error(JavaScriptError {
            exception: e,
            stack_trace: String::from(""),
        }),
    };

    PrimitiveResult::from_output(result).into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn begin_call(
    v8_facade_ptr: *mut V8Facade,
    func_name: *const c_char,
    parameters: *const Primitive,
    parameter_count: usize,
    on_complete: extern "C" fn(*mut PrimitiveResult),
) {
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

    instance
        .begin_call(func_name, parameters, move |output| {
            let result = PrimitiveResult::from_output(output);

            on_complete(result.into_raw());
        })
        .unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn get_heap_statistics(
    v8_facade_ptr: *mut V8Facade,
) -> *mut V8HeapStatistics {
    let instance = {
        assert!(!v8_facade_ptr.is_null());
        &mut *v8_facade_ptr
    };

    let heap_stats = instance.get_heap_statistics().unwrap();

    Box::into_raw(Box::new(heap_stats))
}

#[no_mangle]
pub unsafe extern "C" fn begin_get_heap_statistics(
    v8_facade_ptr: *mut V8Facade,
    on_complete: extern "C" fn(*mut V8HeapStatistics),
) {
    let instance = {
        assert!(!v8_facade_ptr.is_null());
        &mut *v8_facade_ptr
    };

    instance.begin_get_heap_statistics(move |s| {
        let result = Box::new(s);
        let result = Box::into_raw(result);

        on_complete(result);
    }).unwrap();
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
    PrimitiveResult::free_raw(primitive_result_ptr);
}

#[no_mangle]
pub unsafe extern "C" fn free_heap_stats(heap_stats_ptr: *mut V8HeapStatistics) {
    if !heap_stats_ptr.is_null() {
        Box::from_raw(heap_stats_ptr);
    }
}
