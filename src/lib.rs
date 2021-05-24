pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Each DisplayID Section must contain 5 mandatory bytes, but section has {0}")]
    MandatoryFieldMissing(usize),
    #[error("DisplayID Structure version was the unknown value {0}")]
    UnknownVersion(u8),
    #[error("DisplayID section Primary Use Case was {0}; valid values are 0-15")]
    UnknownPrimaryUseCase(u8),
    #[error("DisplayID section checksum was {0}, but the expected checksum was {0}")]
    InvalidChecksum(u8, u8),
    #[error("DisplayID cannot be larger than 256 * 256 bytes, {0} was provided")]
    TooLarge(usize),
    #[error("The DisplayID section size exceeded the total structure size")]
    Malformed,
}

pub mod displayid;
pub mod edid;
