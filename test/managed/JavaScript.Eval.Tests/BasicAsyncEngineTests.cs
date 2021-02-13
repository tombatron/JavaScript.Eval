using JavaScript.Eval.Exceptions;
using System.Collections.Generic;
using System.Threading.Tasks;
using Xunit;

namespace JavaScript.Eval.Tests
{
    public class BasicAsyncEngineTests
    {
        [Fact]
        public async Task ItCanExecuteSimpleScript_Async()
        {
            using var engine = new JavaScriptEngine();

            var result = await engine.EvalAsync<int>("1+1;");

            Assert.Equal(2, result);
        }

        [Fact]
        public async Task ItCanExecuteScriptWithNoResult_Async()
        {
            using var engine = new JavaScriptEngine();

            await engine.EvalAsync("function throwMessage(message) { throw message; }");

            var exception = await Assert.ThrowsAsync<JavaScriptException>(async () => {
                await engine.EvalAsync("throwMessage(\"Hello from the error!\");");
            });

            Assert.Equal("Hello from the error!", exception.Message);
        }

        [Fact]
        public async Task ItCanExecuteASimpleScriptWithUnicode_Async()
        {
            using var engine = new JavaScriptEngine();

            var result = await engine.EvalAsync<string>("\"😏\";");

            Assert.Equal("😏", result);
        }

        [Fact]
        public async Task ItCanCallFunction_Async()
        {
            using var engine = new JavaScriptEngine();

            engine.Eval("function helloWorld() { return \"Hello World!\"; }");

            var result = await engine.CallAsync<string>("helloWorld");

            Assert.Equal("Hello World!", result);
        }

        [Fact]
        public async Task ItCanCallAFunctionThatReturnsUnicode_Async()
        {
            using var engine = new JavaScriptEngine();

            await engine.EvalAsync("function helloWorld() { return \"🐱‍👤\"; }");

            var result = await engine.CallAsync<string>("helloWorld");

            Assert.Equal("🐱‍👤", result);
        }

        [Fact]
        public async Task ItWillThrowJavaScriptException_Async()
        {
            using var engine = new JavaScriptEngine();

            await engine.EvalAsync("function thisShouldBreak() { return foo.bar.baz; }");

            var exception = await Assert.ThrowsAsync<JavaScriptException>(async () =>
            {
                await engine.EvalAsync<string>("thisShouldBreak();");
            });

            Assert.Equal("ReferenceError: foo is not defined", exception.Message);
            Assert.Equal("ReferenceError: foo is not defined\n    at thisShouldBreak (<anonymous>:1:30)\n    at <anonymous>:1:1", exception.StackTrace);
        }

        [Fact]
        public async Task ItWillThrowJavaScriptExceptionOnBadFunctionCall_Async()
        {
            using var engine = new JavaScriptEngine();

            await engine.EvalAsync("function thisShouldBreak() { return foo.bar.baz; }");

            var exception = await Assert.ThrowsAsync<JavaScriptException>(async () =>
            {
                await engine.CallAsync<string>("thisShouldBreak");
            });

            Assert.Equal("ReferenceError: foo is not defined", exception.Message);
            Assert.Equal("ReferenceError: foo is not defined\n    at thisShouldBreak (<anonymous>:1:30)", exception.StackTrace);
        }    

        [Fact]
        public async Task ItCanCallFunctionWithParameters_Async()
        {
            using var engine = new JavaScriptEngine();

            await engine.EvalAsync("function add(x,y) { return x + y; }");

            var result = await engine.CallAsync<int>("add", 1, 2);

            Assert.Equal(3, result);
        }

        [Fact]
        public async Task ItCanCallFunctionWithObjectParameter_Async()
        {
            using var engine = new JavaScriptEngine();

            await engine.EvalAsync("function echo(obj) { return obj.Hello; }");

            var result = await engine.CallAsync<string>("echo", Primitive.FromObject(new Message { Hello = "World!" }));

            Assert.Equal("World!", result);
        }

        [Fact]
        public async Task ItWillThrowExceptionIfYouCallNonExistentFunction_Async()
        {
            using var engine = new JavaScriptEngine();

            var exception = await Assert.ThrowsAsync<JavaScriptException>(async () =>
            {
                await engine.CallAsync<string>("thisDoesntEvenExist");
            });

            Assert.Equal("Couldn't resolve function `thisDoesntEvenExist`, V8 returned: 'undefined'", exception.Message);
            Assert.Empty(exception.StackTrace);
        }

        [Fact]
        public async Task ItCanHandleArrays_Async()
        {
            using var engine = new JavaScriptEngine();

            await engine.EvalAsync("function getArray() { return [1,2,3];}");

            var result = await engine.CallAsync<IEnumerable<int>>("getArray");

            Assert.Equal(new[] { 1, 2, 3 }, result);
        }

        [Fact]
        public async Task ItCanGetObject_Async()
        {
            using var engine = new JavaScriptEngine();

            await engine.EvalAsync("function getObject() { return {\"Hello\":\"World!\"};}");

            var result = await engine.CallAsync<Message>("getObject");

            Assert.Equal("World!", result.Hello);
        }

        [Fact]
        public async Task ItCanGetHeapStatistics_Async()
        {
            using var engine = new JavaScriptEngine();

            var heapStatistics = await engine.GetHeapStatisticsAsync();

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