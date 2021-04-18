using System;

namespace JavaScript.Eval.Exceptions
{
    public class JavaScriptEngineDisposedException : Exception
    {
        public JavaScriptEngineDisposedException() { }

        public JavaScriptEngineDisposedException(string message) : base(message) { }

        public JavaScriptEngineDisposedException(string message, Exception inner) : base(message, inner) { }
    }
}