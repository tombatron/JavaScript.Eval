cargo build --release --manifest-path src/native/JavaScript_Eval_Native/Cargo.toml --target-dir /tmp
cp /tmp/release/libjavascript_eval_native.so libs/runtimes/linux-x64/native/.
