#[cfg(test)]
mod v8facade_async_tests {
    use javascript_eval_native::{function_parameter::FunctionParameter, v8facade::{JavaScriptResult, Output, V8Facade}};

    #[test]
    fn it_can_eval_simple_script_async() {
        let eval = V8Facade::new();

        eval.begin_run("1+1;", |result| {
            if let Output::Result(r) = result {
                if let JavaScriptResult::NumberValue(n) = r {
                    assert_eq!(2.0, n);
                } else {
                    assert!(false, "Wrong answer.");
                }
            } else {
                assert!(false, "Welp.");
            }
        })
        .unwrap();
    }

    #[test]
    fn it_can_execute_script_with_no_result_async() {
        let eval = V8Facade::new();

        eval.run("function throwMessage(message) { throw message; }")
            .unwrap();

        eval.begin_run("throwMessage(\"Hello from the error!\");", |result| {
            if let Output::Error(r) = result {
                assert_eq!("Hello from the error!", r.exception);
            } else {
                assert!(false, "Welp.");
            }
        })
        .unwrap();
    }

    #[test]
    fn it_can_register_method_and_call_it_async() {
        let eval = V8Facade::new();

        let _ = eval.run("function echo(val) { return val; }").unwrap();

        eval.begin_call(
            "echo",
            vec![FunctionParameter::StringValue(String::from("hello world"))],
            |result| {
                if let Output::Result(r) = result {
                    if let JavaScriptResult::StringValue(s) = r {
                        assert_eq!("hello world", s);
                    } else {
                        assert!(false, "Wrong.");
                    }
                } else {
                    assert!(false, "Welp.");
                } 
            },
        )
        .unwrap();
    }

    #[test]
    fn it_can_get_heap_statistics_async() {
        let eval = V8Facade::new();

        eval.begin_get_heap_statistics(|result|{
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
        }).unwrap();
    }
}
