use std::io::Read;

use itertools::Itertools;
use xflate::{self, XmlBCompressor, XmlNSymbolTable};

static BASIC_XML: &str = r#"<section xml:lang="en">
    <title>Basic XML Document</title>
    <para id="183504">Sed risus tortor, auctor non dictum ut, porttitor vel turpis.</para>
    <para id="239085">Sed consectetur, nulla quis consequat placerat.</para>
    <para id="617122">Magna lectus blandit elit, vitae congue ligula arcu sed nulla.</para>
    <para id="135135">Ut fermentum est turpis, sed bibendum ex pellentesque quis.</para>
    <para id="118376">Donec eget dui vestibulum, gravida felis eget, pulvinar odio.</para>
    <para id="102831">Quisque consequat venenatis nisl eu fringilla.</para>
    <para id="991681">Donec nibh mi, congue eu auctor sed, pharetra ac tortor.</para>
    <para id="123173">Praesent pulvinar magna porttitor dolor volutpat, et rutrum ante euismod.</para>
    <para id="987623">Suspendisse consectetur metus sit amet est dictum, a viverra purus sodales.</para>
    <para id="697674">Cras et porta turpis.</para>
    <para id="123412">Maecenas eleifend est varius felis facilisis rutrum.</para>
    <para id="467545">Mauris egestas arcu ut auctor interdum.</para>
    <para id="980092">Cras vitae nisl molestie, gravida nisi at, interdum purus.</para>
    <para id="163384">Suspendisse nec posuere metus. Sed lobortis sed urna nec lobortis. </para>
</section>
"#;

fn main() {
    let xml = BASIC_XML;
    let file = xml.as_bytes();

    const SYMBOL_SIZE: u8 = 2;

    let mut sym_table = xflate::XmlNDynamicSymbolTable::new(SYMBOL_SIZE);
    let mut tag_table = xflate::XmlNDynamicTagTable::new();

    println!("===== xflate Compression =====");
    println!("");

    println!("XML: {}", xml);
    println!("");

    let xmln = match xflate::encode_xmln(file, &mut sym_table, &mut tag_table) {
        Ok(xmln) => xmln,
        Err(err) => {
            panic!("Error encoding XMLN: {:?}", err);
        }
    };
    println!("XMLN: {}", xmln);
    println!("");

    let xmls = match xflate::encode_xmls(xmln.as_str()) {
        Ok(xmls) => xmls,
        Err(err) => {
            panic!("Error encoding XMLS: {:?}", err);
        }
    };
    println!("XMLS: {:?}", xmls);
    println!("");

    let backend = xflate::XmlBDeflateBackend::new(xflate::XmlBCompress::Fast);
    let xmlb = match backend.compress(&xmls) {
        Ok(xmlb) => xmlb,
        Err(err) => {
            panic!("Error compressing XMLB: {:?}", err);
        }
    };
    println!("XMLB: {:?}", xmlb);
    println!("");

    println!("=== xflate Compression End ===");
    println!("");

    println!("{}", sym_table);
    println!("");
    println!("{}", tag_table);
    println!("");

    println!("===  xflate Decompression  ===");
    println!("");

    println!("XMLB: {:?}", xmlb);
    println!("");

    let xmls = match xflate::decode_xmlb(&xmlb, &backend) {
        Ok(xmls) => xmls,
        Err(err) => {
            panic!("Error decompressing XMLB: {:?}", err);
        }
    };
    println!("XMLS: {:?}", xmls);
    println!("");

    let xmln = match xflate::decode_xmls(&xmls) {
        Ok(xmln) => xmln,
        Err(err) => {
            panic!("Error decoding XMLS: {:?}", err);
        }
    };
    println!("XMLN: {}", xmln);
    println!("");

    let xml = xflate::decode_xmln(xmln.as_str(), &mut sym_table, &mut tag_table);
    if let Err(err) = xml {
        panic!("Error decoding XMLN: {:?}", err);
    }
    println!("XML: {}", xml.unwrap());
    println!("");

    println!("=== xflate Decompression End ===");
    println!("");

    println!("=======     Results     =======");
    println!("XML Bytes: {:?}", file.bytes().count());
    println!("XMLN Bytes: {}", xmln.bytes().count());
    println!("XMLS Bytes: {:?}", xmls.bytes().count());
    println!("XMLB Bytes: {:?}", xmlb.bytes().count());
    println!(
        "Compression Ratio (compress / orig): {:.2}%",
        (xmlb.bytes().count() as f64 / file.bytes().count() as f64) * 100.0
    );
}
