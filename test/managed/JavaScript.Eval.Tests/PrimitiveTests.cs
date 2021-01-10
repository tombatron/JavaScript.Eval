using System;
using Xunit;

namespace JavaScript.Eval.Tests
{
    public class PrimitiveTests
    {
        public class ItCanImplicitlyConvert
        {
            [Fact]
            public void FromSByte()
            {
                Primitive p = (sbyte)1;

                Assert.Equal(1, p.number_value_set);
                Assert.Equal(1, p.number_value);
            }

            [Fact]
            public void FromByte()
            {
                Primitive p = (byte)1;

                Assert.Equal(1, p.number_value_set);
                Assert.Equal(1, p.number_value);
            }

            [Fact]
            public void FromShort()
            {
                Primitive p = (short)1;

                Assert.Equal(1, p.number_value_set);
                Assert.Equal(1, p.number_value);
            }

            [Fact]
            public void FromUShort()
            {
                Primitive p = (ushort)1;

                Assert.Equal(1, p.number_value_set);
                Assert.Equal(1, p.number_value);
            }

            [Fact]
            public void FromInt()
            {
                Primitive p = 1;

                Assert.Equal(1, p.number_value_set);
                Assert.Equal(1, p.number_value);
            }

            [Fact]
            public void FromUInt()
            {
                Primitive p = (uint)1;

                Assert.Equal(1, p.number_value_set);
                Assert.Equal(1, p.number_value);
            }

            [Fact]
            public void FromLong()
            {
                Primitive p = 1L;

                Assert.Equal(1, p.bigint_value_set);
                Assert.Equal(1, p.bigint_value);
            }

            [Fact]
            public void FromBool()
            {
                Primitive p = true;

                Assert.Equal(1, p.bool_value_set);
                Assert.Equal(1, p.bool_value);
            }

            [Fact]
            public void FromString()
            {
                Primitive p = "hello world";

                Assert.NotEqual(IntPtr.Zero, p.string_value);
            }

            [Fact]
            public void FromSymbolPrimitive()
            {
                Primitive p = SymbolPrimitive.Create("whoa");

                Assert.NotEqual(IntPtr.Zero, p.symbol_value);
            }
        }
    }
}