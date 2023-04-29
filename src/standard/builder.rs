// Copyright (c) 2023 Joining7943 <joining@posteo.de>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use super::time_units::TimeUnits;
use crate::config::Config;
use crate::parse::Parser;
use crate::{Delimiter, DurationParser, TimeUnit};

#[derive(Debug, PartialEq, Eq)]
enum TimeUnitsChoice<'a> {
    Default,
    All,
    None,
    Custom(&'a [TimeUnit]),
}

/// An ergonomic builder for a [`DurationParser`].
///
/// The [`DurationParserBuilder`] is more ergonomic in some use cases than using
/// [`DurationParser`] directly, especially when using the `DurationParser` for parsing multiple
/// inputs.
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
///
/// use fundu::TimeUnit::*;
/// use fundu::{DurationParser, DurationParserBuilder};
///
/// let parser = DurationParserBuilder::new()
///     .all_time_units()
///     .default_unit(MicroSecond)
///     .allow_delimiter(|byte| byte == b' ')
///     .build();
///
/// assert_eq!(parser.parse("1   ns").unwrap(), Duration::new(0, 1));
/// assert_eq!(parser.parse("1").unwrap(), Duration::new(0, 1_000));
///
/// // instead of
///
/// let mut parser = DurationParser::with_all_time_units();
/// parser
///     .default_unit(MicroSecond)
///     .allow_delimiter(Some(|byte| byte == b' '));
///
/// assert_eq!(parser.parse("1    ns").unwrap(), Duration::new(0, 1));
/// assert_eq!(parser.parse("1").unwrap(), Duration::new(0, 1_000));
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct DurationParserBuilder<'a> {
    time_units_choice: TimeUnitsChoice<'a>,
    config: Config,
}

impl<'a> Default for DurationParserBuilder<'a> {
    /// Construct a new [`DurationParserBuilder`] without any time units.
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> DurationParserBuilder<'a> {
    /// Construct a new reusable [`DurationParserBuilder`].
    ///
    /// This method is the same like invoking [`DurationParserBuilder::default`]. Per default
    /// there are no time units configured in the builder. Use one of
    ///
    /// * [`DurationParserBuilder::default_time_units`]
    /// * [`DurationParserBuilder::all_time_units`]
    /// * [`DurationParserBuilder::custom_time_units`]
    ///
    /// to add time units.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fundu::{DurationParser, DurationParserBuilder};
    ///
    /// assert_eq!(
    ///     DurationParserBuilder::new().build(),
    ///     DurationParser::without_time_units()
    /// );
    /// ```
    pub const fn new() -> Self {
        Self {
            time_units_choice: TimeUnitsChoice::None,
            config: Config::new(),
        }
    }

    /// Configure [`DurationParserBuilder`] to build the [`DurationParser`] with default time
    /// units.
    ///
    /// Setting the time units with this method overwrites any previously made choices with
    ///
    /// * [`DurationParserBuilder::all_time_units`]
    /// * [`DurationParserBuilder::custom_time_units`]
    ///
    /// The default time units with their identifiers are:
    ///
    /// | [`TimeUnit`]    | default id
    /// | --------------- | ----------:
    /// | Nanosecond  |         ns
    /// | Microsecond |         Ms
    /// | Millisecond |         ms
    /// | Second      |          s
    /// | Minute      |          m
    /// | Hour        |          h
    /// | Day         |          d
    /// | Week        |          w
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fundu::DurationParserBuilder;
    /// use fundu::TimeUnit::*;
    ///
    /// assert_eq!(
    ///     DurationParserBuilder::new()
    ///         .default_time_units()
    ///         .build()
    ///         .get_current_time_units(),
    ///     vec![
    ///         NanoSecond,
    ///         MicroSecond,
    ///         MilliSecond,
    ///         Second,
    ///         Minute,
    ///         Hour,
    ///         Day,
    ///         Week
    ///     ]
    /// );
    /// ```
    pub fn default_time_units(&mut self) -> &mut Self {
        self.time_units_choice = TimeUnitsChoice::Default;
        self
    }

    /// Configure [`DurationParserBuilder`] to build the [`DurationParser`] with all time units.
    ///
    /// Setting the time units with this method overwrites any previously made choices with
    ///
    /// * [`DurationParserBuilder::default_time_units`]
    /// * [`DurationParserBuilder::custom_time_units`]
    ///
    /// The time units with their identifiers are:
    ///
    /// | [`TimeUnit`]    | default id
    /// | --------------- | ----------:
    /// | Nanosecond  |         ns
    /// | Microsecond |         Ms
    /// | Millisecond |         ms
    /// | Second      |          s
    /// | Minute      |          m
    /// | Hour        |          h
    /// | Day         |          d
    /// | Week        |          w
    /// | Month       |          M
    /// | Year        |          y
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fundu::DurationParserBuilder;
    /// use fundu::TimeUnit::*;
    ///
    /// assert_eq!(
    ///     DurationParserBuilder::new()
    ///         .all_time_units()
    ///         .build()
    ///         .get_current_time_units(),
    ///     vec![
    ///         NanoSecond,
    ///         MicroSecond,
    ///         MilliSecond,
    ///         Second,
    ///         Minute,
    ///         Hour,
    ///         Day,
    ///         Week,
    ///         Month,
    ///         Year
    ///     ]
    /// );
    /// ```
    pub fn all_time_units(&mut self) -> &mut Self {
        self.time_units_choice = TimeUnitsChoice::All;
        self
    }

    /// Configure the [`DurationParserBuilder`] to build the [`DurationParser`] with a custom set
    /// of time units.
    ///
    /// Setting the time units with this method overwrites any previously made choices with
    ///
    /// * [`DurationParserBuilder::default_time_units`]
    /// * [`DurationParserBuilder::all_time_units`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fundu::DurationParserBuilder;
    /// use fundu::TimeUnit::*;
    ///
    /// assert_eq!(
    ///     DurationParserBuilder::new()
    ///         .custom_time_units(&[NanoSecond, Second, Year])
    ///         .build()
    ///         .get_current_time_units(),
    ///     vec![NanoSecond, Second, Year]
    /// );
    /// ```
    pub fn custom_time_units(&mut self, time_units: &'a [TimeUnit]) -> &mut Self {
        self.time_units_choice = TimeUnitsChoice::Custom(time_units);
        self
    }

    /// Set the default time unit to something different than [`TimeUnit::Second`]
    ///
    /// See also [`DurationParser::default_unit`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    ///
    /// use fundu::DurationParserBuilder;
    /// use fundu::TimeUnit::*;
    ///
    /// assert_eq!(
    ///     DurationParserBuilder::new()
    ///         .all_time_units()
    ///         .default_unit(NanoSecond)
    ///         .build()
    ///         .parse("42")
    ///         .unwrap(),
    ///     Duration::new(0, 42)
    /// );
    /// ```
    pub fn default_unit(&mut self, unit: TimeUnit) -> &mut Self {
        self.config.default_unit = unit;
        self
    }

    /// Allow one or more delimiters between the number and the [`TimeUnit`].
    ///
    /// See also [`DurationParser::allow_delimiter`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    ///
    /// use fundu::DurationParserBuilder;
    ///
    /// let parser = DurationParserBuilder::new()
    ///     .default_time_units()
    ///     .allow_delimiter(|byte| byte.is_ascii_whitespace())
    ///     .build();
    ///
    /// assert_eq!(parser.parse("123 \t\n\x0C\rns"), Ok(Duration::new(0, 123)));
    /// assert_eq!(parser.parse("123\n"), Ok(Duration::new(123, 0)));
    /// ```
    pub fn allow_delimiter(&mut self, delimiter: Delimiter) -> &mut Self {
        self.config.allow_delimiter = Some(delimiter);
        self
    }

    /// Disable parsing an exponent.
    ///
    /// See also [`DurationParser::disable_exponent`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fundu::{DurationParserBuilder, ParseError};
    ///
    /// assert_eq!(
    ///     DurationParserBuilder::new()
    ///         .default_time_units()
    ///         .disable_exponent()
    ///         .build()
    ///         .parse("123e+1"),
    ///     Err(ParseError::Syntax(3, "No exponent allowed".to_string()))
    /// );
    /// ```
    pub fn disable_exponent(&mut self) -> &mut Self {
        self.config.disable_exponent = true;
        self
    }

    /// Disable parsing a fraction in the source string.
    ///
    /// See also [`DurationParser::disable_fraction`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    ///
    /// use fundu::{DurationParserBuilder, ParseError};
    ///
    /// let parser = DurationParserBuilder::new()
    ///     .default_time_units()
    ///     .disable_fraction()
    ///     .build();
    ///
    /// assert_eq!(
    ///     parser.parse("123.456"),
    ///     Err(ParseError::Syntax(3, "No fraction allowed".to_string()))
    /// );
    ///
    /// assert_eq!(parser.parse("123e-2"), Ok(Duration::new(1, 230_000_000)));
    /// assert_eq!(parser.parse("123ns"), Ok(Duration::new(0, 123)));
    /// ```
    pub fn disable_fraction(&mut self) -> &mut Self {
        self.config.disable_fraction = true;
        self
    }

    /// Disable parsing infinity values
    ///
    /// See also [`DurationParser::disable_infinity`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fundu::{DurationParserBuilder, ParseError};
    ///
    /// let parser = DurationParserBuilder::new().disable_infinity().build();
    ///
    /// assert_eq!(
    ///     parser.parse("inf"),
    ///     Err(ParseError::Syntax(0, format!("Invalid input: 'inf'")))
    /// );
    /// assert_eq!(
    ///     parser.parse("infinity"),
    ///     Err(ParseError::Syntax(0, format!("Invalid input: 'infinity'")))
    /// );
    /// assert_eq!(
    ///     parser.parse("+inf"),
    ///     Err(ParseError::Syntax(1, format!("Invalid input: 'inf'")))
    /// );
    /// ```
    pub fn disable_infinity(&mut self) -> &mut Self {
        self.config.disable_infinity = true;
        self
    }

    /// This setting makes a number in the source string optional.
    ///
    /// See also [`DurationParser::number_is_optional`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    ///
    /// use fundu::DurationParserBuilder;
    ///
    /// let parser = DurationParserBuilder::new()
    ///     .default_time_units()
    ///     .number_is_optional()
    ///     .build();
    ///
    /// for input in &["ns", "e-9", "e-3Ms"] {
    ///     assert_eq!(parser.parse(input), Ok(Duration::new(0, 1)));
    /// }
    /// ```
    pub fn number_is_optional(&mut self) -> &mut Self {
        self.config.number_is_optional = true;
        self
    }

    /// Parse possibly multiple durations and sum them up.
    ///
    /// See also [`DurationParser::parse_multiple`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    ///
    /// use fundu::DurationParserBuilder;
    ///
    /// let parser = DurationParserBuilder::new()
    ///     .default_time_units()
    ///     .parse_multiple(|byte| matches!(byte, b' ' | b'\t'))
    ///     .build();
    ///
    /// assert_eq!(parser.parse("1.5h 2e+2ns"), Ok(Duration::new(5400, 200)));
    /// assert_eq!(parser.parse("55s500ms"), Ok(Duration::new(55, 500_000_000)));
    /// assert_eq!(parser.parse("1\t1"), Ok(Duration::new(2, 0)));
    /// assert_eq!(parser.parse("1.   .1"), Ok(Duration::new(1, 100_000_000)));
    /// assert_eq!(parser.parse("2h"), Ok(Duration::new(2 * 60 * 60, 0)));
    /// assert_eq!(
    ///     parser.parse("300ms20s 5d"),
    ///     Ok(Duration::new(5 * 60 * 60 * 24 + 20, 300_000_000))
    /// );
    /// ```
    pub fn parse_multiple(&mut self, delimiter: Delimiter) -> &mut Self {
        self.config.parse_multiple = Some(delimiter);
        self
    }

    /// Finally, build the [`DurationParser`] from this builder.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    ///
    /// use fundu::DurationParserBuilder;
    ///
    /// let parser = DurationParserBuilder::new().default_time_units().build();
    /// for input in &["1m", "60s"] {
    ///     assert_eq!(parser.parse(input).unwrap(), Duration::new(60, 0))
    /// }
    /// ```
    pub fn build(&mut self) -> DurationParser {
        let parser = Parser::with_config(self.config.clone());

        match self.time_units_choice {
            TimeUnitsChoice::Default => DurationParser {
                time_units: TimeUnits::with_default_time_units(),
                inner: parser,
            },
            TimeUnitsChoice::All => DurationParser {
                time_units: TimeUnits::with_all_time_units(),
                inner: parser,
            },
            TimeUnitsChoice::None => DurationParser {
                time_units: TimeUnits::new(),
                inner: parser,
            },
            TimeUnitsChoice::Custom(time_units) => DurationParser {
                time_units: TimeUnits::with_time_units(time_units),
                inner: parser,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::config::Config;
    use crate::TimeUnit::*;

    #[test]
    fn test_duration_parser_builder_when_new() {
        let builder = DurationParserBuilder::new();
        assert_eq!(builder.config, Config::new());
        assert_eq!(builder.time_units_choice, TimeUnitsChoice::None);
    }

    #[test]
    fn test_duration_parser_builder_when_default_time_units() {
        let mut builder = DurationParserBuilder::new();
        builder.default_time_units();
        assert_eq!(builder.time_units_choice, TimeUnitsChoice::Default);
    }

    #[test]
    fn test_duration_parser_builder_when_all_time_units() {
        let mut builder = DurationParserBuilder::new();
        builder.all_time_units();
        assert_eq!(builder.time_units_choice, TimeUnitsChoice::All);
    }

    #[test]
    fn test_duration_parser_builder_when_custom_time_units() {
        let mut builder = DurationParserBuilder::new();
        builder.custom_time_units(&[MicroSecond, Hour, Week, Year]);
        assert_eq!(
            builder.time_units_choice,
            TimeUnitsChoice::Custom(&[MicroSecond, Hour, Week, Year])
        );
    }

    #[test]
    fn test_duration_parser_builder_when_default_unit() {
        let mut expected = Config::new();
        expected.default_unit = MicroSecond;

        let mut builder = DurationParserBuilder::new();
        builder.default_unit(MicroSecond);

        assert_eq!(builder.config, expected);
    }

    #[test]
    fn test_duration_parser_builder_when_allow_delimiter() {
        let mut builder = DurationParserBuilder::new();
        builder.allow_delimiter(|b| b == b' ');

        assert!(builder.config.allow_delimiter.unwrap()(b' '));
    }

    #[test]
    fn test_duration_parser_builder_when_disable_fraction() {
        let mut expected = Config::new();
        expected.disable_fraction = true;

        let mut builder = DurationParserBuilder::new();
        builder.disable_fraction();

        assert_eq!(builder.config, expected);
    }

    #[test]
    fn test_duration_parser_builder_when_disable_exponent() {
        let mut expected = Config::new();
        expected.disable_exponent = true;

        let mut builder = DurationParserBuilder::new();
        builder.disable_exponent();

        assert_eq!(builder.config, expected);
    }

    #[test]
    fn test_duration_parser_builder_when_disable_infinity() {
        let mut expected = Config::new();
        expected.disable_infinity = true;

        let mut builder = DurationParserBuilder::new();
        builder.disable_infinity();

        assert_eq!(builder.config, expected);
    }

    #[test]
    fn test_duration_parser_builder_when_number_is_optional() {
        let mut expected = Config::new();
        expected.number_is_optional = true;

        let mut builder = DurationParserBuilder::new();
        builder.number_is_optional();

        assert_eq!(builder.config, expected);
    }

    #[test]
    fn test_duration_parser_builder_when_parse_multiple() {
        let mut builder = DurationParserBuilder::new();
        builder.parse_multiple(|byte: u8| byte == 0xff);

        assert!(builder.config.parse_multiple.unwrap()(0xff));
    }

    #[rstest]
    #[case::default_time_units(TimeUnitsChoice::Default, DurationParser::new())]
    #[case::all_time_units(TimeUnitsChoice::All, DurationParser::with_all_time_units())]
    #[case::no_time_units(TimeUnitsChoice::None, DurationParser::without_time_units())]
    #[case::custom_time_units(
            TimeUnitsChoice::Custom(&[NanoSecond, Minute]),
            DurationParser::with_time_units(&[NanoSecond, Minute])
    )]
    fn test_duration_parser_builder_build(
        #[case] choice: TimeUnitsChoice,
        #[case] expected: DurationParser,
    ) {
        let mut builder = DurationParserBuilder::new();
        builder.time_units_choice = choice;

        assert_eq!(builder.build(), expected);
    }
}
