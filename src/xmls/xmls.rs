use crate::XmlN;

#[derive(Debug)]
pub enum XmlsError {
    InvalidSymbol(String),
}

pub type XmlS = Vec<u8>;

/// Encode a string into XMLS format.
///
/// This function packs a sequence of XMLN symbols into a byte array.
/// XMLN is composed of 13 symbols, meaning we can represent each symbol with a 4-bit number.
/// Every pair of symbol (using its 4-bit code) is then packed into a single byte.
///
/// # Arguments
/// * `data` - The input XMLN string to encode.
/// # Returns
/// * `Ok(Vec<u8>)` - A vector of bytes representing the encoded XMLS data
/// * `Err(XmlsError)` - An error if the input contains invalid symbols
pub fn encode_xmls(xml_n: &str) -> Result<XmlS, XmlsError> {
    let mut chars = xml_n.chars();
    let mut encoding = XmlS::with_capacity((xml_n.chars().count() + 1) / 2);

    while let Some(left) = chars.next() {
        let lft_nibble = encode_nibble(left)?;

        let byte = if let Some(right) = chars.next() {
            let rgt_nibble = encode_nibble(right)?;
            merge(lft_nibble, rgt_nibble)
        } else {
            // Pad with zeros when symbol count is uneven
            lft_nibble << 4
        };

        encoding.push(byte);
    }

    Ok(encoding)
}

pub fn decode_xmls(xml_s: &[u8]) -> Result<XmlN, XmlsError> {
    let mut decoded = String::with_capacity(xml_s.len() * 2);

    for &byte in xml_s {
        let (lft_nibble, rgt_nibble) = split(byte);
        decoded.push(decode_nibble(lft_nibble)?);
        if rgt_nibble != 0 {
            decoded.push(decode_nibble(rgt_nibble)?);
        }
    }

    Ok(decoded)
}

fn encode_nibble(symbol: char) -> Result<u8, XmlsError> {
    match symbol {
        ' ' => Ok(0x1),
        'T' => Ok(0x2),
        'A' => Ok(0x3),
        '0' => Ok(0x4),
        '1' => Ok(0x5),
        '2' => Ok(0x6),
        '3' => Ok(0x7),
        '4' => Ok(0x8),
        '5' => Ok(0x9),
        '6' => Ok(0xA),
        '7' => Ok(0xB),
        '8' => Ok(0xC),
        '9' => Ok(0xD),
        _ => Err(XmlsError::InvalidSymbol(format!(
            "Invalid symbol: {}",
            symbol
        ))),
    }
}

fn decode_nibble(nibble: u8) -> Result<char, XmlsError> {
    match nibble {
        0x1 => Ok(' '),
        0x2 => Ok('T'),
        0x3 => Ok('A'),
        0x4 => Ok('0'),
        0x5 => Ok('1'),
        0x6 => Ok('2'),
        0x7 => Ok('3'),
        0x8 => Ok('4'),
        0x9 => Ok('5'),
        0xA => Ok('6'),
        0xB => Ok('7'),
        0xC => Ok('8'),
        0xD => Ok('9'),
        _ => Err(XmlsError::InvalidSymbol(format!(
            "Invalid nibble: {}",
            nibble
        ))),
    }
}

fn merge(lft_nibble: u8, rgt_nibble: u8) -> u8 {
    (lft_nibble << 4) | rgt_nibble
}

fn split(byte: u8) -> (u8, u8) {
    (byte >> 4, byte & 0x0F)
}
