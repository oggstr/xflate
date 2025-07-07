use std::{
    fs,
    io::{Cursor, Read},
};

use xflate::{XFlate, XFlateConfig};

fn main() {
    let xml = fs::read_to_string("tests/data/basic.xml").expect("Failed to open file");
    let cursor1 = Cursor::new(&xml);
    let cursor2 = Cursor::new(&xml);

    let config = XFlateConfig::from_xml(cursor1).expect("Failed to create XFlateConfig");
    let mut xflate = XFlate::new(config);

    let compressed = xflate.compress(cursor2).expect("Failed to compress XML");

    println!("Bytes original: {}", xml.bytes().len());
    println!("Bytes compressed: {:?}", compressed.bytes().count());

    let decompressed = xflate
        .decompress(Cursor::new(compressed))
        .expect("Failed to decompress XML");
    println!("Bytes decompressed: {}", decompressed.len());

    println!("");
    println!("----- Original document -----");
    println!("{}", xml);

    println!("");
    println!("----- Decompressed document -----");
    println!("{}", decompressed);
}
