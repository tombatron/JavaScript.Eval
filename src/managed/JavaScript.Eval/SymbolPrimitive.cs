namespace JavaScript.Eval
{
    public readonly struct SymbolPrimitive
    {
        public string Symbol { get; }

        public SymbolPrimitive(string symbol)
        {
            Symbol = symbol;
        }

        public static SymbolPrimitive Create(string symbol) => new SymbolPrimitive(symbol);
    }
}