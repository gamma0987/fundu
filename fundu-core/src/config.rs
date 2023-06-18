// Copyright (c) 2023 Joining7943 <joining@posteo.de>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::time::{Multiplier, TimeUnit, DEFAULT_TIME_UNIT};

pub(crate) const DEFAULT_CONFIG: Config = Config::new();

/// An ascii delimiter defined as closure.
///
/// The [`Delimiter`] is a type alias for a closure taking a `u8` byte and returning a `bool`. Most
/// likely, the [`Delimiter`] is used to define some whitespace but whitespace definitions differ,
/// so a closure provides the most flexible definition of a delimiter. For example the definition of
/// whitespace from rust [`u8::is_ascii_whitespace`]:
///
/// ```text
/// Checks if the value is an ASCII whitespace character: U+0020 SPACE, U+0009 HORIZONTAL TAB,
/// U+000A LINE FEED, U+000C FORM FEED, or U+000D CARRIAGE RETURN.
///
/// Rust uses the WhatWG Infra Standard’s definition of ASCII whitespace. There are several other
/// definitions in wide use. For instance, the POSIX locale includes U+000B VERTICAL TAB as well
/// as all the above characters, but—from the very same specification—the default rule for “field
/// splitting” in the Bourne shell considers only SPACE, HORIZONTAL TAB, and LINE FEED as
/// whitespace.
/// ```
///
/// # Problems
///
/// The delimiter takes a `u8` as input, but matching any non-ascii (`0x80 - 0xff`) bytes may lead
/// to a [`crate::ParseError`] if the input string contains multi-byte utf-8 characters.
///
/// # Examples
///
/// ```rust
/// use fundu_core::config::Delimiter;
///
/// fn is_delimiter(delimiter: Delimiter, byte: u8) -> bool {
///     delimiter(byte)
/// }
///
/// assert!(is_delimiter(
///     |byte| matches!(byte, b' ' | b'\n' | b'\t'),
///     b' '
/// ));
/// assert!(!is_delimiter(
///     |byte| matches!(byte, b' ' | b'\n' | b'\t'),
///     b'\r'
/// ));
/// assert!(is_delimiter(|byte| byte.is_ascii_whitespace(), b'\r'));
/// ```
pub type Delimiter = fn(u8) -> bool;

/// TODO: DOCUMENT
#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(clippy::struct_excessive_bools)]
#[non_exhaustive]
pub struct Config<'a> {
    /// TODO: DOCUMENT
    pub allow_delimiter: Option<Delimiter>,
    /// TODO: DOCUMENT
    pub default_unit: TimeUnit,
    /// TODO: DOCUMENT
    pub default_multiplier: Multiplier,
    /// TODO: DOCUMENT
    pub disable_exponent: bool,
    /// TODO: DOCUMENT
    pub disable_fraction: bool,
    /// TODO: DOCUMENT
    pub disable_infinity: bool,
    /// TODO: DOCUMENT
    pub number_is_optional: bool,
    /// TODO: DOCUMENT, REMOVE ??
    pub max_exponent: i16,
    /// TODO: DOCUMENT, REMOVE ??
    pub min_exponent: i16,
    /// TODO: DOCUMENT, RENAME to delimiter_parse_multiple
    pub parse_multiple_delimiter: Option<Delimiter>,
    /// TODO: DOCUMENT, RENAME to conjunctions_parse_multiple or just conjunctions
    pub parse_multiple_conjunctions: Option<&'a [&'a str]>,
    /// TODO: DOCUMENT
    pub allow_negative: bool,
    /// TODO: DOCUMENT
    pub allow_ago: Option<Delimiter>,
}

impl<'a> Default for Config<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Config<'a> {
    /// TODO: DOCUMENT
    pub const fn new() -> Self {
        Self {
            allow_delimiter: None,
            default_unit: DEFAULT_TIME_UNIT,
            default_multiplier: Multiplier(1, 0),
            disable_exponent: false,
            disable_fraction: false,
            number_is_optional: false,
            max_exponent: i16::MAX,
            min_exponent: i16::MIN,
            disable_infinity: false,
            parse_multiple_delimiter: None,
            parse_multiple_conjunctions: None,
            allow_negative: false,
            allow_ago: None,
        }
    }
}

/// TODO: DOCUMENT
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ConfigBuilder<'a> {
    config: Config<'a>,
}

impl<'a> ConfigBuilder<'a> {
    /// TODO: DOCUMENT
    pub const fn new() -> Self {
        Self {
            config: Config::new(),
        }
    }

    /// TODO: DOCUMENT
    pub const fn build(self) -> Config<'a> {
        self.config
    }

    /// TODO: DOCUMENT
    pub const fn allow_delimiter(mut self, delimiter: Delimiter) -> Self {
        self.config.allow_delimiter = Some(delimiter);
        self
    }

    /// TODO: DOCUMENT
    pub const fn default_unit(mut self, time_unit: TimeUnit) -> Self {
        self.config.default_unit = time_unit;
        self
    }

    /// TODO: DOCUMENT
    pub const fn disable_exponent(mut self) -> Self {
        self.config.disable_exponent = true;
        self
    }

    /// TODO: DOCUMENT
    pub const fn disable_fraction(mut self) -> Self {
        self.config.disable_fraction = true;
        self
    }

    /// TODO: DOCUMENT
    pub const fn disable_infinity(mut self) -> Self {
        self.config.disable_infinity = true;
        self
    }

    /// TODO: DOCUMENT
    pub const fn number_is_optional(mut self) -> Self {
        self.config.number_is_optional = true;
        self
    }

    /// TODO: DOCUMENT
    pub const fn allow_negative(mut self) -> Self {
        self.config.allow_negative = true;
        self
    }

    /// TODO: DOCUMENT
    pub const fn parse_multiple(
        mut self,
        delimiter: Delimiter,
        conjunctions: Option<&'a [&'a str]>,
    ) -> Self {
        self.config.parse_multiple_delimiter = Some(delimiter);
        self.config.parse_multiple_conjunctions = conjunctions;
        self
    }

    /// TODO: DOCUMENT
    pub const fn allow_ago(mut self, delimiter: Delimiter) -> Self {
        self.config.allow_ago = Some(delimiter);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_for_config() {
        assert_eq!(Config::default(), Config::new());
    }
}
