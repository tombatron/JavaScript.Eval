using System;
using System.Text.Json;
using System.Runtime.InteropServices;

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

        public JavaScriptEngine()
        {
            _handle = Native.get_v8();
        }

        public string Eval(string script)
        {
            var scriptPointer = Marshal.StringToHGlobalAuto(script);

            var resultPointer = Native.exec(_handle, scriptPointer);

            Marshal.FreeHGlobal(scriptPointer);

            return CreateManagedString(resultPointer);
        }

        public TResult Call<TResult>(string funcName, params Primitive[] funcParams)
        {
            var funcNamePointer = Marshal.StringToHGlobalAuto(funcName);

            var primitiveResultPointer = Native.call(_handle, funcNamePointer, funcParams, funcParams.Length);
            var primitiveResult = Marshal.PtrToStructure<PrimitiveResult>(primitiveResultPointer);

            TResult result;

            if (primitiveResult.number_value_set > 0)
            {
                result = (TResult)Convert.ChangeType(primitiveResult.number_value, typeof(TResult));
            }
            else if (primitiveResult.bigint_value_set > 0)
            {
                result = (TResult)Convert.ChangeType(primitiveResult.bigint_value, typeof(TResult));
            }
            else if (primitiveResult.bool_value_set > 0)
            {
                result = (TResult)Convert.ChangeType(primitiveResult.bool_value, typeof(TResult));
            }
            else if (primitiveResult.string_value != IntPtr.Zero)
            {
                var stringValue = Marshal.PtrToStringAuto(primitiveResult.string_value);

                result = (TResult)Convert.ChangeType(stringValue, typeof(TResult));
            }
            else if (primitiveResult.array_value != IntPtr.Zero)
            {
                var arrayStringValue = Marshal.PtrToStringAuto(primitiveResult.array_value);

                result = JsonSerializer.Deserialize<TResult>(arrayStringValue);
            }
            else if (primitiveResult.object_value != IntPtr.Zero)
            {
                var objectStringValue = Marshal.PtrToStringAuto(primitiveResult.object_value);

                result = JsonSerializer.Deserialize<TResult>(objectStringValue);
            }
            else
            {
                result = default(TResult);
            }

            Marshal.FreeHGlobal(funcNamePointer);
            Free(funcParams);
            Native.free_primitive_result(primitiveResultPointer);

            return result;
        }

        public void Dispose()
        {
            _handle.Dispose();

            _isDisposed = true;
        }

        private static string CreateManagedString(IntPtr stringPointer)
        {
            var result = Marshal.PtrToStringAuto(stringPointer);

            Native.free_string(stringPointer);

            return result;
        }

        private static void Free(Primitive[] primitives)
        {
            foreach (var p in primitives)
            {
                Free(p);
            }
        }

        private static void Free(Primitive primitive)
        {
            // Marshal.FreeHGlobal(primitive.string_value);
            // Marshal.FreeHGlobal(primitive.symbol_value);

            Console.WriteLine("Free!");
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
        internal static extern IntPtr call(JavaScriptEngineHandle handle, IntPtr func_name, Primitive[] parameters, int parameterCount);

        [DllImport(LIB_NAME)]
        internal static extern void free_string(IntPtr stringPointer);

        [DllImport(LIB_NAME)]
        internal static extern void free_primitive_result(IntPtr handle);
    }
}