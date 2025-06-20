use nsip::encode_xmln;
use nsip::{DynamicSymbolTable, DynamicTagTable};

#[cfg(test)]
mod symbol_table_tests {

    use std::fs::File;

    use super::*;

    #[test]
    fn test_dynamic_symbol_table() {
        let file = File::open("tests/data/basic.xml").expect("Failed to open test file");

        let mut sym_table = DynamicSymbolTable::new();
        let mut tag_table = DynamicTagTable::new();
        let res = encode_xmln(file, &mut sym_table, &mut tag_table);

        println!("{}", res);
    }
}
