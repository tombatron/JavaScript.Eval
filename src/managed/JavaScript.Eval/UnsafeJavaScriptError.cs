using System;
using System.Runtime.InteropServices;

namespace JavaScript.Eval
{
    [StructLayout(LayoutKind.Sequential)]
    public struct UnsafeJavaScriptError
    {
        public IntPtr exception {get;set;}
        public IntPtr stack_trace {get;set;}
    }
}