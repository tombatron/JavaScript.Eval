use std::{convert::TryFrom, ffi::CStr, sync::mpsc::RecvError, thread::JoinHandle};

use std::{
    sync::{mpsc, Once},
    unreachable,
};

use rusty_v8 as v8;

use crate::{Primitive, V8HeapStatistics};

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

#[derive(Debug)]
pub enum FunctionParameter {
    StringValue(String),
    SymbolValue(String),
    NumberValue(f64),
    BigIntValue(i64),
    BoolValue(bool),
    ObjectValue(String),
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

impl FunctionParameter {
    pub fn from(p: &Primitive) -> FunctionParameter {
        if !p.string_value.is_null() {
            unsafe {
                return FunctionParameter::StringValue(
                    CStr::from_ptr(p.string_value)
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }

        if !p.symbol_value.is_null() {
            unsafe {
                return FunctionParameter::SymbolValue(
                    CStr::from_ptr(p.symbol_value)
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }

        if !p.object_value.is_null() {
            unsafe {
                return FunctionParameter::ObjectValue(
                    CStr::from_ptr(p.object_value)
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }

        if p.number_value_set {
            return FunctionParameter::NumberValue(p.number_value);
        }

        if p.bigint_value_set {
            return FunctionParameter::BigIntValue(p.bigint_value);
        }

        if p.bool_value_set {
            return FunctionParameter::BoolValue(p.bool_value);
        }

        unreachable!();
    }
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
                
                let scope = &mut v8::ContextScope::new(scope, context);
                
                let global = context.global(scope);

                match input {
                    Input::Source(code) => {
                        let tc = &mut v8::TryCatch::new(scope);
                        let result = V8Facade::eval(tc, code.as_str());

                        //V8Facade::send_result_to_output(result, tc, global, &tx_out);
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

                drop(scope);
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

#[cfg(test)]
mod tests {
    use rusty_v8::inspector::StringBuffer;

    use super::{FunctionParameter, JavaScriptResult, Output, V8Facade};

    #[test]
    fn it_gets_error_with_bad_function_call() {
        let eval = V8Facade::new();
        let result = eval.call("what", vec![]).unwrap();

        if let Output::Error(e) = result {
            assert_eq!("", e.stack_trace);
            assert_eq!(
                "Couldn't resolve function `what`, V8 returned: 'undefined'",
                e.exception
            );
        } else {
            assert!(false, "I guess no error was thrown...");
        }
    }

    #[test]
    fn it_can_get_heap_statistics() {
        let eval = V8Facade::new();
        let result = eval.get_heap_statistics().unwrap();

        assert!(result.used_heap_size > 0);
        assert!(result.total_heap_size > 0);
        assert!(result.total_heap_size > result.used_heap_size);
        assert!(result.heap_size_limit > 0);
        assert!(result.heap_size_limit > result.total_heap_size);
        assert!(result.malloced_memory > 0);
        assert!(result.peak_malloced_memory > 0);
        assert_eq!(result.used_global_handles_size, 0);
        assert_eq!(result.total_global_handles_size, 0);
        assert_ne!(result.number_of_native_contexts, 0);
    }

    #[test]
    fn it_gets_error_when_provided_bad_javascript_for_eval() {
        let eval = V8Facade::new();
        let result = eval
            .run("fucktion () { return \"Hello World!\"; }")
            .unwrap();

        if let Output::Error(e) = result {
            assert_eq!("", e.stack_trace);
            assert_eq!("There was an issue compiling the provided script: SyntaxError: Unexpected token '{'", e.exception);
        } else {
            assert!(false, "I guess no error was throw...");
        }
    }

    #[test]
    fn it_can_eval_simple_script() {
        let eval = V8Facade::new();
        let result = eval.run("1+1;").unwrap();

        if let Output::Result(r) = result {
            if let JavaScriptResult::NumberValue(n) = r {
                assert_eq!(2.0, n);                
            } else {
                assert!(false, "Wrong answer.");
            }
        } else {
            assert!(false, "Welp.");
        }
    }

    #[test]
    fn it_can_register_method_and_call_it() {
        let eval = V8Facade::new();
        
        let _ = eval.run("function echo(val) { return val; }");
        let result = eval.call("echo", vec![FunctionParameter::StringValue(String::from("hello world"))]).unwrap();

        if let Output::Result(r) = result {
            if let JavaScriptResult::StringValue(s) = r {
                assert_eq!("hello world", s);
            } else {
                assert!(false, "Wrong.");
            }
        } else {
            assert!(false, "Welp.");
        }
    }
}
