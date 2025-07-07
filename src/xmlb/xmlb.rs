use crate::{XFlateError, XmlS};

pub type XmlB = Vec<u8>;

/// Compression level for XMLB data.
/// This flag is an indication for how the backend
/// compressor should optimize its compression.
#[derive(Debug, Clone, Copy)]
pub enum XmlBCompress {
    /// No compression
    None,
    /// Fast compression
    Fast,
    /// Best compression
    Best,
}

/// Implementors of this trait may act as backend compressors
/// for xflate. This is the last step when compressing XMLN data.
pub trait XmlBCompressor {
    /// Compresses data into XMLB binary format
    fn compress(&self, input: &[u8]) -> Result<XmlB, XFlateError>;
}

/// Implementors of this trait may act as backend decompressors
/// for xflate. This is the first step when decompressing XMLB data.
pub trait XmlBDecompressor {
    /// Decompresses data into XMLN encoding
    fn decompress(&self, input: &[u8]) -> Result<XmlS, XFlateError>;
}

pub fn encode_xmlb<D: XmlBCompressor>(data: &[u8], compressor: &D) -> Result<XmlB, XFlateError> {
    compressor.compress(data)
}

pub fn decode_xmlb<D: XmlBDecompressor>(
    data: &[u8],
    decompressor: &D,
) -> Result<XmlS, XFlateError> {
    decompressor.decompress(data)
}
