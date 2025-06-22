use std::io::Write;

use flate2::write::{DeflateDecoder, DeflateEncoder};

use crate::{XmlB, XmlBCompress, XmlBCompressor, XmlBDecompressor, XmlBError, XmlS};

pub struct XmlBDeflateBackend {
    opt: XmlBCompress,
}

impl XmlBDeflateBackend {
    pub fn new(opt: XmlBCompress) -> Self {
        XmlBDeflateBackend { opt }
    }
}

impl XmlBCompressor for XmlBDeflateBackend {
    fn compress(&self, buf: &[u8]) -> Result<XmlB, XmlBError> {
        let mut encoder = DeflateEncoder::new(
            Vec::new(),
            match self.opt {
                XmlBCompress::None => flate2::Compression::none(),
                XmlBCompress::Fast => flate2::Compression::fast(),
                XmlBCompress::Best => flate2::Compression::best(),
            },
        );

        encoder
            .write_all(buf)
            .map_err(|e| XmlBError::CompressionError(e.to_string()))?;

        encoder
            .finish()
            .map_err(|e| XmlBError::CompressionError(e.to_string()))
    }
}

impl XmlBDecompressor for XmlBDeflateBackend {
    fn decompress(&self, buf: &[u8]) -> Result<XmlS, XmlBError> {
        let mut decoder = DeflateDecoder::new(Vec::new());

        decoder
            .write_all(buf)
            .map_err(|e| XmlBError::DecompressionError(e.to_string()))?;

        decoder
            .finish()
            .map_err(|e| XmlBError::DecompressionError(e.to_string()))
    }
}
