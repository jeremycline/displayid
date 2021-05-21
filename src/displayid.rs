/// Provides the DisplayID parser
use super::Error;

const STRUCTURE_VERSION: usize = 0;
const BYTES_IN_SECTION: usize = 1;
const PRIMARY_USE_CASE: usize = 2;
const EXTENSION_COUNT: usize = 3;

#[derive(Debug)]
pub struct DisplayIdSection<'a> {
    section: &'a [u8],
}

/// The DisplayID structure.
///
/// A DisplayID is made up of up to 256 variable-length sections, each of which may be up to 256
/// bytes long.
#[derive(Debug)]
pub struct DisplayId<'a> {
    // The entire DisplayID structure
    blob: &'a [u8],

    // The offset into `blob` of the current section we're iterating over
    current_section: usize,
}

impl<'a> Iterator for DisplayId<'a> {
    type Item = DisplayIdSection<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.blob.len() < self.current_section + 5 {
            return None;
        }

        let section_size = self.blob[self.current_section + BYTES_IN_SECTION] as usize + 5;
        let section_start = self.current_section;
        if self.blob.len() > self.current_section + section_size {
            self.current_section += section_size;
            Some(DisplayIdSection {
                section: &self.blob[section_start..section_size],
            })
        } else {
            self.current_section = 0;
            None
        }
    }
}

impl DisplayId<'_> {
    /// Create and validate a new DisplayID structure.
    pub fn new(blob: &[u8]) -> Result<DisplayId, Error> {
        let mut displayid = DisplayId {
            blob,
            current_section: 0,
        };
        displayid.validate()?;
        Ok(displayid)
    }

    /// Validate the DisplayID structure.
    ///
    /// DisplayID requires each section to contain 5 mandatory 1-byte fields, including a checksum.
    /// This asserts the require fields are present in each section, are within the allowable
    /// ranges, and computes the checksum.
    pub fn validate(&mut self) -> Result<(), Error> {
        // Start by ensuring the base block is at least long enough in theory to be iterated on.
        if self.blob.len() < 5 {
            return Err(Error::MandatoryFieldMissing(self.blob.len()));
        }

        for section in self {
            let blob = section.section;
            if blob.len() < 5 {
                return Err(Error::MandatoryFieldMissing(blob.len()));
            }

            // Only 2.0 is supported at the moment.
            if blob[STRUCTURE_VERSION] != 0x20 {
                return Err(Error::UnknownVersion(blob[STRUCTURE_VERSION]));
            }

            // As of 2.0 only 15 primary use-cases are known
            // TODO maybe allow more and offer a hook for users to handle.
            if blob[PRIMARY_USE_CASE] > 0xf {
                return Err(Error::UnknownPrimaryUseCase(blob[PRIMARY_USE_CASE]));
            }

            let expected_checksum = *blob.last().unwrap();
            let actual_checksum = blob[..blob.len() - 1].iter().sum();
            if actual_checksum != expected_checksum {
                return Err(Error::InvalidChecksum(actual_checksum, expected_checksum));
            }
        }

        Ok(())
    }

    /// Retrieve the number of extensions in the DisplayID.
    pub fn extension_count(&self) -> u8 {
        self.blob[EXTENSION_COUNT]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_missing_fields() {
        let mut blob = vec![];
        for x in 0..5 {
            match DisplayId::new(&blob) {
                Err(Error::MandatoryFieldMissing(count)) => {
                    assert_eq!(x, count)
                }
                _ => {
                    panic!("Expected DisplayId::new to return an Err, but it didn't")
                }
            }
            blob.push(0);
        }
    }

    #[test]
    fn extension_count() {
        let blob = vec![0x20, 0, 0, 0, 0];
        let display_id = DisplayId::new(&blob).unwrap();

        assert_eq!(0, display_id.extension_count());
    }
}
