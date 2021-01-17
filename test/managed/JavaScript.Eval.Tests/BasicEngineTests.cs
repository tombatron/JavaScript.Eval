using JavaScript.Eval.Exceptions;
using System.Collections.Generic;
using Xunit;

namespace JavaScript.Eval.Tests
{
    public class BasicTests
    {
        [Fact]
        public void ItCanExecuteSimpleScript()
        {
            using var engine = new JavaScriptEngine();

            var result = engine.Eval<int>("1+1;");

            Assert.Equal(2, result);
        }

        [Fact]
        public void ItCanCallFunction()
        {
            using var engine = new JavaScriptEngine();

            engine.Eval("function helloWorld() { return \"Hello World!\" }");

            var result = engine.Call<string>("helloWorld");

            Assert.Equal("Hello World!", result);
        }

        [Fact]
        public void ItWillThrowJavaScriptException()
        {
            using var engine = new JavaScriptEngine();

            engine.Eval("function thisShouldBreak() { return foo.bar.baz; }");

            var exception = Assert.Throws<JavaScriptException>(() =>
            {
                engine.Eval<string>("thisShouldBreak();");
            });

            Assert.Equal("ReferenceError: foo is not defined", exception.Message);
            Assert.Equal("ReferenceError: foo is not defined\n    at thisShouldBreak (<anonymous>:1:30)\n    at <anonymous>:1:1", exception.StackTrace);
        }

        [Fact]
        public void ItWillThrowJavaScriptExceptionOnBadFunctionCall()
        {
            using var engine = new JavaScriptEngine();

            engine.Eval("function thisShouldBreak() { return foo.bar.baz; }");

            var exception = Assert.Throws<JavaScriptException>(() =>
            {
                engine.Call<string>("thisShouldBreak");
            });

            Assert.Equal("ReferenceError: foo is not defined", exception.Message);
            Assert.Equal("ReferenceError: foo is not defined\n    at thisShouldBreak (<anonymous>:1:30)", exception.StackTrace);
        }

        [Fact]
        public void ItCanCallFunctionWithParameters()
        {
            using var engine = new JavaScriptEngine();

            engine.Eval("function add(x,y) { return x + y; }");

            var result = engine.Call<int>("add", 1, 2);

            Assert.Equal(3, result);
        }

        [Fact]
        public void ItCanCallFunctionWithObjectParameter()
        {
            using var engine = new JavaScriptEngine();

            engine.Eval("function echo(obj) { return obj.Hello; }");

            var result = engine.Call<string>("echo", Primitive.FromObject(new Message { Hello = "World!" }));

            Assert.Equal("World!", result);
        }

        [Fact]
        public void ItWillThrowExceptionIfYouCallNonExistentFunction()
        {
            using var engine = new JavaScriptEngine();

            var exception = Assert.Throws<JavaScriptException>(() =>
            {
                engine.Call<string>("thisDoesntEvenExist");
            });

            Assert.Equal("Couldn't resolve function `thisDoesntEvenExist`, V8 returned: 'undefined'", exception.Message);
            Assert.Empty(exception.StackTrace);
        }

        [Fact]
        public void ItCanHandleArrays()
        {
            using var engine = new JavaScriptEngine();

            engine.Eval("function getArray() { return [1,2,3];}");

            var result = engine.Call<IEnumerable<int>>("getArray");

            Assert.Equal(new[] { 1, 2, 3 }, result);
        }

        [Fact]
        public void ItCanGetObject()
        {
            using var engine = new JavaScriptEngine();

            engine.Eval("function getObject() { return {\"Hello\":\"World!\"};}");

            var result = engine.Call<Message>("getObject");

            Assert.Equal("World!", result.Hello);
        }

        [Fact]
        public void ItCanGetHeapStatistics()
        {
            using var engine = new JavaScriptEngine();

            var heapStatistics = engine.GetHeapStatistics();

            Assert.True(heapStatistics.used_heap_size > 0);
            Assert.True(heapStatistics.total_heap_size > 0);
            Assert.True(heapStatistics.total_heap_size > heapStatistics.used_heap_size);
            Assert.True(heapStatistics.heap_size_limit > 0);
            Assert.True(heapStatistics.heap_size_limit > heapStatistics.total_heap_size);
            Assert.True(heapStatistics.malloced_memory > 0);
            Assert.True(heapStatistics.peak_malloced_memory > 0);
            Assert.Equal(0, heapStatistics.used_global_handles_size);
            Assert.Equal(0, heapStatistics.total_global_handles_size);
            Assert.Equal(1, heapStatistics.number_of_native_contexts);
        }

        public class Message
        {
            public string Hello { get; set; }
        }
    }
}