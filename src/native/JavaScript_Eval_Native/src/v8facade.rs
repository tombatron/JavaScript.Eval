use std::{
    convert::TryFrom, ffi::CString, os::raw::c_char, sync::mpsc::RecvError, thread::JoinHandle,
};

use std::{
    sync::{mpsc, Once},
    unreachable,
};

use rusty_v8 as v8;

use crate::{
    function_parameter::FunctionParameter, primitive_result::PrimitiveResult, V8HeapStatistics,
};

static INIT_PLATFORM: Once = Once::new();

fn init_platform() {
    let platform = v8::new_default_platform().unwrap();

    v8::V8::initialize_platform(platform);
    v8::V8::initialize();
}

enum Input {
    Source(String),
    Function(FunctionCall),
    HeapReport,

    BeginSource(String, fn(*mut PrimitiveResult)),
    BeginFunction(FunctionCall, fn(*mut PrimitiveResult)),
}

pub enum Output {
    Result(JavaScriptResult),
    Error(JavaScriptError),
    HeapStatistics(V8HeapStatistics),
}

pub struct FunctionCall {
    name: String,
    arguments: Vec<FunctionParameter>,
}

pub enum JavaScriptResult {
    StringValue(String),
    NumberValue(f64),
    BigIntValue(i64),
    BoolValue(bool),

    // These will be tossed back as JSON strings.
    ArrayValue(String),
    ObjectValue(String),
}

impl JavaScriptResult {
    pub fn from<'s>(
        value: v8::Local<v8::Value>,
        scope: &mut v8::HandleScope<'s>,
        global: v8::Local<v8::Object>,
    ) -> JavaScriptResult {
        if value.is_string() {
            let string_result = value.to_string(scope).unwrap();
            JavaScriptResult::StringValue(string_result.to_rust_string_lossy(scope))
        } else if value.is_number() {
            let number_result = value.to_number(scope).unwrap();
            JavaScriptResult::NumberValue(number_result.value())
        } else if value.is_big_int() {
            let bigint_result = value.to_big_int(scope).unwrap();
            JavaScriptResult::BigIntValue(bigint_result.i64_value().0)
        } else if value.is_boolean() {
            let bool_result = value.to_boolean(scope);
            JavaScriptResult::BoolValue(bool_result.boolean_value(scope))
        } else {
            let json = v8::String::new(scope, "JSON").unwrap();
            let json = v8::Local::from(json);
            let json = global.get(scope, json).unwrap();
            let json = v8::Local::<v8::Object>::try_from(json).unwrap();

            let stringify = v8::String::new(scope, "stringify").unwrap();
            let stringify = v8::Local::from(stringify);
            let stringify = json.get(scope, stringify).unwrap();
            let stringify = v8::Local::<v8::Function>::try_from(stringify).unwrap();

            let string_result = stringify.call(scope, global.into(), &[value]).unwrap();
            let string_result = string_result.to_string(scope).unwrap();
            let string_result = string_result.to_rust_string_lossy(scope);

            if value.is_array() {
                JavaScriptResult::ArrayValue(string_result)
            } else if value.is_object() {
                JavaScriptResult::ObjectValue(string_result)
            } else {
                JavaScriptResult::StringValue(string_result)
            }
        }
    }
}

pub struct JavaScriptError {
    pub exception: String,
    pub stack_trace: String,
}

pub struct V8Facade {
    input: mpsc::Sender<Input>,
    output: mpsc::Receiver<Output>,
    _handle: JoinHandle<Result<(), RecvError>>, // TODO: Signal the managed code that something happend here...
}

impl V8Facade {
    // https://github.com/denoland/rusty_v8/blob/584a0378002d2f952c55dd5f3d34ea2017ed0c7b/tests/test_api.rs#L565
    fn eval<'s>(
        scope: &mut v8::TryCatch<'s, v8::HandleScope>,
        code: &str,
    ) -> Result<Option<v8::Local<'s, v8::Value>>, String> {
        let source = v8::String::new(scope, &code).unwrap();
        let script = v8::Script::compile(scope, source, None);

        match script {
            Some(script) => {
                let scope = &mut v8::EscapableHandleScope::new(scope);
                let r = script.run(scope);
                Ok(r.map(|v| scope.escape(v)))
            }

            None => {
                let exception = scope.exception().unwrap();
                let exception = exception.to_rust_string_lossy(scope);

                Err(format!(
                    "There was an issue compiling the provided script: {}",
                    exception
                ))
            }
        }
    }

    fn call_func<'s>(
        scope: &mut v8::HandleScope<'s>,
        global: v8::Local<v8::Object>,
        func_args: &FunctionCall,
    ) -> Result<Option<v8::Local<'s, v8::Value>>, String> {
        let scope = &mut v8::EscapableHandleScope::new(scope);

        let func_name = v8::String::new(scope, &func_args.name).unwrap();
        let func_name = v8::Local::from(func_name);

        let func = global.get(scope, func_name).unwrap();
        let func = v8::Local::<v8::Function>::try_from(func).map_err(|_| {
            format!(
                "Couldn't resolve function `{}`, V8 returned: '{}'",
                func_args.name,
                func.to_rust_string_lossy(scope),
            )
        })?;

        let args: Vec<v8::Local<v8::Value>> = func_args
            .arguments
            .iter()
            .map(|p| -> v8::Local<v8::Value> {
                match p {
                    FunctionParameter::StringValue(v) => {
                        v8::String::new(scope, v.as_str()).unwrap().into()
                    }

                    FunctionParameter::NumberValue(v) => v8::Number::new(scope, *v).into(),

                    FunctionParameter::BigIntValue(v) => v8::BigInt::new_from_i64(scope, *v).into(),

                    FunctionParameter::BoolValue(v) => v8::Boolean::new(scope, *v).into(),

                    FunctionParameter::SymbolValue(v) => {
                        let desc = v8::String::new(scope, v.as_str());

                        v8::Symbol::new(scope, desc).into()
                    }

                    FunctionParameter::ObjectValue(o) => {
                        let object_json = v8::String::new(scope, o.as_str()).unwrap();

                        V8Facade::json_parse(object_json.into(), scope, global)
                    }
                }
            })
            .collect();

        let args = args.as_slice();

        let result = func.call(scope, global.into(), args);
        Result::Ok(result.map(|v| scope.escape(v)))
    }

    fn send_result_to_output(
        result: Option<v8::Local<v8::Value>>,
        scope: &mut v8::TryCatch<v8::HandleScope>,
        global: v8::Local<v8::Object>,
        tx_out: &mpsc::Sender<Output>,
    ) {
        match result {
            Some(v) => {
                let result = JavaScriptResult::from(v, scope, global);

                tx_out.send(Output::Result(result)).unwrap();
            }

            None => {
                let exception = scope.exception().unwrap();
                let exception = exception.to_rust_string_lossy(scope);

                let stack_trace = scope.stack_trace().unwrap();
                let stack_trace = stack_trace.to_rust_string_lossy(scope);

                tx_out
                    .send(Output::Error(JavaScriptError {
                        exception,
                        stack_trace,
                    }))
                    .unwrap();
            }
        }
    }

    fn send_result_to_delegate(
        result: Option<v8::Local<v8::Value>>,
        scope: &mut v8::TryCatch<v8::HandleScope>,
        global: v8::Local<v8::Object>,
        on_complete: fn(*mut PrimitiveResult),
    ) {
        match result {
            Some(v) => {
                let result = JavaScriptResult::from(v, scope, global);
                let result = PrimitiveResult::from_javascriptresult(result);

                on_complete(result.into_raw());
            }

            None => {
                let exception = scope.exception().unwrap();
                let exception = exception.to_rust_string_lossy(scope);

                let stack_trace = scope.stack_trace().unwrap();
                let stack_trace = stack_trace.to_rust_string_lossy(scope);

                let result = JavaScriptError {
                    exception,
                    stack_trace,
                };

                let result = PrimitiveResult::create_for_error(result);

                on_complete(result.into_raw());
            }
        }
    }

    fn json_parse<'s>(
        json_value: v8::Local<v8::Value>,
        scope: &mut v8::HandleScope<'s>,
        global: v8::Local<v8::Object>,
    ) -> v8::Local<'s, v8::Value> {
        let scope = &mut v8::EscapableHandleScope::new(scope);

        let json = v8::String::new(scope, "JSON").unwrap();
        let json = v8::Local::from(json);
        let json = global.get(scope, json).unwrap();
        let json = v8::Local::<v8::Object>::try_from(json).unwrap();

        let parse = v8::String::new(scope, "parse").unwrap();
        let parse = v8::Local::from(parse);
        let parse = json.get(scope, parse).unwrap();
        let parse = v8::Local::<v8::Function>::try_from(parse).unwrap();

        let parse_result = parse.call(scope, global.into(), &[json_value]).unwrap();

        scope.escape(parse_result)
    }

    #[allow(unreachable_code)]
    pub fn new() -> Self {
        INIT_PLATFORM.call_once(init_platform);

        let (tx_in, rx_in) = mpsc::channel::<Input>();
        let (tx_out, rx_out) = mpsc::channel::<Output>();

        let handle = std::thread::spawn(move || {
            let isolate = &mut v8::Isolate::new(Default::default());
            let scope = &mut v8::HandleScope::new(isolate);
            let context = v8::Context::new(scope);

            loop {
                let input = rx_in.recv()?;

                let scope = &mut v8::HandleScope::new(scope);
                let scope = &mut v8::ContextScope::new(scope, context);

                let global = context.global(scope);

                match input {
                    Input::Source(code) => {
                        let tc = &mut v8::TryCatch::new(scope);
                        let result = V8Facade::eval(tc, code.as_str());

                        match result {
                            Ok(result) => {
                                V8Facade::send_result_to_output(result, tc, global, &tx_out);
                            }

                            Err(error) => {
                                let error = JavaScriptError {
                                    exception: error,
                                    stack_trace: String::from(""),
                                };

                                tx_out.send(Output::Error(error)).unwrap();
                            }
                        }
                    }

                    Input::BeginSource(code, on_complete) => {
                        let tc = &mut v8::TryCatch::new(scope);
                        let result = V8Facade::eval(tc, code.as_str());

                        match result {
                            Ok(result) => {
                                V8Facade::send_result_to_delegate(result, tc, global, on_complete);
                            }

                            Err(error) => {
                                let error = JavaScriptError {
                                    exception: error,
                                    stack_trace: String::from(""),
                                };

                                let error = PrimitiveResult::create_for_error(error);

                                on_complete(error.into_raw());
                            }
                        }
                    }

                    Input::Function(func_args) => {
                        let tc = &mut v8::TryCatch::new(scope);
                        let result = V8Facade::call_func(tc, global, &func_args);

                        match result {
                            Ok(result) => {
                                V8Facade::send_result_to_output(result, tc, global, &tx_out)
                            }

                            Err(error) => {
                                let error = JavaScriptError {
                                    exception: error,
                                    stack_trace: String::from(""),
                                };

                                tx_out.send(Output::Error(error)).unwrap();
                            }
                        };
                    }

                    Input::BeginFunction(func_args, on_complete) => {
                        let tc = &mut v8::TryCatch::new(scope);
                        let result = V8Facade::call_func(tc, global, &func_args);

                        match result {
                            Ok(result) => {
                                V8Facade::send_result_to_delegate(result, tc, global, on_complete)
                            }

                            Err(error) => {
                                let error = JavaScriptError {
                                    exception: error,
                                    stack_trace: String::from(""),
                                };

                                let error = PrimitiveResult::create_for_error(error);

                                on_complete(error.into_raw());
                            }
                        };                        
                    }

                    Input::HeapReport => {
                        let heap_stats = &mut v8::HeapStatistics::default();

                        scope.get_heap_statistics(heap_stats);

                        let heap_stats = V8HeapStatistics {
                            total_heap_size: heap_stats.total_heap_size(),
                            total_heap_size_executable: heap_stats.total_heap_size_executable(),
                            total_physical_size: heap_stats.total_physical_size(),
                            total_available_size: heap_stats.total_available_size(),
                            used_heap_size: heap_stats.used_heap_size(),
                            heap_size_limit: heap_stats.heap_size_limit(),
                            malloced_memory: heap_stats.malloced_memory(),
                            does_zap_garbage: heap_stats.does_zap_garbage(),
                            number_of_native_contexts: heap_stats.number_of_native_contexts(),
                            number_of_detached_contexts: heap_stats.number_of_detached_contexts(),
                            peak_malloced_memory: heap_stats.peak_malloced_memory(),
                            used_global_handles_size: heap_stats.used_global_handles_size(),
                            total_global_handles_size: heap_stats.total_global_handles_size(),
                        };

                        tx_out.send(Output::HeapStatistics(heap_stats)).unwrap();
                    }
                };
            }

            unreachable!();
        });

        Self {
            input: tx_in,
            output: rx_out,
            _handle: handle,
        }
    }

    pub fn run<S: Into<String>>(&self, source: S) -> Result<Output, String> {
        self.input
            .send(Input::Source(source.into()))
            .map_err(|e| format!("{:?}", e))?;

        self.output.recv().map_err(|e| format!("{:?}", e))
    }

    pub fn begin_run<S: Into<String>>(
        &self,
        source: S,
        on_complete: fn(*mut PrimitiveResult),
    ) -> Result<(), String> {
        self.input
            .send(Input::BeginSource(source.into(), on_complete))
            .map_err(|e| format!("{:?}", e))?;

        Ok(())
    }

    pub fn call<S: Into<String>>(
        &self,
        func_name: S,
        func_params: Vec<FunctionParameter>,
    ) -> Result<Output, String> {
        let call_spec = Input::Function(FunctionCall {
            name: func_name.into(),
            arguments: func_params,
        });

        self.input.send(call_spec).map_err(|e| format!("{:?}", e))?;

        self.output.recv().map_err(|e| format!("{:?}", e))
    }

    pub fn begin_call<S: Into<String>>(
        &self,
        func_name: S,
        func_params: Vec<FunctionParameter>,
        on_complete: fn(*mut PrimitiveResult),
    ) -> Result<(), String> {
        let call_spec = Input::BeginFunction(
            FunctionCall {
                name: func_name.into(),
                arguments: func_params,
            },
            on_complete,
        );

        self.input.send(call_spec).map_err(|e| format!("{:?}", e))?;

        Ok(())
    }

    pub fn get_heap_statistics(&self) -> Result<V8HeapStatistics, String> {
        self.input
            .send(Input::HeapReport)
            .map_err(|e| format!("{:?}", e))?;

        let result = self.output.recv().map_err(|e| format!("{:?}", e))?;

        if let Output::HeapStatistics(s) = result {
            Ok(s)
        } else {
            Err(String::from("Couldn't get the heap statistics..."))
        }
    }
}
