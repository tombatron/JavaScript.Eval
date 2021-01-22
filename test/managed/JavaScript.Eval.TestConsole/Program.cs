using System;
using System.Text;

namespace JavaScript.Eval.TestConsole
{
    class Program
    {
        static void Main(string[] args)
        {
            var giantString = RandomString(50_000);
            {
                using var engine = new JavaScriptEngine();

                engine.Eval("function echo (value) { return value; }");

                for (var i = 0; i < 10_000; i++)
                {
                    var result = engine.Call<string>("echo", giantString);

                    if (i == 5_000)
                    {
                        var stats = engine.GetHeapStatistics();

                        Console.WriteLine($"Heap Stats:");
                    }

                    Console.WriteLine($"Iteration: {i}, Size: {result.Length}");
                }
            }

            {
                using var engine = new JavaScriptEngine();

                engine.Eval("function echo (value) { return value; }");

                for (var i = 0; i < 10_000; i++)
                {
                    var result = engine.Call<string>("echo", giantString);

                    if (i == 5_000)
                    {
                        var stats = engine.GetHeapStatistics();

                        Console.WriteLine($"Heap Stats:");
                    }

                    Console.WriteLine($"Iteration: {i}, Size: {result.Length}");
                }
            }

            Console.ReadLine();
        }

        private static readonly Random _random = new Random();

        public static string RandomString(int size, bool lowerCase = false)
        {
            var builder = new StringBuilder(size);

            // Unicode/ASCII Letters are divided into two blocks
            // (Letters 65–90 / 97–122):
            // The first group containing the uppercase letters and
            // the second group containing the lowercase.  

            // char is a single Unicode character  
            char offset = lowerCase ? 'a' : 'A';
            const int lettersOffset = 26; // A...Z or a..z: length=26  

            for (var i = 0; i < size; i++)
            {
                var @char = (char)_random.Next(offset, offset + lettersOffset);
                builder.Append(@char);
            }

            return lowerCase ? builder.ToString().ToLower() : builder.ToString();
        }
    }
}
