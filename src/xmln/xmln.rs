use itertools::Itertools;
use itertools::MultiPeek;
use std::io::BufReader;
use std::io::Read;
use std::str::Chars;
use xml::ParserConfig;
use xml::reader::XmlEvent;

use crate::XmlNSymbolTable;
use crate::XmlNTagTable;

pub type XmlN = String;

#[derive(Debug)]
pub enum XmlNError {
    EncodingError(String),
    DecodingError(String),
}

pub fn encode_xmln<D, S, T>(
    data: D,
    sym_table: &mut S,
    tag_table: &mut T,
) -> Result<XmlN, XmlNError>
where
    D: Read,
    S: XmlNSymbolTable,
    T: XmlNTagTable,
{
    let buf = BufReader::new(data);
    let config = ParserConfig::new();
    let parser = config.create_reader(buf);

    let mut xmln = XmlN::new();
    let mut translate = String::new();

    for e in parser {
        match e {
            Ok(XmlEvent::StartDocument {
                version,
                encoding,
                standalone,
            }) => {
                println!(
                    "Start Document: version={}, encoding={}, standalone={:?}",
                    version, encoding, standalone
                );
            }

            Ok(XmlEvent::EndDocument) => {
                println!("End Document");
            }

            Ok(XmlEvent::StartElement {
                name: tag,
                attributes,
                namespace: _,
            }) => {
                put_elem_start_tag(&mut translate, tag.to_string().as_str(), tag_table)?;

                for attr in attributes {
                    let attr_name = match attr.name.prefix {
                        Some(prefix) => prefix + ":" + &attr.name.local_name,
                        None => attr.name.local_name.clone(),
                    };

                    put_attr_tag(&mut translate, attr_name.as_str(), tag_table)?;
                    put_symbols(&mut translate, attr.value, sym_table)?;
                }
            }

            Ok(XmlEvent::EndElement { name: _ }) => {
                translate.push(' ');
                translate.push('0');
            }

            Ok(XmlEvent::Characters(data)) => {
                put_symbols(&mut translate, data, sym_table)?;
            }

            Ok(XmlEvent::Whitespace(data)) => {
                put_symbols(&mut translate, data, sym_table)?;
            }

            Ok(XmlEvent::CData(data)) => {
                println!("CData: {}", data);
            }

            Ok(XmlEvent::ProcessingInstruction { name, data }) => {
                println!("Processing Instruction: name={}, data={:?}", name, data);
            }

            Ok(XmlEvent::Comment(data)) => {
                println!("Comment: {}", data);
            }

            Err(e) => {
                panic!("Error parsing XML: {}", e);
            }
        }

        xmln.push_str(&translate);
        translate.clear();
    }

    // Clear any leading whitespace
    trim_start_in_place(&mut xmln);

    Ok(xmln)
}

fn put_symbols<S>(translate: &mut String, token: String, sym_table: &mut S) -> Result<(), XmlNError>
where
    S: XmlNSymbolTable,
{
    translate.push(' ');

    for c in token.chars() {
        let enc = sym_table
            .encode(c)
            .ok_or_else(|| XmlNError::EncodingError(format!("Failed to encode symbol: {}", c)))?;
        translate.push_str(enc);
    }
    Ok(())
}

fn put_elem_start_tag<T>(
    translate: &mut String,
    tag: &str,
    tag_table: &mut T,
) -> Result<(), XmlNError>
where
    T: XmlNTagTable,
{
    translate.push(' ');

    let enc = tag_table
        .encode(tag)
        .ok_or_else(|| XmlNError::EncodingError(format!("Failed to encode tag: {}", tag)))?;

    translate.push('T');
    translate.push_str(&enc.to_string());

    Ok(())
}

fn put_attr_tag<T>(
    translate: &mut String,
    attr_name: &str,
    tag_table: &mut T,
) -> Result<(), XmlNError>
where
    T: XmlNTagTable,
{
    translate.push(' ');

    let enc = tag_table.encode(attr_name).ok_or_else(|| {
        XmlNError::EncodingError(format!("Failed to encode attribute name: {}", attr_name))
    })?;

    translate.push('A');
    translate.push_str(&enc.to_string());

    Ok(())
}

fn trim_start_in_place(s: &mut String) {
    let trimmed_start = s.trim_start();
    let chars_to_remove = s.len() - trimmed_start.len();
    s.drain(..chars_to_remove);
}

pub fn decode_xmln<S, T>(
    xmln: &str,
    sym_table: &mut S,
    tag_table: &mut T,
) -> Result<String, XmlNError>
where
    S: XmlNSymbolTable,
    T: XmlNTagTable,
{
    let mut decoded = String::new();
    let mut chars = xmln.chars().multipeek();

    // Store opened tags to handle nested strctures
    let mut tag_stack = Vec::new();

    while let Some(token) = chars.peek().copied() {
        let token_next = chars.peek().copied();

        match (token, token_next) {
            (' ', _) => {
                chars.next(); // Consume whitespace
            }
            // Opening tag
            ('T', _) => {
                chars.next(); // Consume 'T'

                let code_str = consume_until_whitespace(&mut chars);
                if code_str.is_empty() {
                    return Err(XmlNError::DecodingError("Empty tag code".to_string()));
                }

                let tag_code: u16 = code_str.parse().map_err(|_| {
                    XmlNError::DecodingError(format!("Invalid tag code: {}", code_str))
                })?;

                let tag = tag_table.decode(tag_code).ok_or_else(|| {
                    XmlNError::DecodingError(format!("Unknown tag code: {}", tag_code))
                })?;

                decoded.push_str("<");
                decoded.push_str(tag);

                tag_stack.push(tag.to_string());

                chars.peek(); // Whitespace
                // Check if this element has attributes
                if chars.peek().cloned() != Some('A') {
                    decoded.push('>');
                }
            }

            // Attribute
            ('A', _) => {
                chars.next(); // Consume 'A'

                let attr_code_str = consume_until_whitespace(&mut chars);
                if attr_code_str.is_empty() {
                    return Err(XmlNError::DecodingError("Empty attribute code".to_string()));
                }

                let attr_code: u16 = attr_code_str.parse().map_err(|_| {
                    XmlNError::DecodingError(format!("Invalid attribute code: {}", attr_code_str))
                })?;

                let attr_name = tag_table.decode(attr_code).ok_or_else(|| {
                    XmlNError::DecodingError(format!("Unknown attribute code: {}", attr_code))
                })?;

                chars.peek(); // Whitespace
                // Check if there exists an attribute value
                let attr_val = match (chars.peek().cloned(), chars.peek().cloned()) {
                    (Some('0'..='9'), Some('0'..='9')) => {
                        chars.next(); // Consume whitespace
                        decode_text(&mut chars, sym_table)?
                    }
                    _ => "".to_string(),
                };

                decoded.push(' ');
                decoded.push_str(&attr_name);
                decoded.push_str("=\"");
                decoded.push_str(&attr_val);
                decoded.push('\"');

                chars.peek(); // Whitespace
                // Check if this is the last
                // attribute for this element
                if chars.peek().cloned() != Some('A') {
                    decoded.push('>');
                }
            }

            // Closing tag
            ('0', Some(' ') | None) => {
                chars.next(); // Consume '0'

                if let Some(tag) = tag_stack.pop() {
                    decoded.push_str("</");
                    decoded.push_str(&tag);
                    decoded.push('>');
                } else {
                    return Err(XmlNError::DecodingError(
                        "Unmatched closing tag found".to_string(),
                    ));
                }
            }

            // Text content
            ('0'..='9', Some('0'..='9')) => {
                let text = decode_text(&mut chars, sym_table)?;
                decoded.push_str(&text);
            }

            // Unexpected character
            _ => {
                return Err(XmlNError::DecodingError(format!(
                    "Unexpected character in XMLN: {}",
                    token,
                )));
            }
        }

        chars.reset_peek();
    }

    Ok(decoded)
}

fn decode_text<S>(chars: &mut MultiPeek<Chars<'_>>, sym_table: &S) -> Result<String, XmlNError>
where
    S: XmlNSymbolTable,
{
    chars.reset_peek();
    let code_strings = consume_until_whitespace(chars)
        .chars()
        .collect::<Vec<char>>()
        .chunks(sym_table.get_symbol_size() as usize)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<String>>();

    let mut result = String::new();
    for code_str in code_strings {
        match sym_table.decode(&code_str) {
            Some(symbol) => result.push(symbol),
            None => {
                return Err(XmlNError::DecodingError(format!(
                    "Unknown symbol code: {}",
                    code_str
                )));
            }
        }
    }

    Ok(result)
}

fn consume_until_whitespace(chars: &mut MultiPeek<Chars<'_>>) -> String {
    let mut result: String = String::new();
    while let Some(c) = chars.peek().cloned() {
        if c.is_whitespace() {
            // Avoid side effects
            chars.reset_peek();
            break;
        }
        chars.next(); // Consume the character
        result.push(c);
    }

    result
}
