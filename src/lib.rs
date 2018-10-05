//! This crate implements a type that is a thin wrapper around uuids that
//! can hold a "debug id".  This is a concept that originally comes from
//! breakpad and is also used by Sentry to identify a debug information
//! file.
//!
//! The reason this is not just a UUID is that at least on Windows a debug
//! information file (PDB) is not just associated by using a UUID alone
//! but it has an appendix (a `u32` age field).
//!
//! ## Representation
//!
//! The string representation must be between 33 and 40 characters long and
//! consist of:
//!
//! 1. 36 character hyphenated hex representation of the UUID field
//! 2. 1-16 character lowercase hex representation of the u64 appendix
//!
//! ```
//! # extern crate debugid;
//! use debugid::DebugId;
//!
//! # fn foo() -> Result<(), ::debugid::ParseDebugIdError> {
//! let id: DebugId = "dfb8e43a-f242-3d73-a453-aeb6a777ef75-a".parse()?;
//! assert_eq!("dfb8e43a-f242-3d73-a453-aeb6a777ef75-a".to_string(), id.to_string());
//! # Ok(())
//! # }
//!
//! # fn main() { foo().unwrap() }
//! ```
//!
//! ## Breakpad compatibility
//!
//! Separately the breakpad format can be generated as well:
//!
//! ```
//! # extern crate debugid;
//! use debugid::DebugId;
//!
//! # fn foo() -> Result<(), ::debugid::ParseDebugIdError> {
//! let id: DebugId = "dfb8e43a-f242-3d73-a453-aeb6a777ef75-a".parse()?;
//! assert_eq!(id.breakpad().to_string(), "DFB8E43AF2423D73A453AEB6A777EF75a");
//! # Ok(())
//! # }
//!
//! # fn main() { foo().unwrap() }
//! ```
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate uuid;

#[cfg(feature = "with_serde")]
extern crate serde;

use regex::Regex;
use std::error;
use std::fmt;
use std::str;
use uuid::Uuid;

lazy_static! {
    static ref DEBUG_ID_RE: Regex = Regex::new(
        r"(?ix)
        ^
            ([0-9a-f]{8}-?
             [0-9a-f]{4}-?
             [0-9a-f]{4}-?
             [0-9a-f]{4}-?
             [0-9a-f]{12})
            -?
            ([0-9a-f]{1,8})?
        $
    "
    ).unwrap();
}

/// Indicates a parsing error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseDebugIdError;

impl error::Error for ParseDebugIdError {
    fn description(&self) -> &str {
        "invalid debug identifier"
    }
}

impl fmt::Display for ParseDebugIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        write!(f, "{}", self.description())
    }
}

/// Unique identifier for debug information files and their debug information.
///
/// The string representation must be between 33 and 40 characters long and
/// consist of:
///
/// 1. 36 character hyphenated hex representation of the UUID field
/// 2. 1-16 character lowercase hex representation of the u64 appendix
///
/// **Example:**
///
/// ```
/// # extern crate debugid;
/// use std::str::FromStr;
/// use debugid::DebugId;
///
/// # fn foo() -> Result<(), ::debugid::ParseDebugIdError> {
/// let id = DebugId::from_str("dfb8e43a-f242-3d73-a453-aeb6a777ef75-a")?;
/// assert_eq!("dfb8e43a-f242-3d73-a453-aeb6a777ef75-a".to_string(), id.to_string());
/// # Ok(())
/// # }
///
/// # fn main() { foo().unwrap() }
/// ```
#[repr(C, packed)]
#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub struct DebugId {
    uuid: Uuid,
    appendix: u32,
    _padding: [u8; 12],
}

impl DebugId {
    /// Constructs a `DebugId` from its `uuid`.
    pub fn from_uuid(uuid: Uuid) -> DebugId {
        Self::from_parts(uuid, 0)
    }

    /// Constructs a `DebugId` from its `uuid` and `appendix` parts.
    pub fn from_parts(uuid: Uuid, appendix: u32) -> DebugId {
        DebugId {
            uuid,
            appendix,
            _padding: [0; 12],
        }
    }

    /// Constructs a `DebugId` from a Microsoft little-endian GUID and age.
    pub fn from_guid_age(guid: &[u8], age: u32) -> Result<DebugId, ParseDebugIdError> {
        if guid.len() != 16 {
            return Err(ParseDebugIdError);
        }

        let uuid = Uuid::from_bytes([
            guid[3], guid[2], guid[1], guid[0], guid[5], guid[4], guid[7], guid[6], guid[8],
            guid[9], guid[10], guid[11], guid[12], guid[13], guid[14], guid[15],
        ]);

        Ok(DebugId::from_parts(uuid, age))
    }

    /// Parses a breakpad identifier from a string.
    pub fn from_breakpad(string: &str) -> Result<DebugId, ParseDebugIdError> {
        // Technically, we are are too permissive here by allowing dashes, but
        // we are complete.
        string.parse()
    }

    /// Returns the UUID part of the code module's debug_identifier.
    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    /// Returns the appendix part of the code module's debug identifier.
    ///
    /// On Windows, this is an incrementing counter to identify the build.
    /// On all other platforms, this value will always be zero.
    pub fn appendix(&self) -> u32 {
        self.appendix
    }

    /// Returns a wrapper which when formatted via `fmt::Display` will format a
    /// a breakpad identifier.
    pub fn breakpad(&self) -> BreakpadFormat {
        BreakpadFormat { inner: self }
    }
}

impl fmt::Debug for DebugId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DebugId")
            .field("uuid", &self.uuid().to_hyphenated_ref().to_string())
            .field("appendix", &self.appendix())
            .finish()
    }
}

impl fmt::Display for DebugId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.uuid.fmt(f)?;
        if self.appendix > 0 {
            write!(f, "-{:x}", { self.appendix })?;
        }
        Ok(())
    }
}

impl str::FromStr for DebugId {
    type Err = ParseDebugIdError;

    fn from_str(string: &str) -> Result<DebugId, ParseDebugIdError> {
        let captures = DEBUG_ID_RE.captures(string).ok_or(ParseDebugIdError)?;
        let uuid = captures
            .get(1)
            .unwrap()
            .as_str()
            .parse()
            .map_err(|_| ParseDebugIdError)?;
        let appendix = captures
            .get(2)
            .map_or(Ok(0), |s| u32::from_str_radix(s.as_str(), 16))
            .map_err(|_| ParseDebugIdError)?;
        Ok(DebugId::from_parts(uuid, appendix))
    }
}

impl From<Uuid> for DebugId {
    fn from(uuid: Uuid) -> DebugId {
        DebugId::from_uuid(uuid)
    }
}

impl From<(Uuid, u32)> for DebugId {
    fn from(tuple: (Uuid, u32)) -> DebugId {
        let (uuid, appendix) = tuple;
        DebugId::from_parts(uuid, appendix)
    }
}

#[cfg(feature = "with_serde")]
mod serde_support {
    use serde::de::{self, Deserialize, Deserializer, Unexpected, Visitor};
    use serde::ser::{Serialize, Serializer};

    use super::*;

    impl<'de> Deserialize<'de> for DebugId {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct V;

            impl<'de> Visitor<'de> for V {
                type Value = DebugId;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("DebugId")
                }

                fn visit_str<E: de::Error>(self, value: &str) -> Result<DebugId, E> {
                    value
                        .parse()
                        .map_err(|_| de::Error::invalid_value(Unexpected::Str(value), &self))
                }
            }

            deserializer.deserialize_str(V)
        }
    }

    impl Serialize for DebugId {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&self.to_string())
        }
    }
}

/// Wrapper around `DebugId` for Breakpad formatting.
///
/// **Example:**
///
/// ```
/// # extern crate debugid;
/// use std::str::FromStr;
/// use debugid::DebugId;
///
/// # fn foo() -> Result<(), debugid::ParseDebugIdError> {
/// let id = DebugId::from_breakpad("DFB8E43AF2423D73A453AEB6A777EF75a")?;
/// assert_eq!("DFB8E43AF2423D73A453AEB6A777EF75a".to_string(), id.breakpad().to_string());
/// # Ok(())
/// # }
///
/// # fn main() { foo().unwrap() }
/// ```
#[derive(Debug)]
pub struct BreakpadFormat<'a> {
    inner: &'a DebugId,
}

impl<'a> fmt::Display for BreakpadFormat<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:X}{:x}",
            self.inner.uuid().to_simple_ref(),
            self.inner.appendix()
        )
    }
}
