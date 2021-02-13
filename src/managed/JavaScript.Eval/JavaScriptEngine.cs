using JavaScript.Eval.Exceptions;
using System;
using System.Runtime.InteropServices;
using System.Text.Json;
using System.Threading.Tasks;

namespace JavaScript.Eval
{
    internal sealed class JavaScriptEngineHandle : SafeHandle
    {
        public JavaScriptEngineHandle() : base(IntPtr.Zero, true) { }

        public override bool IsInvalid => this.handle == IntPtr.Zero;

        protected override bool ReleaseHandle()
        {
            if (!this.IsInvalid)
            {
                Native.free_v8(handle);
            }

            return true;
        }
    }
    public sealed class JavaScriptEngine : IDisposable
    {
        private readonly JavaScriptEngineHandle _handle;
        private bool _isDisposed = false;

        public delegate void OnComplete(IntPtr result);

        public JavaScriptEngine()
        {
            _handle = Native.get_v8();
        }

        public TResult Eval<TResult>(string script)
        {
            var scriptPointer = Marshal.StringToCoTaskMemUTF8(script);

            var primitiveResultPointer = Native.exec(_handle, scriptPointer);
            var primitiveResult = Marshal.PtrToStructure<PrimitiveResult>(primitiveResultPointer);

            var result = MapPrimitiveResult<TResult>(primitiveResult);

            Marshal.FreeCoTaskMem(scriptPointer);
            Native.free_primitive_result(primitiveResultPointer);

            return result;
        }

        public Task<TResult> EvalAsync<TResult>(string script)
        {
            var resultSource = new TaskCompletionSource<TResult>();

            var scriptPointer = Marshal.StringToCoTaskMemUTF8(script);

            Native.begin_exec(_handle, scriptPointer, (resultPointer) =>
            {
                try
                {
                    var primitiveResult = Marshal.PtrToStructure<PrimitiveResult>(resultPointer);

                    var result = MapPrimitiveResult<TResult>(primitiveResult);

                    resultSource.SetResult(result);
                }
                catch (Exception ex)
                {
                    resultSource.SetException(ex);
                }
                finally
                {
                    Marshal.FreeCoTaskMem(scriptPointer);
                    Native.free_primitive_result(resultPointer);
                }
            });

            return resultSource.Task;
        }

        public void Eval(string script)
        {
            var scriptPointer = Marshal.StringToCoTaskMemUTF8(script);

            var primitiveResultPointer = Native.exec(_handle, scriptPointer);
            var primitiveResult = Marshal.PtrToStructure<PrimitiveResult>(primitiveResultPointer);

            if (TryCheckForException(primitiveResult, out var exception))
            {
                throw exception;
            }

            Marshal.FreeCoTaskMem(scriptPointer);
            Native.free_primitive_result(primitiveResultPointer);
        }

        public Task EvalAsync(string script)
        {
            var resultSource = new TaskCompletionSource<Task>();

            var scriptPointer = Marshal.StringToCoTaskMemUTF8(script);

            Native.begin_exec(_handle, scriptPointer, (resultPointer) =>
            {
                try
                {
                    var primitiveResult = Marshal.PtrToStructure<PrimitiveResult>(resultPointer);

                    if (TryCheckForException(primitiveResult, out var exception))
                    {
                        resultSource.SetException(exception);
                    }
                }
                catch (Exception ex)
                {
                    resultSource.SetException(ex);
                }
                finally
                {
                    Marshal.FreeCoTaskMem(scriptPointer);
                    Native.free_primitive_result(resultPointer);
                }
            });

            return resultSource.Task;
        }

        public TResult Call<TResult>(string funcName, params Primitive[] funcParams)
        {
            var funcNamePointer = Marshal.StringToCoTaskMemUTF8(funcName);

            var primitiveResultPointer = Native.call(_handle, funcNamePointer, funcParams, funcParams.Length);
            var primitiveResult = Marshal.PtrToStructure<PrimitiveResult>(primitiveResultPointer);

            var result = MapPrimitiveResult<TResult>(primitiveResult);

            Marshal.FreeCoTaskMem(funcNamePointer);
            Primitive.Free(funcParams);
            Native.free_primitive_result(primitiveResultPointer);

            return result;
        }

        public HeapStatistics GetHeapStatistics()
        {
            var heapStatisticsPointer = Native.get_heap_statistics(_handle);
            var heapStatistics = Marshal.PtrToStructure<HeapStatistics>(heapStatisticsPointer);

            Native.free_heap_stats(heapStatisticsPointer);

            return heapStatistics;
        }

        private TResult MapPrimitiveResult<TResult>(PrimitiveResult primitiveResult)
        {
            if (primitiveResult.number_value_set > 0)
            {
                return (TResult)Convert.ChangeType(primitiveResult.number_value, typeof(TResult));
            }
            else if (primitiveResult.bigint_value_set > 0)
            {
                return (TResult)Convert.ChangeType(primitiveResult.bigint_value, typeof(TResult));
            }
            else if (primitiveResult.bool_value_set > 0)
            {
                return (TResult)Convert.ChangeType(primitiveResult.bool_value, typeof(TResult));
            }
            else if (primitiveResult.string_value != IntPtr.Zero)
            {
                var stringValue = Marshal.PtrToStringUTF8(primitiveResult.string_value);

                return (TResult)Convert.ChangeType(stringValue, typeof(TResult));
            }
            else if (primitiveResult.array_value != IntPtr.Zero)
            {
                var arrayStringValue = Marshal.PtrToStringUTF8(primitiveResult.array_value);

                return JsonSerializer.Deserialize<TResult>(arrayStringValue);
            }
            else if (primitiveResult.object_value != IntPtr.Zero)
            {
                var objectStringValue = Marshal.PtrToStringUTF8(primitiveResult.object_value);

                return JsonSerializer.Deserialize<TResult>(objectStringValue);
            }
            else if (TryCheckForException(primitiveResult, out var exception))
            {
                throw exception;
            }
            else
            {
                return default(TResult);
            }
        }

        private bool TryCheckForException(PrimitiveResult primitiveResult, out JavaScriptException exception)
        {
            if (primitiveResult.error != IntPtr.Zero)
            {
                var error = Marshal.PtrToStructure<UnsafeJavaScriptError>(primitiveResult.error);

                exception = new JavaScriptException(error);

                return true;
            }
            else
            {
                exception = default;

                return false;
            }
        }

        public void Dispose()
        {
            _handle.Dispose();

            _isDisposed = true;
        }
    }

    internal static class Native
    {
        private const string LIB_NAME = "javascript_eval_native";

        [DllImport(LIB_NAME)]
        internal static extern JavaScriptEngineHandle get_v8();

        [DllImport(LIB_NAME)]
        internal static extern void free_v8(IntPtr handle);

        [DllImport(LIB_NAME)]
        internal static extern IntPtr exec(JavaScriptEngineHandle handle, IntPtr script);

        [DllImport(LIB_NAME)]
        internal static extern void begin_exec(JavaScriptEngineHandle handle, IntPtr script, JavaScriptEngine.OnComplete on_complete);

        [DllImport(LIB_NAME)]
        internal static extern IntPtr call(JavaScriptEngineHandle handle, IntPtr func_name, Primitive[] parameters, int parameterCount);

        [DllImport(LIB_NAME)]
        internal static extern void begin_call(JavaScriptEngineHandle handle, IntPtr func_name, Primitive[] parameters, int parameterCount, JavaScriptEngine.OnComplete on_complete);

        [DllImport(LIB_NAME)]
        internal static extern void free_string(IntPtr stringPointer);

        [DllImport(LIB_NAME)]
        internal static extern void free_primitive_result(IntPtr handle);

        [DllImport(LIB_NAME)]
        internal static extern IntPtr get_heap_statistics(JavaScriptEngineHandle handle);

        [DllImport(LIB_NAME)]
        internal static extern void begin_get_heap_statistics(JavaScriptEngineHandle handle, JavaScriptEngine.OnComplete on_complete);

        [DllImport(LIB_NAME)]
        internal static extern void free_heap_stats(IntPtr statisticsHandle);
    }
}