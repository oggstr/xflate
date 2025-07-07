use std::{collections::HashMap, fmt::Display};

use itertools::Itertools;

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
    /// Encode a tag a tag
    fn encode(&mut self, tag: &str) -> Option<u16>;

    /// Decode a tag
    fn decode(&self, code: u16) -> Option<&str>;

    /// Iterate over all tags currently in table
    fn iter_tags(&self) -> impl Iterator<Item = &str>;

    /// Get the number of tags in the table
    fn tag_count(&self) -> usize;

    /// Returns a header over all tags. This header
    /// Can be parsed and used to reconstruct the tag table.
    /// The header is defined as this (informal ebnf):
    /// <header>    := E <tag_count> <tags>
    /// <tag_count> := [number]
    /// <tags>      := <tag> <tags>|<tag>
    /// <tag>       := [str]
    /// E <tag_count> <tag...>
    fn to_header(&self) -> Vec<u8> {
        let mut header = String::new();
        header.push('E');
        header.push(' ');
        header.push_str(self.tag_count().to_string().as_str());
        header.push(' ');

        for tag in self.iter_tags() {
            header.push_str(tag);
            header.push(' ');
        }

        header.bytes().collect()
    }
}

pub struct XmlNDynamicTagTable {
    encoder: HashMap<String, u16>,
    decoder: HashMap<u16, String>,
    tags: Vec<String>,
}

impl XmlNDynamicTagTable {
    pub fn new() -> Self {
        XmlNDynamicTagTable {
            encoder: HashMap::new(),
            decoder: HashMap::new(),
            tags: Vec::new(),
        }
    }
}

impl XmlNTagTable for XmlNDynamicTagTable {
    fn encode(&mut self, tag: &str) -> Option<u16> {
        if !self.encoder.contains_key(tag) {
            let code = self.encoder.len() as u16;
            self.encoder.insert(tag.to_string(), code);
            self.decoder.insert(code, tag.to_string());
            self.tags.push(tag.to_string());
        }

        self.encoder.get(tag).copied()
    }

    fn decode(&self, code: u16) -> Option<&str> {
        self.decoder.get(&code).map(|s| s.as_str())
    }

    fn tag_count(&self) -> usize {
        self.encoder.len()
    }

    fn iter_tags(&self) -> impl Iterator<Item = &str> {
        self.tags.iter().map(|s| s.as_str())
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
