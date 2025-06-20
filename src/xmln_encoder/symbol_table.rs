use std::collections::HashMap;

pub trait SymbolTable {
    fn encode(&mut self, symbol: char) -> Option<u16>;
    fn decode(&self, code: u16) -> Option<char>;
}

pub struct DynamicSymbolTable {
    encoder: HashMap<char, u16>,
    decoder: HashMap<u16, char>,
}

impl DynamicSymbolTable {
    pub fn new() -> Self {
        DynamicSymbolTable {
            encoder: HashMap::new(),
            decoder: HashMap::new(),
        }
    }
}

impl SymbolTable for DynamicSymbolTable {
    fn encode(&mut self, symbol: char) -> Option<u16> {
        if !self.encoder.contains_key(&symbol) {
            let code = self.encoder.len() as u16;
            self.encoder.insert(symbol, code);
            self.decoder.insert(code, symbol);
        }

        self.encoder.get(&symbol).copied()
    }

    fn decode(&self, code: u16) -> Option<char> {
        self.decoder.get(&code).copied()
    }
}
