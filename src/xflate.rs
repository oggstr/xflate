use core::panic;
use itertools::{Itertools, MultiPeek};
use std::{
    collections::HashSet,
    fmt::format,
    io::{BufReader, Read},
    str::Chars,
};
use xml::{ParserConfig, reader::XmlEvent};

use crate::{
    XmlBCompress, XmlBDecompressor, XmlBDeflateBackend, XmlNDynamicSymbolTable,
    XmlNDynamicTagTable, XmlNSymbolTable, XmlNTagTable, consume_until_space, decode_xmlb,
    decode_xmln, decode_xmls, encode_xmlb, encode_xmln, encode_xmls,
};

#[derive(Debug)]
pub enum XFlateError {
    PrePassError(String),
    XmlNError(String),
    XmlSError(String),
    XmlBError(String),
}

/// XFlate compression algorithm.
pub struct XFlate {
    /// Symbol table for XMLN encoding.
    /// This table translates characters to symbols.
    /// A symbol is simply a fixed size string composed of
    /// character c, where c ∈ {1, ..., 9}
    sym_table: XmlNDynamicSymbolTable,
    /// Tag table for XMLN encoding.
    /// This table translates XML tags and attribute names
    /// into string encodings. Ideally, frequently used
    /// tags/attributes should be assigned to shorter symbols.
    tag_table: XmlNDynamicTagTable,
    /// Backend compression algorithm.
    /// This is the last step in the compression process.
    backend: XmlBDeflateBackend,
    /// Configuration for XFlate compression.
    /// This includes options such as symbol size, backend compression options,
    /// and whether to include headers for symbols and tags.
    config: XFlateConfig,
}

impl XFlate {
    pub fn new(config: XFlateConfig) -> Self {
        XFlate {
            sym_table: XmlNDynamicSymbolTable::new(config.symbol_size),
            tag_table: XmlNDynamicTagTable::new(),
            backend: XmlBDeflateBackend::new(config.xmlb_opt),
            config,
        }
    }

    /// Run XFlate compression on the provided XML.
    ///
    /// # Arguments
    /// * `xml` - The XML data to compress, provided as a `Read` trait object.
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - Compressed XML as byte vector
    /// * `Err(XFlateError)` - Error if compression fails
    pub fn compress<D>(&mut self, xml: D) -> Result<Vec<u8>, XFlateError>
    where
        D: Read,
    {
        let xmln = encode_xmln(xml, &mut self.sym_table, &mut self.tag_table)?;
        let xmls = encode_xmls(&xmln)?;

        // Prepend symbol header
        let xmls = if self.config.add_symbol_header {
            let mut header = self.sym_table.to_header();
            header.extend(xmls);
            header
        } else {
            xmls
        };

        // Prepend tag header
        let xmls = if self.config.add_tag_header {
            let mut header = self.tag_table.to_header();
            header.extend(xmls);
            header
        } else {
            xmls
        };

        let xmlb = encode_xmlb(&xmls, &self.backend)?;

        Ok(xmlb)
    }

    pub fn decompress<D>(&mut self, binary: D) -> Result<String, XFlateError>
    where
        D: Read,
    {
        let xmlb: Vec<u8> = binary
            .bytes()
            .collect::<Result<_, _>>()
            .map_err(|e| XFlateError::XmlBError(format!("Failed to read bytes: {}", e)))?;

        // Ugly hack, should probably work.
        // The idea here is that our header information
        // must be first and a valid utf8 chunk.
        let xmls_raw_bytes = decode_xmlb(xmlb.as_slice(), &self.backend)?;
        let mut xmls_chunks = xmls_raw_bytes.utf8_chunks();

        let header_chunk = xmls_chunks.next().ok_or(XFlateError::XmlSError(
            "Unable to locate header".to_string(),
        ))?;

        let mut header_chars = header_chunk.valid().chars().multipeek();

        // Look for tag header
        self.tag_table = XFlate::parse_tag_header(&mut header_chars)?;

        // Look for symbol header
        self.sym_table = XFlate::parse_symbol_header(&mut header_chars)?;

        // Again real ugly, but collect everything that
        // wasn't parsed from the header as bytes again
        let mut xmls: Vec<u8> = header_chars.collect::<String>().into();
        xmls.extend(header_chunk.invalid());
        for chunk in xmls_chunks {
            xmls.extend(chunk.valid().as_bytes());
            xmls.extend(chunk.invalid());
        }

        let xmln = decode_xmls(xmls.as_slice())?;

        decode_xmln(xmln.as_str(), &mut self.sym_table, &mut self.tag_table)
    }

    /// Hacky parser for tag table header.
    ///
    /// TODO: move this into the tag table struct.
    fn parse_tag_header(
        chars: &mut MultiPeek<Chars<'_>>,
    ) -> Result<XmlNDynamicTagTable, XFlateError> {
        chars.next(); // 'E'
        chars.next(); // ' '

        let tag_count = consume_until_space(chars)
            .parse::<usize>()
            .map_err(|e| XFlateError::XmlSError(format!("Unable to parse tag count {}", e)))?;

        chars.next(); // ' '

        let mut tag_table = XmlNDynamicTagTable::new();
        for _ in 0..tag_count {
            let tag = consume_until_space(chars);
            chars.next(); // ' '
            if tag_table.encode(tag.as_str()).is_none() {
                return Err(XFlateError::XmlSError(format!(
                    "Unable to add tag {} to tag table",
                    tag
                )));
            }
        }

        Ok(tag_table)
    }

    /// Hacky parser for symbol table header.
    ///
    /// TODO: move this into the symbol table struct.
    fn parse_symbol_header(
        chars: &mut MultiPeek<Chars<'_>>,
    ) -> Result<XmlNDynamicSymbolTable, XFlateError> {
        chars.next(); // 'C'
        chars.next(); // ' '

        let code_size = consume_until_space(chars)
            .parse::<u8>()
            .map_err(|e| XFlateError::XmlSError(format!("Unable to prase the code size {}", e)))?;

        chars.next(); // ' '

        let symbol_count = consume_until_space(chars).parse::<usize>().map_err(|e| {
            XFlateError::XmlSError(format!("Unable to parse the symbol count {}", e))
        })?;

        chars.next(); // ' '

        let mut sym_table = XmlNDynamicSymbolTable::new(code_size);

        for _ in 0..symbol_count {
            let s = chars.next().ok_or(XFlateError::XmlSError(
                "Symbol header is missing symbols".to_string(),
            ))?;
            if sym_table.encode(s).is_none() {
                return Err(XFlateError::XmlSError(format!(
                    "Unable to add symbol {} to the symbol table",
                    s
                )));
            }
        }

        chars.next(); // ' '

        Ok(sym_table)
    }
}

/// Configuration struct for XFlate compression.
pub struct XFlateConfig {
    /// Size of symbols used when encoding XMLN.
    /// A symbol size of 1 can handle 9 unique symbols,
    /// a size of 2 can handle 99 unique symbols, and so on.
    pub symbol_size: u8,

    /// Backend compression options.
    /// The underlying backend algorithm determines how
    /// this flag is interpreted. Generally, one can either
    /// optimize for speed or for compression ratio.
    pub xmlb_opt: XmlBCompress,

    /// Tells XFlate to include a header with symbol information.
    /// This header can be used to decode the compressed XMLN data.
    ///
    /// Note that false is not supported, yet.
    pub add_symbol_header: bool,

    /// Tells XFlate to include a header with tag information.
    /// This header can be used to decode the compressed XMLN data.
    ///
    /// Note that false is not supported, yet.
    pub add_tag_header: bool,
}

impl XFlateConfig {
    pub fn from_xml<D>(xml: D) -> Result<Self, XFlateError>
    where
        D: Read,
    {
        let unique_symbols = scan(xml)?;
        let symbol_size = (unique_symbols as f64).log(10.0).ceil() as u8;

        Ok(XFlateConfig {
            symbol_size,
            ..Default::default()
        })
    }
}

impl Default for XFlateConfig {
    fn default() -> Self {
        XFlateConfig {
            symbol_size: 2,
            xmlb_opt: XmlBCompress::Best,
            add_symbol_header: true,
            add_tag_header: true,
        }
    }
}

/// Run a scan over the XML data.
/// This counts the number of unique symbols in the XML data.
///
/// # Warning
/// There is no guarantee that a scan performed on one document,
/// will yield a result that will work for another document.
fn scan<D>(xml: D) -> Result<u32, XFlateError>
where
    D: Read,
{
    let buf = BufReader::new(xml);
    let config = ParserConfig::new();
    let parser = config.create_reader(buf);

    let mut symbols: HashSet<char> = HashSet::new();
    for e in parser {
        match e {
            Ok(XmlEvent::Characters(data)) => {
                for c in data.chars() {
                    if !symbols.contains(&c) {
                        symbols.insert(c);
                    }
                }
            }
            Ok(XmlEvent::Whitespace(data)) => {
                for c in data.chars() {
                    if !symbols.contains(&c) {
                        symbols.insert(c);
                    }
                }
            }
            Ok(XmlEvent::StartElement {
                name: _,
                attributes,
                namespace: _,
            }) => {
                for attr in attributes {
                    for c in attr.value.chars() {
                        if !symbols.contains(&c) {
                            symbols.insert(c);
                        }
                    }
                }
            }
            Ok(XmlEvent::CData(_)) => {
                panic!("CData not implemented yet")
            }
            Ok(XmlEvent::Comment(_)) => {
                panic!("Comment not implemented yet")
            }
            _ => continue,
        };
    }

    symbols
        .len()
        .try_into()
        .map_err(|e| XFlateError::PrePassError(format!("Failed to count unique symbols: {}", e)))
}
