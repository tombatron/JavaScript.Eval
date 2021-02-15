#[cfg(test)]
mod v8facade_tests {
    use javascript_eval_native::{
        function_parameter::FunctionParameter,
        primitive_result::PrimitiveResult,
        v8facade::{JavaScriptResult, Output, V8Facade},
    };

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
    fn it_can_execute_script_with_no_result() {
        let eval = V8Facade::new();

        eval.run("function throwMessage(message) { throw message; }")
            .unwrap();

        let result = eval
            .run("throwMessage(\"Hello from the error!\");")
            .unwrap();

        if let Output::Error(r) = result {
            assert_eq!("Hello from the error!", r.exception);
        } else {
            assert!(false, "Welp.");
        }
    }

    #[test]
    fn it_can_register_method_and_call_it() {
        let eval = V8Facade::new();

        let _ = eval.run("function echo(val) { return val; }");
        let result = eval
            .call(
                "echo",
                vec![FunctionParameter::StringValue(String::from("hello world"))],
            )
            .unwrap();

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
    fn it_can_get_array_result() {
        let eval = V8Facade::new();

        let _ = eval.run("function getArray() { return [1,2,3];}");
        let result = eval.call("getArray", vec![]).unwrap();

        if let Output::Result(r) = result {
            if let JavaScriptResult::ArrayValue(s) = r {
                assert_eq!("[1,2,3]", s);
            } else {
                assert!(false, "Wrong.");
            }
        } else {
            assert!(false, "Welp.");
        }
    }
}
