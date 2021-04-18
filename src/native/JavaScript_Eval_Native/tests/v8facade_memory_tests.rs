#[cfg(test)]
mod v8facade_memory_tests {
    use javascript_eval_native::{function_parameter::FunctionParameter, v8facade::V8Facade};

    #[test]
    #[ignore]
    fn it_wont_leak_memory_like_crazy() {
        let eval = V8Facade::new();

        let _ = eval.run("function echo(val) { return val; }").unwrap();

        let mut last_heap_stats = eval.get_heap_statistics().unwrap();

        let mut allocation_increased = false;
        let mut allocation_decreased = false;

        for _ in 0..25000 {
            let _ = eval
                .call(
                    "echo",
                    vec![FunctionParameter::StringValue(String::from(
                        "The quick brown fox jumped over the lazy moon.",
                    ))],
                )
                .unwrap();

            let current_heap_stats = eval.get_heap_statistics().unwrap();

            if last_heap_stats.total_heap_size > current_heap_stats.total_heap_size {
                allocation_decreased = true;
            }

            if current_heap_stats.total_heap_size > last_heap_stats.total_heap_size {
                allocation_increased = true;
            }

            last_heap_stats = current_heap_stats;
        }

        assert!(allocation_increased);
        assert!(allocation_decreased);
    }
}
