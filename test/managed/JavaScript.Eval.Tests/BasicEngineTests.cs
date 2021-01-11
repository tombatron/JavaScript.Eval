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

        [Fact]
        public void ItCanHandleArrays()
        {
            using var engine = new JavaScriptEngine();

            engine.Eval("function getArray(foo) { return [1,2,3];}");

            var result = engine.Call("getArray", 123);

            Assert.Equal("[1,2,3]", result);
        }

        [Fact]
        public void ItCanGetObject()
        {
            using var engine = new JavaScriptEngine();

            engine.Eval("function getObject(foo) { return {\"hello\":\"world\"};}");

            var result = engine.Call("getObject", 123);

            Assert.Equal("{}", result);
        }
    }
}