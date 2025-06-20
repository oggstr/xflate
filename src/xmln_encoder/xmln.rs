use std::io::BufReader;
use std::io::Read;
use xml::reader::EventReader;
use xml::reader::XmlEvent;

use crate::SymbolTable;
use crate::TagTable;

pub fn encode_xmln<D: Read, S: SymbolTable, T: TagTable>(
    data: D,
    sym_table: &mut S,
    tag_table: &mut T,
) -> String {
    let buf = BufReader::new(data);
    let parser = EventReader::new(buf);

    let mut xmln = String::new();
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
                put_elem_start_tag(&mut translate, tag.to_string().as_str(), tag_table);
                translate.push(' ');

                for attr in attributes {
                    put_attr_tag(&mut translate, attr.name.to_string().as_str(), tag_table);
                    translate.push(' ');
                    put_symbols(&mut translate, attr.value, sym_table);
                }
            }

            Ok(XmlEvent::EndElement { name: _ }) => {
                translate.push('0');
            }

            Ok(XmlEvent::Characters(data)) => {
                put_symbols(&mut translate, data, sym_table);
            }

            Ok(XmlEvent::Whitespace(data)) => {
                put_symbols(&mut translate, data, sym_table);
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

        xmln.push(' ');
        xmln.push_str(&translate);
    }

    return xmln;
}

fn put_symbols<S: SymbolTable>(translate: &mut String, token: String, sym_table: &mut S) {
    for c in token.chars() {
        translate.push_str(&format!(
            "{}",
            sym_table.encode(c).unwrap_or_else(|| {
                panic!("Failed to encode symbol: {}", c);
            })
        ));
    }
}

fn put_elem_start_tag<T: TagTable>(translate: &mut String, tag: &str, tag_table: &mut T) {
    translate.push_str(&format!(
        "T{}",
        tag_table.encode(tag).unwrap_or_else(|| {
            panic!("Failed to encode tag: {}", tag);
        })
    ));
}

fn put_attr_tag<T: TagTable>(translate: &mut String, attr_name: &str, tag_table: &mut T) {
    translate.push_str(&format!(
        "A{}",
        tag_table.encode(attr_name).unwrap_or_else(|| {
            panic!("Failed to encode attribute name: {}", attr_name);
        }),
    ));
}
