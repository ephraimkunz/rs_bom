use crate::{BOMError, Reference, BOM};
use std::{fmt, iter, str};

/// Types of references that we'll parse:
/// * Alma 3:16
/// * Alma 3:16 - 17
/// * Alma 3:16, 18-20 & 13:2-4
#[derive(Debug)]
pub struct ReferenceCollection {
    refs: Vec<Reference>,
}

impl ReferenceCollection {
    fn is_valid(&self, bom: &BOM) -> bool {
        self.refs.iter().all(|r| r.is_valid(bom))
    }
}

impl iter::FromIterator<Reference> for ReferenceCollection {
    fn from_iter<I: IntoIterator<Item = Reference>>(iter: I) -> Self {
        ReferenceCollection{
            refs: iter.into_iter().collect()
        }
    }
}

impl str::FromStr for ReferenceCollection {
    type Err = BOMError;
    fn from_str(_: &str) -> Result<Self, Self::Err> {
        unimplemented!()
    }
}

impl fmt::Display for ReferenceCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        unimplemented!()
    }
}
