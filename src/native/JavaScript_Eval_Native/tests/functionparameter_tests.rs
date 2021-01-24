#[cfg(test)]
mod functionparameter_tests {
    use std::{ffi::CString, ptr};

    use javascript_eval_native::{function_parameter::FunctionParameter, Primitive};

    #[test]
    fn it_can_create_from_string_value() {
        let string = CString::new(String::from("Hello World"))
            .unwrap()
            .into_raw();

        let primitive = Primitive {
            string_value: string,

            number_value: 0.0,
            number_value_set: false,
            bigint_value: 0,
            bigint_value_set: false,
            bool_value: false,
            bool_value_set: false,
            object_value: ptr::null_mut(),
            symbol_value: ptr::null_mut(),
        };

        let func_param = FunctionParameter::from(&primitive);

        match func_param {
            FunctionParameter::StringValue(s) => assert_eq!("Hello World", s),
            _ => assert!(false, "Expected value wasn't returned."),
        }

        unsafe {
            let _ = CString::from_raw(string);
        }
    }

    #[test]
    fn it_can_create_from_symbol_value() {
        let symbol = CString::new(String::from("symbol"))
            .unwrap()
            .into_raw();

        let primitive = Primitive {
            symbol_value: symbol,

            number_value: 0.0,
            number_value_set: false,
            bigint_value: 0,
            bigint_value_set: false,
            bool_value: false,
            bool_value_set: false,
            object_value: ptr::null_mut(),
            string_value: ptr::null_mut(),
        };

        let func_param = FunctionParameter::from(&primitive);

        match func_param {
            FunctionParameter::SymbolValue(s) => assert_eq!("symbol", s),
            _ => assert!(false, "Expected value wasn't returned."),
        }

        unsafe {
            let _ = CString::from_raw(symbol);
        }        
    }

    #[test]
    fn it_can_create_from_object_value() {
        let object = CString::new(String::from("object"))
            .unwrap()
            .into_raw();

        let primitive = Primitive {
            object_value: object,

            number_value: 0.0,
            number_value_set: false,
            bigint_value: 0,
            bigint_value_set: false,
            bool_value: false,
            bool_value_set: false,
            string_value: ptr::null_mut(),
            symbol_value: ptr::null_mut(),
        };

        let func_param = FunctionParameter::from(&primitive);

        match func_param {
            FunctionParameter::ObjectValue(s) => assert_eq!("object", s),
            _ => assert!(false, "Expected value wasn't returned."),
        }

        unsafe {
            let _ = CString::from_raw(object);
        }          
    }

    #[test]
    fn it_can_create_from_number_value() {
        let primitive = Primitive {
            number_value: 1.1,
            number_value_set: true,

            bigint_value: 0,
            bigint_value_set: false, 
            bool_value: false,
            bool_value_set: false,
            string_value: ptr::null_mut(),
            symbol_value: ptr::null_mut(),
            object_value: ptr::null_mut(),
        };

        let func_param = FunctionParameter::from(&primitive);

        match func_param {
            FunctionParameter::NumberValue(n) => assert_eq!(1.1, n), 
            _ => assert!(false, "Expected value wasn't returned."),
        }
    }

    #[test]
    fn it_can_create_from_bigint_value() {
        let primitive = Primitive {
            bigint_value: 1,
            bigint_value_set: true, 

            number_value: 0.0,
            number_value_set: false,
            bool_value: false,
            bool_value_set: false,
            string_value: ptr::null_mut(),
            symbol_value: ptr::null_mut(),
            object_value: ptr::null_mut(),
        };

        let func_param = FunctionParameter::from(&primitive);

        match func_param {
            FunctionParameter::BigIntValue(n) => assert_eq!(1, n), 
            _ => assert!(false, "Expected value wasn't returned."),
        }        
    }

    #[test]
    fn it_can_create_from_bool_value() {
        let primitive = Primitive {
            bool_value: true,
            bool_value_set: true,

            number_value: 0.0,
            number_value_set: false,
            bigint_value: 0,
            bigint_value_set: false, 
            string_value: ptr::null_mut(),
            symbol_value: ptr::null_mut(),
            object_value: ptr::null_mut(),
        };

        let func_param = FunctionParameter::from(&primitive);

        match func_param {
            FunctionParameter::BoolValue(b) => assert!(b),
            _ => assert!(false, "Expected value wasn't returned."),
        }
    }
}
