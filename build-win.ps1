cargo build --release --manifest-path src/native/JavaScript_Eval_Native/Cargo.toml --target-dir c:\temp
copy C:\temp\release\javascript_eval_native.dll libs\runtimes\win-x64\native
