using System;
using System.Runtime.InteropServices;
using System.Text.Json;

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
        public IntPtr object_value {get;set;}

        public static implicit operator Primitive(sbyte b) => new Primitive { number_value = b, number_value_set = 1 };

        public static implicit operator Primitive(byte b) => new Primitive { number_value = b, number_value_set = 1 };

        public static implicit operator Primitive(short s) => new Primitive { number_value = s, number_value_set = 1 };

        public static implicit operator Primitive(ushort u) => new Primitive { number_value = u, number_value_set = 1 };

        public static implicit operator Primitive(int i) => new Primitive { number_value = i, number_value_set = 1 };

        public static implicit operator Primitive(uint u) => new Primitive { number_value = u, number_value_set = 1 };

        public static implicit operator Primitive(long l) => new Primitive { bigint_value = l, bigint_value_set = 1 };

        // We're not messing with `ulong` because it'd probably just overflow anyway...

        public static implicit operator Primitive(bool b) => new Primitive { bool_value = (byte)(b ? 1 : 0), bool_value_set = 1 };

        public static implicit operator Primitive(string s) => new Primitive { string_value = Marshal.StringToCoTaskMemUTF8(s) };

        public static implicit operator Primitive(SymbolPrimitive symbolPrimitive) => new Primitive { symbol_value = Marshal.StringToCoTaskMemUTF8(symbolPrimitive.Symbol) };

        public static Primitive FromObject<T>(T o)
        {
            var serializedObject = JsonSerializer.Serialize<T>(o);
            var serializedObjectPointer = Marshal.StringToCoTaskMemUTF8(serializedObject);

            return new Primitive
            {
                object_value = serializedObjectPointer
            };
        }
    }
}