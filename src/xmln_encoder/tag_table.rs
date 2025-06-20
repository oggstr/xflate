use std::collections::HashMap;

pub trait TagTable {
    fn encode(&mut self, tag: &str) -> Option<u16>;
    fn decode(&self, code: u16) -> Option<&str>;
}

pub struct DynamicTagTable {
    encoder: HashMap<String, u16>,
    decoder: HashMap<u16, String>,
}

impl DynamicTagTable {
    pub fn new() -> Self {
        DynamicTagTable {
            encoder: HashMap::new(),
            decoder: HashMap::new(),
        }
    }
}

impl TagTable for DynamicTagTable {
    fn encode(&mut self, tag: &str) -> Option<u16> {
        if !self.encoder.contains_key(tag) {
            let code = self.encoder.len() as u16;
            self.encoder.insert(tag.to_string(), code);
            self.decoder.insert(code, tag.to_string());
        }

        self.encoder.get(tag).copied()
    }

    fn decode(&self, code: u16) -> Option<&str> {
        self.decoder.get(&code).map(|s| s.as_str())
    }
}
