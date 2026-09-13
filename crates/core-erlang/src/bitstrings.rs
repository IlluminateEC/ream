use core::fmt;

use super::Expr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BitType {
    Integer,
    Float,
    Binary,
    Utf8,
    Utf16,
    Utf32,
}

impl fmt::Display for BitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer => write!(f, "integer"),
            Self::Float => write!(f, "float"),
            Self::Binary => write!(f, "binary"),
            Self::Utf8 => write!(f, "utf8"),
            Self::Utf16 => write!(f, "utf16"),
            Self::Utf32 => write!(f, "utf32"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Signedness {
    Signed,
    Unsigned,
}

impl fmt::Display for Signedness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Signed => write!(f, "signed"),
            Self::Unsigned => write!(f, "unsigned"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Endianness {
    Big,
    Little,
    Native,
}

impl fmt::Display for Endianness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Big => write!(f, "big"),
            Self::Little => write!(f, "little"),
            Self::Native => write!(f, "native"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BitSegment<T> {
    pub value: Box<T>,
    pub bit_size: Option<Box<Expr>>,
    pub item_count: Option<u64>,
    pub segment_type: BitType,
    pub signedness: Option<Signedness>,
    pub endianness: Option<Endianness>,
}

macro_rules! comma_value_or_empty {
    ($value:expr) => {
        $value
            .as_ref()
            .map_or_else(String::new, |value| format!(", {value}"))
    };
}

impl<T: fmt::Display> fmt::Display for BitSegment<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#<{}>({}{}{}, {}{})",
            self.value,
            comma_value_or_empty!(self.bit_size),
            comma_value_or_empty!(self.item_count),
            self.segment_type,
            comma_value_or_empty!(self.signedness),
            comma_value_or_empty!(self.endianness)
        )
    }
}
