use std::{convert::TryFrom, ffi::CStr, sync::mpsc::RecvError, thread::JoinHandle};

use std::{
    sync::{mpsc, Once},
    unreachable,
};

use rusty_v8 as v8;

use crate::Primitive;

static INIT_PLATFORM: Once = Once::new();

fn init_platform() {
    let platform = v8::new_default_platform().unwrap();

    v8::V8::initialize_platform(platform);
    v8::V8::initialize();
}

enum Input {
    Source(String),
    Function(FunctionCall),
}

pub enum Output {
    Result(JavaScriptResult),
    Error(JavaScriptError),
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
    fn eval<'s>(scope: &mut v8::HandleScope<'s>, code: &str) -> Option<v8::Local<'s, v8::Value>> {
        let scope = &mut v8::EscapableHandleScope::new(scope);
        let source = v8::String::new(scope, &code).unwrap();
        let script = v8::Script::compile(scope, source, None).unwrap();

        let r = script.run(scope);
        r.map(|v| scope.escape(v))
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
                }
            })
            .collect();

        let args = args.as_slice();

        let result = func.call(scope, global.into(), args);
        Result::Ok(result.map(|v| scope.escape(v)))
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
            let scope = &mut v8::ContextScope::new(scope, context);

            let global = context.global(scope);

            loop {
                let input = rx_in.recv()?;

                match input {
                    Input::Source(code) => {
                        let tc = &mut v8::TryCatch::new(scope);
                        let result = V8Facade::eval(tc, code.as_str());

                        match result {
                            Some(v) => {
                                let result = if v.is_string() {
                                    let string_result = v.to_string(tc).unwrap();
                                    JavaScriptResult::StringValue(
                                        string_result.to_rust_string_lossy(tc),
                                    )
                                } else if v.is_number() {
                                    let number_result = v.to_number(tc).unwrap();
                                    JavaScriptResult::NumberValue(number_result.value())
                                } else if v.is_big_int() {
                                    let bigint_result = v.to_big_int(tc).unwrap();
                                    JavaScriptResult::BigIntValue(bigint_result.i64_value().0)
                                } else if v.is_boolean() {
                                    let bool_result = v.to_boolean(tc);
                                    JavaScriptResult::BoolValue(bool_result.boolean_value(tc))
                                } else {
                                    let json = v8::String::new(tc, "JSON").unwrap();
                                    let json = v8::Local::from(json);
                                    let json = global.get(tc, json).unwrap();
                                    let json = v8::Local::<v8::Object>::try_from(json).unwrap();

                                    let stringify = v8::String::new(tc, "stringify").unwrap();
                                    let stringify = v8::Local::from(stringify);
                                    let stringify = json.get(tc, stringify).unwrap();
                                    let stringify =
                                        v8::Local::<v8::Function>::try_from(stringify).unwrap();

                                    let string_result =
                                        stringify.call(tc, global.into(), &[v]).unwrap();
                                    let string_result = string_result.to_string(tc).unwrap();
                                    let string_result = string_result.to_rust_string_lossy(tc);

                                    if v.is_array() {
                                        JavaScriptResult::ArrayValue(string_result)
                                    } else if v.is_object() {
                                        JavaScriptResult::ObjectValue(string_result)
                                    } else {
                                        JavaScriptResult::StringValue(string_result)
                                    }
                                };

                                tx_out.send(Output::Result(result)).unwrap();
                            }
                            None => {
                                let exception = tc.exception().unwrap();
                                let exception = exception.to_rust_string_lossy(tc);

                                let stack_trace = tc.stack_trace().unwrap();
                                let stack_trace = stack_trace.to_rust_string_lossy(tc);

                                tx_out
                                    .send(Output::Error(JavaScriptError {
                                        exception,
                                        stack_trace,
                                    }))
                                    .unwrap();
                            }
                        }
                    }

                    Input::Function(func_args) => {
                        let tc = &mut v8::TryCatch::new(scope);
                        let result = V8Facade::call_func(tc, global, &func_args);

                        match result {
                            Ok(result) => match result {
                                Some(v) => {
                                    let result = if v.is_string() {
                                        let string_result = v.to_string(tc).unwrap();
                                        JavaScriptResult::StringValue(
                                            string_result.to_rust_string_lossy(tc),
                                        )
                                    } else if v.is_number() {
                                        let number_result = v.to_number(tc).unwrap();
                                        JavaScriptResult::NumberValue(number_result.value())
                                    } else if v.is_big_int() {
                                        let bigint_result = v.to_big_int(tc).unwrap();
                                        JavaScriptResult::BigIntValue(bigint_result.i64_value().0)
                                    } else if v.is_boolean() {
                                        let bool_result = v.to_boolean(tc);
                                        JavaScriptResult::BoolValue(bool_result.boolean_value(tc))
                                    } else {
                                        let json = v8::String::new(tc, "JSON").unwrap();
                                        let json = v8::Local::from(json);
                                        let json = global.get(tc, json).unwrap();
                                        let json = v8::Local::<v8::Object>::try_from(json).unwrap();

                                        let stringify = v8::String::new(tc, "stringify").unwrap();
                                        let stringify = v8::Local::from(stringify);
                                        let stringify = json.get(tc, stringify).unwrap();
                                        let stringify =
                                            v8::Local::<v8::Function>::try_from(stringify).unwrap();

                                        let string_result =
                                            stringify.call(tc, global.into(), &[v]).unwrap();
                                        let string_result = string_result.to_string(tc).unwrap();
                                        let string_result = string_result.to_rust_string_lossy(tc);

                                        if v.is_array() {
                                            JavaScriptResult::ArrayValue(string_result)
                                        } else if v.is_object() {
                                            JavaScriptResult::ObjectValue(string_result)
                                        } else {
                                            JavaScriptResult::StringValue(string_result)
                                        }
                                    };

                                    tx_out.send(Output::Result(result)).unwrap();
                                }

                                None => {
                                    let exception = tc.exception().unwrap();
                                    let exception = exception.to_rust_string_lossy(tc);

                                    let stack_trace = tc.stack_trace().unwrap();
                                    let stack_trace = stack_trace.to_rust_string_lossy(tc);

                                    tx_out
                                        .send(Output::Error(JavaScriptError {
                                            exception,
                                            stack_trace,
                                        }))
                                        .unwrap();
                                }
                            },

                            Err(error) => {
                                let error = JavaScriptError {
                                    exception: error,
                                    stack_trace: String::from(""),
                                };

                                tx_out.send(Output::Error(error)).unwrap();
                            }
                        };
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
        // TODO: Maybe come up with an error struct?
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
}

#[cfg(test)]
mod tests {
    use super::{Output, V8Facade};

    #[test]
    fn it_gets_error_with_bad_function_call() {
        let eval = V8Facade::new();
        let result = eval.call("what", vec![]).unwrap();

        if let Output::Error(e) = result {
            assert_eq!("", e.stack_trace);
            assert_eq!("Couldn't resolve function `what`, V8 returned: 'undefined'", e.exception);
        } else {
            assert!(false, "I guess no error was thrown...");
        }
    }
}
