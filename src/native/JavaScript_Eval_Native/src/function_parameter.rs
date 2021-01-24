use std::ffi::CStr;

use crate::Primitive;


#[derive(Debug)]
pub enum FunctionParameter {
    StringValue(String),
    SymbolValue(String),
    NumberValue(f64),
    BigIntValue(i64),
    BoolValue(bool),
    ObjectValue(String),
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