use std::{collections::HashMap, fmt::Display};

pub trait XmlNSymbolTable {
    fn encode(&mut self, symbol: char) -> Option<&str>;
    fn decode(&self, code: &str) -> Option<char>;
    fn get_symbol_size(&self) -> u8;
}

pub struct XmlNDynamicSymbolTable {
    /// Maps symbols to encoded strings
    encoder: HashMap<char, String>,
    /// Maps encoded strings to symbols
    decoder: HashMap<String, char>,
    /// Size of symbols
    symbol_size: u8,
    /// Ordered list of all symbols
    symbols: Vec<char>,
}

impl XmlNDynamicSymbolTable {
    pub fn new(symbol_size: u8) -> Self {
        XmlNDynamicSymbolTable {
            encoder: HashMap::new(),
            decoder: HashMap::new(),
            symbol_size,
            symbols: Vec::new(),
        }
    }

    pub fn from_symbols(symbols: &[char]) -> Self {
        // Round up to the nearest power of 10
        let symbol_size = (symbols.len() as f64).log(10.0).ceil() as u8;

        let mut encoder = HashMap::new();
        let mut decoder = HashMap::new();

        for &symbol in symbols {
            let code = encoder.len();
            let code_str = to_code_str(code, symbol_size);

            encoder.insert(symbol, code_str.clone());
            decoder.insert(code_str, symbol);
        }

        return XmlNDynamicSymbolTable {
            encoder,
            decoder,
            symbol_size,
            symbols: symbols.to_vec(),
        };
    }
}

impl XmlNSymbolTable for XmlNDynamicSymbolTable {
    fn encode(&mut self, symbol: char) -> Option<&str> {
        if !self.encoder.contains_key(&symbol) {
            let code = self.encoder.len();

            if code >= usize::pow(10, self.symbol_size as u32) {
                panic!("Symbol table overflow: cannot encode {}", symbol);
            }

            let code_str = to_code_str(code, self.symbol_size);
            self.encoder.insert(symbol, code_str.clone());
            self.decoder.insert(code_str, symbol);
            self.symbols.push(symbol);
        }

        self.encoder.get(&symbol).map(|s| s.as_str())
    }

    fn decode(&self, code: &str) -> Option<char> {
        if code.len() != self.symbol_size as usize {
            panic!(
                "Invalid code length: expected size {}, got {}",
                self.symbol_size,
                code.len()
            );
        }

        self.decoder.get(code).copied()
    }

    fn get_symbol_size(&self) -> u8 {
        self.symbol_size
    }
}

fn to_code_str(code: usize, width: u8) -> String {
    format!("{:0width$}", code, width = width as usize)
}

impl Display for XmlNDynamicSymbolTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "XmlNDynamicSymbolTable")?;
        write!(f, "\n  Symbol size: {}", self.symbol_size)?;
        write!(f, "\n  Symbols mappings: ")?;
        for (symbol, code) in &self.encoder {
            write!(f, "\n    {} -> {}", symbol.escape_debug(), code)?;
        }

        Ok(())
    }
}
