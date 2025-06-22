use std::{collections::HashMap, fmt::Display};

/// Tag table is used to encode and decode XML tags and
/// attributes to/from a numeric code.
///
/// Ideally, frequently used tags and attributes should
/// be assigned lower codes, while less frequently
/// used ones should be assigned higher codes.
///
/// Currently, codes are assigned using u16, meaning
/// it can hold up to 65536 unique tags and attributes.
pub trait XmlNTagTable {
    fn encode(&mut self, tag: &str) -> Option<u16>;
    fn decode(&self, code: u16) -> Option<&str>;
}

pub struct XmlNDynamicTagTable {
    encoder: HashMap<String, u16>,
    decoder: HashMap<u16, String>,
}

impl XmlNDynamicTagTable {
    pub fn new() -> Self {
        XmlNDynamicTagTable {
            encoder: HashMap::new(),
            decoder: HashMap::new(),
        }
    }
}

impl XmlNTagTable for XmlNDynamicTagTable {
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

impl Display for XmlNDynamicTagTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "XmlNTagTable")?;
        write!(f, "\n  Tag mappings:")?;
        for (tag, code) in &self.encoder {
            write!(f, "\n    {} -> {}", tag, code)?;
        }

        Ok(())
    }
}
