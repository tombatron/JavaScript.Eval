using System;
using System.Runtime.InteropServices;

namespace JavaScript.Eval
{
    [StructLayout(LayoutKind.Sequential)]
    public struct Primitive
    {
        public double number_value { get; set; }
        public byte number_value_set { get; set; }
        public long bigint_value { get; set; }
        public byte bigint_value_set { get; set; }
        public byte bool_value { get; set; }
        public byte bool_value_set { get; set; }
        public IntPtr string_value { get; set; }
        public IntPtr symbol_value { get; set; }
    }
}