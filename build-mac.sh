cargo build --release --manifest-path src/native/JavaScript_Eval_Native/Cargo.toml --target-dir /tmp
cp /tmp/release/javascript_eval_native.dylib libs/runtimes/osx-x64/native/.
