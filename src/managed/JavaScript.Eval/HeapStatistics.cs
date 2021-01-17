using System.Runtime.InteropServices;

namespace JavaScript.Eval
{
    [StructLayout(LayoutKind.Sequential)]
    public struct HeapStatistics
    {
        public long total_heap_size { get; set; }

        public long total_heap_size_executable { get; set; }

        public long total_physical_size { get; set; }

        public long total_available_size { get; set; }

        public long used_heap_size { get; set; }

        public long heap_size_limit { get; set; }

        public long malloced_memory { get; set; }

        public long does_zap_garbage { get; set; }

        public long number_of_native_contexts { get; set; }

        public long number_of_detached_contexts { get; set; }

        public long peak_malloced_memory { get; set; }

        public long used_global_handles_size { get; set; }

        public long total_global_handles_size { get; set; }
    }
}