#[cfg(test)]
mod v8facade_error_handling_tests {
    use javascript_eval_native::v8facade::{Output, V8Facade};

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
}
