//! This crate provides types for identifiers of object files, such as executables, dynamic
//! libraries or debug companion files. The concept originates in Google Breakpad and defines two
//! types:
//!
//!  - [`CodeId`]: Identifies the file containing source code, i.e. the actual library or
//!    executable. The identifier is platform dependent and implementation defined. Thus, there is
//!    no canonical representation.
//!  - [`DebugId`]: Identifies a debug information file, which may or may not use information from
//!    the Code ID. The contents are also implementation defined, but as opposed to `CodeId`, the
//!    structure is streamlined across platforms. It is also guaranteed to be 32 bytes in size.
//!
//! [`CodeId`]: struct.CodeId.html
//! [`DebugId`]: struct.DebugId.html

#![warn(missing_docs)]

use std::error;
use std::fmt;
use std::fmt::Write;
use std::str;

use uuid::Uuid;

/// Indicates an error parsing a [`DebugId`](struct.DebugId.html).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseDebugIdError;

impl error::Error for ParseDebugIdError {}

impl fmt::Display for ParseDebugIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid debug identifier")
    }
}

#[derive(Clone, Copy, Debug)]
struct ParseOptions {
    allow_hyphens: bool,
    require_appendix: bool,
    allow_tail: bool,
}

/// Unique identifier for debug information files and their debug information.
///
/// This type is analogous to [`CodeId`], except that it identifies a debug file instead of the
/// actual library or executable. One some platforms, a `DebugId` is an alias for a `CodeId` but the
/// exact rules around this are complex. On Windows, the identifiers are completely different and
/// refer to separate files.
///
/// The string representation must be between 33 and 40 characters long and consist of:
///
/// 1. 36 character hyphenated hex representation of the UUID field
/// 2. 1-16 character lowercase hex representation of the u32 appendix
///
/// The debug identifier is compatible to Google Breakpad. Use [`DebugId::breakpad`] to get a
/// breakpad string representation of this debug identifier.
///
/// There is one exception to this: for the old PDB 2.0 format the debug identifier consists
/// of only a 32-bit integer + age resulting in a string representation of between 9 and 16
/// hex characters.
///
/// # Example
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
///
/// [`CodeId`]: struct.CodeId.html
/// [`DebugId::breakpad`]: struct.DebugId.html#method.breakpad
#[repr(C, packed)]
#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub struct DebugId {
    id: Identifier,
    appendix: u32,
    _padding: [u8; 12],
}

/// The identifier part, normally the UUID.
///
/// This exists purely to support the ancient PDB 2.0 format.
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
enum Identifier {
    /// The PDB 2.0 format identifier.
    ///
    /// This is a timestamp in seconds since the EPOCH.
    Pdb20(u32),
    /// The UUID format identifier for all other formats.
    Uuid(Uuid),
}

impl Default for Identifier {
    fn default() -> Self {
        Self::Uuid(Default::default())
    }
}

impl DebugId {
    /// Constructs an empty debug identifier, containing only zeros.
    pub fn nil() -> Self {
        Self::default()
    }

    /// Constructs a `DebugId` from its `uuid`.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self::from_parts(uuid, 0)
    }

    /// Constructs a `DebugId` from its `uuid` and `appendix` parts.
    pub fn from_parts(uuid: Uuid, appendix: u32) -> Self {
        DebugId {
            id: Identifier::Uuid(uuid),
            appendix,
            _padding: [0; 12],
        }
    }

    /// Constructs a `DebugId` from a Microsoft little-endian GUID and age.
    pub fn from_guid_age(guid: &[u8], age: u32) -> Result<Self, ParseDebugIdError> {
        if guid.len() != 16 {
            return Err(ParseDebugIdError);
        }

        let uuid = Uuid::from_bytes([
            guid[3], guid[2], guid[1], guid[0], guid[5], guid[4], guid[7], guid[6], guid[8],
            guid[9], guid[10], guid[11], guid[12], guid[13], guid[14], guid[15],
        ]);

        Ok(DebugId::from_parts(uuid, age))
    }

    /// Constructs a `DebugId` from a PDB 2.0 timestamp and age.
    pub fn from_timestamp_age(timestamp: u32, age: u32) -> Self {
        DebugId {
            id: Identifier::Pdb20(timestamp),
            appendix: age,
            _padding: [0; 12],
        }
    }

    /// Parses a breakpad identifier from a string.
    pub fn from_breakpad(string: &str) -> Result<Self, ParseDebugIdError> {
        let options = ParseOptions {
            allow_hyphens: false,
            require_appendix: true,
            allow_tail: false,
        };
        Self::parse_str(string, options).ok_or(ParseDebugIdError)
    }

    /// Returns the UUID part of the code module's debug_identifier.
    ///
    /// If this is a debug identifier for the PDB 2.0 format a nil UUID is returned.
    pub fn uuid(&self) -> Uuid {
        match self.id {
            Identifier::Pdb20(_) => Uuid::nil(),
            Identifier::Uuid(uuid) => uuid,
        }
    }

    /// Returns the appendix part of the code module's debug identifier.
    ///
    /// On Windows, this is an incrementing counter to identify the build.
    /// On all other platforms, this value will always be zero.
    pub fn appendix(&self) -> u32 {
        self.appendix
    }

    /// Returns whether this identifier is nil, i.e. it consists only of zeros.
    pub fn is_nil(&self) -> bool {
        let id_is_nil = match self.id {
            Identifier::Pdb20(timestamp) => timestamp == 0,
            Identifier::Uuid(uuid) => uuid.is_nil(),
        };
        id_is_nil && self.appendix() == 0
    }

    /// Returns whether this identifier is from the PDB 2.0 format.
    pub fn is_pdb20(&self) -> bool {
        match self.id {
            Identifier::Pdb20(_) => true,
            Identifier::Uuid(_) => false,
        }
    }

    /// Returns a wrapper which when formatted via `fmt::Display` will format a
    /// a breakpad identifier.
    pub fn breakpad(&self) -> BreakpadFormat<'_> {
        BreakpadFormat { inner: self }
    }

    fn parse_str(string: &str, options: ParseOptions) -> Option<Self> {
        let is_hyphenated = string.get(8..9) == Some("-");
        if is_hyphenated && !options.allow_hyphens || !string.is_ascii() {
            return None;
        }

        // Can the PDB 2.0 format match?  This can never be true for a valid UUID.
        if (is_hyphenated && string.len() >= 10 && string.len() <= 17)
            || (!is_hyphenated && string.len() >= 9 && string.len() <= 16)
        {
            let timestamp_str = string.get(..8)?;
            let timestamp = u32::from_str_radix(timestamp_str, 16).ok()?;
            let appendix_str = match is_hyphenated {
                true => string.get(9..)?,
                false => string.get(8..)?,
            };
            let appendix = u32::from_str_radix(appendix_str, 16).ok()?;
            return Some(Self::from_timestamp_age(timestamp, appendix));
        }

        let uuid_len = if is_hyphenated { 36 } else { 32 };

        let uuid = string.get(..uuid_len)?.parse().ok()?;
        if !options.require_appendix && string.len() == uuid_len {
            return Some(Self::from_parts(uuid, 0));
        }

        let mut appendix_str = &string[uuid_len..];
        if is_hyphenated ^ appendix_str.starts_with('-') {
            return None; // Require a hyphen if and only if we're hyphenated.
        } else if is_hyphenated {
            appendix_str = &appendix_str[1..]; // Skip the hyphen for parsing.
        }

        if options.allow_tail && appendix_str.len() > 8 {
            appendix_str = &appendix_str[..8];
        }

        // Parse the appendix, which fails on empty strings.
        let appendix = u32::from_str_radix(appendix_str, 16).ok()?;
        Some(Self::from_parts(uuid, appendix))
    }
}

impl fmt::Debug for DebugId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.id {
            Identifier::Pdb20(timestamp) => f
                .debug_struct("DebugId")
                .field("timestamp", &timestamp)
                .field("appendix", &self.appendix())
                .finish(),
            Identifier::Uuid(uuid) => f
                .debug_struct("DebugId")
                .field("uuid", &uuid.to_hyphenated_ref().to_string())
                .field("appendix", &self.appendix())
                .finish(),
        }
    }
}

impl fmt::Display for DebugId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.id {
            Identifier::Pdb20(timestamp) => write!(f, "{:08X}", timestamp)?,
            Identifier::Uuid(uuid) => uuid.fmt(f)?,
        }
        if self.appendix > 0 {
            write!(f, "-{:x}", { self.appendix })?;
        }
        Ok(())
    }
}

impl str::FromStr for DebugId {
    type Err = ParseDebugIdError;

    fn from_str(string: &str) -> Result<Self, ParseDebugIdError> {
        let options = ParseOptions {
            allow_hyphens: true,
            require_appendix: false,
            allow_tail: true,
        };
        Self::parse_str(string, options).ok_or(ParseDebugIdError)
    }
}

impl From<Uuid> for DebugId {
    fn from(uuid: Uuid) -> Self {
        DebugId::from_uuid(uuid)
    }
}

impl From<(Uuid, u32)> for DebugId {
    fn from(tuple: (Uuid, u32)) -> Self {
        let (uuid, appendix) = tuple;
        DebugId::from_parts(uuid, appendix)
    }
}

/// Wrapper around [`DebugId`] for Breakpad formatting.
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
///
/// [`DebugId`]: struct.DebugId.html
#[derive(Debug)]
pub struct BreakpadFormat<'a> {
    inner: &'a DebugId,
}

impl<'a> fmt::Display for BreakpadFormat<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner.id {
            Identifier::Pdb20(timestamp) => {
                write!(f, "{:08X}{:x}", timestamp, self.inner.appendix())
            }
            Identifier::Uuid(uuid) => {
                write!(f, "{:X}{:x}", uuid.to_simple_ref(), self.inner.appendix())
            }
        }
    }
}

/// Indicates an error parsing a [`CodeId`](struct.CodeId.html).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseCodeIdError;

impl error::Error for ParseCodeIdError {}

impl fmt::Display for ParseCodeIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid code identifier")
    }
}

/// Unique platform-dependent identifier of code files.
///
/// This identifier assumes a string representation that depends on the platform and compiler used.
/// The representation only retains hex characters and canonically stores lower case.
///
/// There are the following known formats:
///
///  - **MachO UUID**: The unique identifier of a Mach binary, specified in the `LC_UUID` load
///    command header.
///  - **GNU Build ID**: Contents of the `.gnu.build-id` note or section contents formatted as
///    lowercase hex string.
///  - **PE Timestamp**: Timestamp and size of image values from a Windows PE header. The size of
///    image value is truncated, so the length of the `CodeId` might not be a multiple of 2.
#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CodeId {
    inner: String,
}

impl CodeId {
    /// Constructs an empty code identifier.
    pub fn nil() -> Self {
        Self::default()
    }

    /// Constructs a `CodeId` from its string representation.
    pub fn new(mut string: String) -> Self {
        string.retain(|c| c.is_ascii_hexdigit());
        string.make_ascii_lowercase();
        CodeId { inner: string }
    }

    /// Constructs a `CodeId` from a binary slice.
    pub fn from_binary(slice: &[u8]) -> Self {
        let mut string = String::with_capacity(slice.len() * 2);

        for byte in slice {
            write!(&mut string, "{:02x}", byte).expect("");
        }

        Self::new(string)
    }

    /// Returns whether this identifier is nil, i.e. it is empty.
    pub fn is_nil(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the string representation of this code identifier.
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }
}

impl fmt::Display for CodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.inner)
    }
}

impl fmt::Debug for CodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CodeId({})", self)
    }
}

impl From<String> for CodeId {
    fn from(string: String) -> Self {
        Self::new(string)
    }
}

impl From<&'_ str> for CodeId {
    fn from(string: &str) -> Self {
        Self::new(string.into())
    }
}

impl AsRef<str> for CodeId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl str::FromStr for CodeId {
    type Err = ParseCodeIdError;

    fn from_str(string: &str) -> Result<Self, ParseCodeIdError> {
        Ok(Self::new(string.into()))
    }
}

#[cfg(feature = "serde")]
mod serde_support {
    use serde::de::{self, Deserialize, Deserializer, Unexpected, Visitor};
    use serde::ser::{Serialize, Serializer};

    use super::*;

    impl Serialize for CodeId {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(self.as_str())
        }
    }

    impl<'de> Deserialize<'de> for CodeId {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let string = String::deserialize(deserializer)?;
            Ok(CodeId::new(string))
        }
    }

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
