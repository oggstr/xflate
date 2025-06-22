use crate::XmlS;

pub type XmlB = Vec<u8>;

/// Errors that may occur during XMLB
/// compression or decompression.
#[derive(Debug)]
pub enum XmlBError {
    /// Error during compression
    CompressionError(String),
    /// Error during decompression
    DecompressionError(String),
}

/// Compression level for XMLB data.
/// This flag is an indication for how the backend
/// compressor should optimize its compression.
pub enum XmlBCompress {
    /// No compression
    None,
    /// Fast compression
    Fast,
    /// Best compression
    Best,
}

/// Implementors of this trait may act as backend compressors
/// for NSIP. This is the last step when compressing XMLN data.
pub trait XmlBCompressor {
    /// Compresses data into XMLB binary format
    fn compress(&self, input: &[u8]) -> Result<XmlB, XmlBError>;
}

/// Implementors of this trait may act as backend decompressors
/// for NSIP. This is the first step when decompressing XMLB data.
pub trait XmlBDecompressor {
    /// Decompresses data into XMLN encoding
    fn decompress(&self, input: &[u8]) -> Result<XmlS, XmlBError>;
}

pub fn encode_xmlb<D: XmlBCompressor>(data: &[u8], compressor: &D) -> Result<XmlB, XmlBError> {
    compressor.compress(data)
}

pub fn decode_xmlb<D: XmlBDecompressor>(data: &[u8], decompressor: &D) -> Result<XmlS, XmlBError> {
    decompressor.decompress(data)
}
