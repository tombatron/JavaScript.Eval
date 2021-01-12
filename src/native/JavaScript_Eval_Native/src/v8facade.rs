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
    Error(String),
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

impl FunctionParameter {
    pub fn from(p: &Primitive) -> FunctionParameter {
        println!("from call: {:?}", p);
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
                        let script_source = v8::String::new(scope, &code).unwrap();
                        let script = v8::Script::compile(scope, script_source, None).unwrap();

                        let result = script.run(scope).unwrap();
                        let result = result.to_string(scope).unwrap();

                        let output = result.to_rust_string_lossy(scope);
                        let output = JavaScriptResult::StringValue(output);

                        tx_out.send(Output::Result(output)).unwrap();
                    }

                    Input::Function(func_args) => {
                        let func_name = v8::String::new(scope, &func_args.name).unwrap();
                        let func_name = v8::Local::from(func_name);

                        let func = global.get(scope, func_name).unwrap();
                        let func = v8::Local::<v8::Function>::try_from(func).unwrap();

                        let args: Vec<v8::Local<v8::Value>> = func_args
                            .arguments
                            .iter()
                            .map(|p| -> v8::Local<v8::Value> {
                                match p {
                                    FunctionParameter::StringValue(v) => {
                                        v8::String::new(scope, v.as_str()).unwrap().into()
                                    }

                                    FunctionParameter::NumberValue(v) => {
                                        v8::Number::new(scope, *v).into()
                                    }
                                    FunctionParameter::BigIntValue(v) => {
                                        v8::BigInt::new_from_i64(scope, *v).into()
                                    }
                                    FunctionParameter::BoolValue(v) => {
                                        v8::Boolean::new(scope, *v).into()
                                    }

                                    FunctionParameter::SymbolValue(v) => {
                                        let desc = v8::String::new(scope, v.as_str());

                                        v8::Symbol::new(scope, desc).into()
                                    }
                                }
                            })
                            .collect();

                        let args = args.as_slice();

                        let result = func.call(scope, global.into(), args).unwrap();

                        let result = if result.is_string() {
                            let string_result = result.to_string(scope).unwrap();
                            JavaScriptResult::StringValue(string_result.to_rust_string_lossy(scope))
                        } else if result.is_number() {
                            let number_result = result.to_number(scope).unwrap();
                            JavaScriptResult::NumberValue(number_result.value())
                        } else if result.is_big_int() {
                            let bigint_result = result.to_big_int(scope).unwrap();
                            JavaScriptResult::BigIntValue(bigint_result.i64_value().0)
                        } else if result.is_boolean() {
                            let bool_result = result.to_boolean(scope);
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

                            let string_result =
                                stringify.call(scope, global.into(), &[result]).unwrap();
                            let string_result = string_result.to_string(scope).unwrap();
                            let string_result = string_result.to_rust_string_lossy(scope);

                            if result.is_array() {
                                JavaScriptResult::ArrayValue(string_result)
                            } else if result.is_object() {
                                JavaScriptResult::ObjectValue(string_result)
                            } else {
                                JavaScriptResult::StringValue(string_result)
                            }
                        };

                        tx_out.send(Output::Result(result)).unwrap();
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
