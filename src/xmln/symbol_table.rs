use std::{collections::HashMap, fmt::Display};

use itertools::Itertools;

pub trait XmlNSymbolTable {
    /// Creates a new symbol table with
    /// the specified code size
    fn new(code_size: u8) -> Self;

    /// Returns number encoding of given symbol
    fn encode(&mut self, symbol: char) -> Option<&str>;

    /// Returns the symbol for a given encoded string
    fn decode(&self, code: &str) -> Option<char>;

    /// Returns the size of encodings
    fn code_size(&self) -> u8;

    /// Returns the number of symbols in the table
    fn symbol_count(&self) -> usize;

    /// Returns an iterator over all symbols in the table
    fn iter_symbols(&self) -> impl Iterator<Item = char>;

    /// Returns a header over all symbols. This header
    /// Can be parsed and used to reconstruct the symbol table.
    /// The header is defined as this (informal ebnf):
    /// <header>       := C <code_size> <symbol_count> <symbols>
    /// <code_size>    := [number]
    /// <symbol_count> := [number]
    /// <symbols>      := [char]<symbols>|[char]
    fn to_header(&self) -> Vec<u8> {
        let mut header = String::new();

        header.push('C');
        header.push(' ');
        header.push_str(self.code_size().to_string().as_ref());
        header.push(' ');
        header.push_str(self.symbol_count().to_string().as_ref());
        header.push(' ');
        for s in self.iter_symbols() {
            header.push(s);
        }
        header.push(' ');

        header.bytes().collect()
    }
}

pub struct XmlNDynamicSymbolTable {
    /// Maps symbols to encoded strings
    encoder: HashMap<char, String>,
    /// Maps encoded strings to symbols
    decoder: HashMap<String, char>,
    /// Size of encoded strings
    code_size: u8,
    /// Ordered list of all symbols
    symbols: Vec<char>,
}

impl XmlNDynamicSymbolTable {
    pub fn from_symbols(symbols: &[char]) -> Self {
        // Round up to the nearest power of 10
        let code_size = (symbols.len() as f64).log(10.0).ceil() as u8;

        let mut encoder = HashMap::new();
        let mut decoder = HashMap::new();

        for &symbol in symbols {
            let code = encoder.len() + 1;
            let code_str = to_code_str(code, code_size);

            encoder.insert(symbol, code_str.clone());
            decoder.insert(code_str, symbol);
        }

        return XmlNDynamicSymbolTable {
            encoder,
            decoder,
            code_size,
            symbols: symbols.to_vec(),
        };
    }

    fn get_next_code(&self) -> usize {
        // Ensure codes start from 1 to avoid
        // conflicts with 0 - which denotes closing tags
        self.encoder.len() + 1
    }
}

impl XmlNSymbolTable for XmlNDynamicSymbolTable {
    fn new(code_size: u8) -> Self {
        XmlNDynamicSymbolTable {
            encoder: HashMap::new(),
            decoder: HashMap::new(),
            code_size,
            symbols: Vec::new(),
        }
    }

    fn encode(&mut self, symbol: char) -> Option<&str> {
        if !self.encoder.contains_key(&symbol) {
            let code = self.get_next_code();

            if code >= usize::pow(10, self.code_size as u32) {
                panic!("Symbol table overflow: cannot encode {}", symbol);
            }

            let code_str = to_code_str(code, self.code_size);
            self.encoder.insert(symbol, code_str.clone());
            self.decoder.insert(code_str, symbol);
            self.symbols.push(symbol);
        }

        self.encoder.get(&symbol).map(|s| s.as_str())
    }

    fn decode(&self, code: &str) -> Option<char> {
        if code.len() != self.code_size as usize {
            panic!(
                "Invalid code length: expected size {}, got {}",
                self.code_size,
                code.len()
            );
        }

        self.decoder.get(code).copied()
    }

    fn code_size(&self) -> u8 {
        self.code_size
    }

    fn symbol_count(&self) -> usize {
        self.encoder.len()
    }

    fn iter_symbols(&self) -> impl Iterator<Item = char> {
        self.symbols.iter().copied()
    }
}

fn to_code_str(code: usize, width: u8) -> String {
    format!("{:0width$}", code, width = width as usize)
}

impl Display for XmlNDynamicSymbolTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "XmlNDynamicSymbolTable")?;
        write!(f, "\n  Symbol size: {}", self.code_size)?;
        write!(f, "\n  Symbols mappings: ")?;
        for (symbol, code) in &self.encoder {
            write!(f, "\n    {} -> {}", symbol.escape_debug(), code)?;
        }

        Ok(())
    }
}
