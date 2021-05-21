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
}

pub mod displayid;
pub mod edid;
