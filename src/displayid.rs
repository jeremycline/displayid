/// Provides the DisplayID parser
use super::Error;

const STRUCTURE_VERSION: usize = 0;
const BYTES_IN_SECTION: usize = 1;
const PRIMARY_USE_CASE: usize = 2;
const EXTENSION_COUNT: usize = 3;

/// The primary use case for a DisplayID section.
pub enum PrimaryUseCase {
    Extension,
    Test,
    Generic,
    Television,
    Productivity,
    Gaming,
    Presentation,
    VirtualReality,
    AugmentedReality,
}

#[derive(Debug)]
pub struct DisplayIdSection<'a> {
    section: &'a [u8],
}

impl DisplayIdSection<'_> {
    pub fn primary_use_case(&self) -> PrimaryUseCase {
        PrimaryUseCase::Test
    }
}

/// The DisplayID structure.
///
/// A DisplayID is made up of up to 256 variable-length sections, each of which may be up to 256
/// bytes long.
#[derive(Debug)]
pub struct DisplayId<'a> {
    // The entire DisplayID structure
    blob: &'a [u8],

    sections: Vec<DisplayIdSection<'a>>,
}

impl<'a, Idx> std::ops::Index<Idx> for DisplayId<'a>
where
    Idx: std::slice::SliceIndex<[DisplayIdSection<'a>]>,
{
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.sections[index]
    }
}

impl<'a> IntoIterator for &'a DisplayId<'a> {
    type Item = &'a DisplayIdSection<'a>;
    type IntoIter = std::slice::Iter<'a, DisplayIdSection<'a>>;

    fn into_iter(self) -> std::slice::Iter<'a, DisplayIdSection<'a>> {
        self.sections.iter()
    }
}

impl DisplayId<'_> {
    /// Create and validate a new DisplayID structure.
    pub fn new(blob: &[u8]) -> Result<DisplayId, Error> {
        let display_id_len = blob.len();
        if display_id_len > 256 * 256 {
            return Err(Error::TooLarge(display_id_len));
        }
        if display_id_len < 5 {
            return Err(Error::MandatoryFieldMissing(display_id_len));
        }

        let mut section_start = 0;
        let mut section_size = 0;
        let mut sections = vec![];

        for _ in 0..blob[EXTENSION_COUNT] {
            section_start += section_size;
            if section_start + BYTES_IN_SECTION >= display_id_len {
                return Err(Error::Malformed);
            }

            section_size = blob[section_start + BYTES_IN_SECTION] as usize + 5;
            if section_start + section_size > display_id_len {
                return Err(Error::Malformed);
            }
            sections.push(DisplayIdSection {
                section: &blob[section_start..section_start + section_size],
            });
        }

        let displayid = DisplayId { blob, sections };
        displayid.validate()?;
        Ok(displayid)
    }

    /// Validate the DisplayID structure.
    ///
    /// DisplayID requires each section to contain 5 mandatory 1-byte fields, including a checksum.
    /// This asserts the require fields are present in each section, are within the allowable
    /// ranges, and computes the checksum.
    pub fn validate(&self) -> Result<(), Error> {
        for section in &self.sections {
            let blob = section.section;
            if blob.len() < 5 {
                return Err(Error::MandatoryFieldMissing(blob.len()));
            }

            // Only 2.0 is supported at the moment.
            if blob[STRUCTURE_VERSION] != 0x20 {
                return Err(Error::UnknownVersion(blob[STRUCTURE_VERSION]));
            }

            // As of 2.0 only 15 primary use-cases are known
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
    pub fn extension_count(&self) -> usize {
        self.sections.len()
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
        let blob = vec![0x20, 0, 0, 0, 0x20];
        let display_id = DisplayId::new(&blob).unwrap();

        assert_eq!(0, display_id.extension_count());
    }
}
