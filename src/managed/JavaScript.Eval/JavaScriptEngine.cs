using System;
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

        public string Call(string funcName, params Primitive[] funcParams)
        {
            var funcNamePointer = Marshal.StringToHGlobalAuto(funcName);

            var resultPointer = Native.call(_handle, funcNamePointer, funcParams, funcParams.Length);

            Marshal.FreeHGlobal(funcNamePointer);

            Free(funcParams);

            return CreateManagedString(resultPointer);
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
    }
}