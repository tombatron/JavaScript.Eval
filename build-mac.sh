cargo build --release --manifest-path src/native/JavaScript_Eval_Native/Cargo.toml --target-dir /tmp
mkdir libs/runtimes/osx-x64/native
cp /tmp/release/libjavascript_eval_native.dylib libs/runtimes/osx-x64/native/.
