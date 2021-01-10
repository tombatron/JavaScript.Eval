using Xunit;

namespace JavaScript.Eval.Tests
{
    public class BasicTests
    {
        [Fact]
        public void ItCanExecuteSimpleScript()
        {
            using var engine = new JavaScriptEngine();

            var result = engine.Eval("1+1;");

            Assert.Equal("2", result);
        }

        [Fact]
        public void ItCanCallFunction()
        {
            using var engine = new JavaScriptEngine();

            engine.Eval("function add(x,y) { return x + y; }");

            var result = engine.Call("add", 1, 1);

            Assert.Equal("2", result);
        }
    }
}