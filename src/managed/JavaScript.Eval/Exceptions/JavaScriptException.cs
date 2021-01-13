using System;
using System.Runtime.InteropServices;

namespace JavaScript.Eval.Exceptions
{
    public class JavaScriptException : Exception
    {
        private readonly string _stackTrace;

        public JavaScriptException() { }

        public JavaScriptException(UnsafeJavaScriptError javaScriptError) :
            base(Marshal.PtrToStringAuto(javaScriptError.exception))
        {
            _stackTrace = Marshal.PtrToStringAuto(javaScriptError.stack_trace);
        }

        public JavaScriptException(string message) :
            base(message)
        { }

        public JavaScriptException(string message, Exception inner) :
            base(message, inner)
        { }

        public override string StackTrace => _stackTrace;
    }
}