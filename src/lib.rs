// Copyright (c) 2023 Joining7943 <joining@posteo.de>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

// spell-checker:ignore Nanos Repr rstest fract

use std::cmp::Ordering;
use std::slice::Iter;
use std::time::Duration;

pub const NANOS_MAX: u32 = 999_999_999;
pub const SECONDS_MAX: u64 = u64::MAX;

#[derive(Debug)]
enum ParseError {
    Syntax,
    Overflow,
}

#[derive(Debug, PartialEq)]
enum TimeUnit {
    NanoSecond,
    MicroSecond,
    MilliSecond,
    Second,
    Minute,
    Hour,
    Day,
}

impl Default for TimeUnit {
    fn default() -> Self {
        TimeUnit::Second
    }
}

impl TryFrom<&[u8]> for TimeUnit {
    type Error = ParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        use TimeUnit::*;

        match value {
            b"ns" => Ok(NanoSecond),
            b"mms" => Ok(MicroSecond),
            b"ms" => Ok(MilliSecond),
            b"s" => Ok(Second),
            b"m" => Ok(Minute),
            b"h" => Ok(Hour),
            b"d" => Ok(Day),
            _ => Err(ParseError::Syntax),
        }
    }
}

const ALL_TIMEUNITS: [TimeUnit; 2] = [TimeUnit::NanoSecond, TimeUnit::MicroSecond];

impl TimeUnit {
    fn multiplier(&self) -> u64 {
        use TimeUnit::*;

        match self {
            NanoSecond => 9,
            MicroSecond => 6,
            MilliSecond => 3,
            Second => 1,
            Minute => 60,
            Hour => 3600,
            Day => 86400,
        }
    }
}

/// An intermediate representation of seconds.
///
/// Optionally append `usize` amount of zeroes.
struct Seconds<'a>(Option<&'a [u8]>, Option<&'a [u8]>, Option<usize>);

impl<'a> Seconds<'a> {
    const ZERO: Self = Seconds(None, None, None);

    fn parse(&self) -> Result<u64, ParseError> {
        let mut seconds: u64 = 0;
        // 20 is the number of digits of u64::MAX
        let num_zeroes = self.2.unwrap_or(0).min(20);

        for c in self
            .0
            .iter()
            .flat_map(|s| s.iter())
            .chain(self.1.iter().flat_map(|s| s.iter()))
            .chain((0..num_zeroes).map(|_| &0u8))
        {
            match seconds
                .checked_mul(10)
                .and_then(|s| s.checked_add(*c as u64))
            {
                Some(s) => seconds = s,
                None => {
                    return Err(ParseError::Overflow);
                }
            }
        }

        Ok(seconds)
    }
}

/// An intermediate representation of nano seconds.
///
/// Optionally prepend `usize` amount of zeroes.
struct Nanos<'a>(Option<usize>, Option<&'a [u8]>, Option<&'a [u8]>);

impl<'a> Nanos<'a> {
    const ZERO: Self = Nanos(None, None, None);

    fn parse(&self) -> u32 {
        let mut multi = 100_000_000;
        let mut nanos: u32 = 0;

        // 9 is the number of digits of nano seconds
        let num_zeroes = self.0.unwrap_or(0).min(9);

        for c in (0..num_zeroes)
            .map(|_| &0)
            .chain(self.1.iter().flat_map(|s| s.iter()))
            .chain(self.2.iter().flat_map(|s| s.iter()))
            .take(9)
        {
            nanos += *c as u32 * multi;
            multi /= 10;
        }

        nanos
    }
}

#[derive(Debug, Default)]
struct DurationRepr {
    is_negative: bool,
    is_infinite: bool,
    whole: Option<Vec<u8>>,
    fract: Option<Vec<u8>>,
    exponent: i16,
    unit: TimeUnit,
}

impl DurationRepr {
    fn parse(&mut self) -> Result<Duration, ParseError> {
        if self.is_infinite {
            if self.is_negative {
                return Err(ParseError::Syntax);
            } else {
                return Ok(Duration::MAX);
            }
        }

        let (whole, fract) = match (self.whole.take(), self.fract.take()) {
            (None, None) => return Err(ParseError::Syntax),
            (None, Some(fract)) => (vec![], fract),
            (Some(whole), None) => (whole, vec![]),
            (Some(whole), Some(fract)) => (whole, fract),
        };

        // The maximum absolute value of the exponent is `1023`, so it is safe to cast to usize
        let exponent_abs: usize = self.exponent.unsigned_abs().into();

        // We're operating on slices to minimize runtime costs. Applying the exponent before parsing
        // to integers is necessary, since the exponent can move digits into the to be considered
        // final integer domain.
        let (seconds, nanos) = match self.exponent.cmp(&0) {
            Ordering::Less if whole.len() > exponent_abs => {
                let seconds = Seconds(Some(&whole[..whole.len() - exponent_abs]), None, None);
                let nanos = Nanos(
                    None,
                    Some(&whole[whole.len() - exponent_abs..]),
                    Some(&fract),
                );
                (seconds, nanos)
            }
            Ordering::Less => {
                let seconds = Seconds::ZERO;
                let nanos = Nanos(Some(exponent_abs - whole.len()), Some(&whole), Some(&fract));
                (seconds, nanos)
            }
            Ordering::Equal => {
                let seconds = Seconds(Some(&whole), None, None);
                let nanos = Nanos(None, Some(&fract), None);
                (seconds, nanos)
            }
            Ordering::Greater if fract.len() > exponent_abs => {
                let seconds = Seconds(Some(&whole), Some(&fract[..exponent_abs]), None);
                let nanos = Nanos(None, Some(&fract[exponent_abs..]), None);
                (seconds, nanos)
            }
            Ordering::Greater => {
                let seconds = Seconds(Some(&whole), Some(&fract), Some(exponent_abs - fract.len()));
                let nanos = Nanos::ZERO;
                (seconds, nanos)
            }
        };

        // Finally, parse the seconds and nano seconds and interpret a seconds overflow as
        // maximum `Duration`.
        let (seconds, nanos) = match seconds.parse() {
            Ok(seconds) => (seconds, nanos.parse()),
            Err(ParseError::Overflow) => (SECONDS_MAX, NANOS_MAX),
            Err(_) => unreachable!(), // only ParseError::Overflow is returned by `Seconds::parse`
        };

        // allow `-0` or `-0.0` and interpret as plain `0`
        if self.is_negative && seconds == 0 && nanos == 0 {
            Ok(Duration::ZERO)
        } else if self.is_negative {
            Err(ParseError::Syntax)
        } else {
            Ok(Duration::new(seconds, nanos))
        }
    }
}
struct DurationParser<'a> {
    current_byte: Option<&'a u8>,
    iterator: Iter<'a, u8>,
}

/// Parse a source string into a [`DurationRepr`].
impl<'a> DurationParser<'a> {
    fn new(input: &'a str) -> Self {
        let mut iterator = input.as_bytes().iter();
        Self {
            current_byte: iterator.next(),
            iterator,
        }
    }

    fn advance(&mut self) {
        self.current_byte = self.iterator.next();
    }

    fn parse(&mut self) -> Result<DurationRepr, ParseError> {
        let mut duration_repr = DurationRepr::default();

        // parse the sign if present
        if self.parse_sign_is_negative()? {
            duration_repr.is_negative = true;
        }

        // parse infinity or the whole number part of the input
        match self.current_byte {
            Some(byte) if *byte == b'i' || *byte == b'I' => {
                self.parse_infinity()?;
                duration_repr.is_infinite = true;
                return Ok(duration_repr);
            }
            Some(byte) if byte.is_ascii_digit() => {
                // the maximum number of digits that need to be considered:
                // max(-exponent) = 1022 + max_digits(u64::MAX) = 20 + 1
                let whole = self.parse_digits(1043)?;
                duration_repr.whole = Some(whole);
            }
            Some(byte) if *byte == b'.' => {}
            Some(_) | None => return Err(ParseError::Syntax),
        }

        // parse the fraction number part of the input
        match self.current_byte {
            Some(byte) if *byte == b'.' => {
                self.advance();
                let fract = match self.current_byte {
                    // the maximum number of digits that need to be considered:
                    // max(+exponent) = 1023 + max_digits(nano seconds) = 9 + 1
                    Some(byte) if byte.is_ascii_digit() => Some(self.parse_digits(1033)?),
                    Some(byte) if byte.eq_ignore_ascii_case(&b'e') => None,
                    Some(_) => return Err(ParseError::Syntax),
                    None => return Ok(duration_repr),
                };
                duration_repr.fract = fract;
            }
            Some(byte) if byte.eq_ignore_ascii_case(&b'e') => {}
            Some(_) => return Err(ParseError::Syntax),
            None => return Ok(duration_repr),
        }

        // parse the exponent of the input if present
        match self.current_byte {
            Some(byte) if byte.eq_ignore_ascii_case(&b'e') => {
                self.advance();
                let exponent = self.parse_exponent()?;
                duration_repr.exponent = exponent;
            }
            Some(_) => return Err(ParseError::Syntax),
            None => return Ok(duration_repr),
        }

        match self.current_byte {
            Some(_) => {
                let unit = self.parse_time_unit()?;
                duration_repr.unit = unit;
            }
            None => return Ok(duration_repr),
        }

        // check we've reached the end of input
        match self.current_byte {
            Some(_) => Err(ParseError::Syntax),
            None => Ok(duration_repr),
        }
    }

    fn parse_time_unit(&mut self) -> Result<TimeUnit, ParseError> {
        let mut max_bytes = 3;
        let mut bytes = Vec::<u8>::with_capacity(max_bytes);
        while let Some(byte) = self.current_byte {
            if max_bytes != 0 {
                bytes.push(*byte);
                self.advance();
                max_bytes -= 1;
            } else {
                break;
            }
        }

        TimeUnit::try_from(bytes.as_slice())
    }

    fn parse_digits(&mut self, mut max: usize) -> Result<Vec<u8>, ParseError> {
        // Using `size_hint()` is a rough (but always correct) estimation for an upper bound.
        // However, using maybe more memory than needed spares the costly memory reallocations and
        // maximum memory used is just around `1kB` per parsed number.
        let capacity = max.min(self.iterator.size_hint().1.unwrap() + 1);
        let mut digits = Vec::<u8>::with_capacity(capacity);

        while let Some(byte) = self.current_byte {
            let digit = byte.wrapping_sub(b'0');
            if digit < 10 {
                if max > 0 {
                    digits.push(digit);
                    max -= 1;
                }
                self.advance();
            } else {
                break;
            }
        }

        if digits.is_empty() {
            Err(ParseError::Syntax)
        } else {
            Ok(digits)
        }
    }

    fn parse_infinity(&mut self) -> Result<(), ParseError> {
        let expected = [b'i', b'n', b'f', b'i', b'n', b'i', b't', b'y'];
        for (pos, byte) in expected.iter().enumerate() {
            match self.current_byte {
                Some(current) if current.eq_ignore_ascii_case(byte) => self.advance(),
                Some(_) => return Err(ParseError::Syntax), // wrong character
                None if pos == 3 => return Ok(()),         // short `inf` is allowed
                None => return Err(ParseError::Syntax),    // premature end of input
            }
        }

        // assure we've reached the end of input
        if self.current_byte.is_none() {
            Ok(())
        } else {
            Err(ParseError::Syntax)
        }
    }

    /// Parse and consume the sign if present. Return true if sign is negative.
    fn parse_sign_is_negative(&mut self) -> Result<bool, ParseError> {
        match self.current_byte {
            Some(byte) if *byte == b'+' => {
                self.advance();
                Ok(false)
            }
            Some(byte) if *byte == b'-' => {
                self.advance();
                Ok(true)
            }
            Some(_) => Ok(false),
            None => Err(ParseError::Syntax),
        }
    }

    fn parse_exponent(&mut self) -> Result<i16, ParseError> {
        let is_negative = self.parse_sign_is_negative()?;

        let mut exponent = 0i16;
        while let Some(byte) = self.current_byte {
            let digit = byte.wrapping_sub(b'0');
            if digit < 10 {
                exponent = exponent * 10 + digit as i16;
                if (is_negative && exponent <= 1022) || (!is_negative && exponent <= 1023) {
                    self.advance();
                } else {
                    return Err(ParseError::Overflow);
                }
            } else {
                break;
            }
        }

        Ok(if is_negative { -exponent } else { exponent })
    }
}

/// Parse a string into a [`Duration`] by accepting a source string similar to floating point.
///
/// No whitespace is allowed in the source string. By parsing directly into a `u64` for the whole
/// number part (the [`Duration`] seconds) and `u32` for the fraction part (the [`Duration`] nano
/// seconds), we avoid the possibly lossy intermediate conversion to a `f64` and can represent the
/// exact user input as `Duration`. We can also represent valid durations, which
/// [`Duration::from_secs_f64`] can not parse without errors, like `format!("{}.0", u64::MAX)`. The
/// accepted grammar is (closely related to [`f64::from_str`]):
///
/// ```text
/// Duration ::= Sign? ( 'inf' | 'infinity' | Number )
/// Number   ::= ( Digit+ |
///                Digit+ '.' Digit* |
///                Digit* '.' Digit+ ) Exp?
/// Exp      ::= [eE] Sign? Digit+
/// Sign     ::= [+-]
/// Digit    ::= [0-9]
/// ```
///
/// The parsed [`Duration`] saturates at `seconds == u64::MAX`, `nanos (max) == .999999999` and is
/// bounded below at `nanos (min if not 0) == .000000001`. Infinity values like `inf`, `+infinity`
/// etc. are valid input and resolve to `Duration::MAX`.
///
/// # Performance
///
/// Parsing the exact representation comes with a small performance loss. `parse_duration` is around
/// 2-5 times slower than parsing to `f64 and then `Duration::from_secs_f64` (depending on the
/// length of the input string). But, total computation time for usual input like (`+1.0e2`) is
/// still in the `100 - 300` nanoseconds domain (on a `4-core ~3000Mhz` processor).
///
/// # Errors
///
/// This function will return an error when parsing fails, the `src` was negative (`-0.0` counts as
/// not negative) or the exponent wasn't in the allowed range (`-1022..=1023`).
///
/// # Examples
///
/// ```ignore
/// use std::time::Duration;
///
/// let duration = parse_duration("+1.09e1").unwrap();
/// assert_eq!(duration, Duration::new(10, 900_000_000));
/// ```
///
/// [`f64::from_str`]: https://doc.rust-lang.org/std/primitive.f64.html#method.from_str
pub fn parse_duration(string: &str) -> Result<Duration, String> {
    let mut parser = DurationParser::new(string);
    parser
        .parse()
        .and_then(|mut repr| repr.parse())
        .map_err(|_| "Error parsing duration".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::empty_string("")]
    #[case::leading_whitespace("  1")]
    #[case::trailing_whitespace("1   ")]
    #[case::only_whitespace("  \t\n")]
    #[case::only_point(".")]
    #[case::only_sign("+")]
    #[case::only_exponent("e-10")]
    #[case::sign_with_exponent("-e1")]
    #[case::sign_with_point_and_exponent("-.e1")]
    #[case::negative_seconds("-1")]
    #[case::negative_seconds_with_fraction("-1.0")]
    #[case::negative_nano_seconds("-0.000000001")]
    #[should_panic]
    fn test_parse_duration_with_illegal_argument_then_error(#[case] source: &str) {
        parse_duration(source).unwrap();
    }

    #[rstest]
    #[case::simple_zero("0", Duration::ZERO)]
    #[case::zero_point_zero("0.0", Duration::ZERO)]
    #[case::point_zero(".0", Duration::ZERO)]
    #[case::zero_point("0.", Duration::ZERO)]
    #[case::simple_number("1", Duration::new(1, 0))]
    #[case::one_with_fraction_number("1.1", Duration::new(1, 100_000_000))]
    #[case::leading_zero_max_nanos("0.999999999", Duration::new(0, 999_999_999))]
    #[case::leading_number_max_nanos("1.999999999", Duration::new(1, 999_999_999))]
    #[case::simple_number("1234.123456789", Duration::new(1234, 123_456_789))]
    #[case::max_seconds(&u64::MAX.to_string(), Duration::new(u64::MAX, 0))]
    #[case::leading_zeros("000000100", Duration::new(100, 0))]
    #[case::leading_zeros_with_fraction("00000010.0", Duration::new(10, 0))]
    #[case::trailing_zeros("10.010000000", Duration::new(10, 10_000_000))]
    fn test_parse_duration_when_simple_arguments_are_valid(
        #[case] source: &str,
        #[case] expected: Duration,
    ) {
        let duration = parse_duration(source).unwrap();
        assert_eq!(duration, expected);
    }

    #[rstest]
    #[case::zero("1.1e0", Duration::new(1, 100_000_000))]
    #[case::negative_zero("1.1e-0", Duration::new(1, 100_000_000))]
    #[case::simple("1.09e1", Duration::new(10, 900_000_000))]
    #[case::simple_big_e("1.09E1", Duration::new(10, 900_000_000))]
    #[case::lower_than_nanos_min("0.0000000001e1", Duration::new(0, 1))]
    #[case::higher_than_seconds_max(&format!("{}9.999999999e-1", u64::MAX), Duration::MAX)]
    #[case::plus_sign("0.1000000001e+1", Duration::new(1, 1))]
    #[case::minus_sign_whole_to_fract("1.00000001e-1", Duration::new(0, 100_000_001))]
    #[case::minus_sign_zero_to_fract("10.00000001e-1", Duration::new(1, 1))]
    #[case::no_overflow_error_low("1.0e-1022", Duration::ZERO)]
    #[case::no_overflow_error_high("1.0e1023", Duration::MAX)]
    #[case::maximum_amount_of_seconds_digits_no_overflow(&format!("{}.0e-1022", "1".repeat(1042)), Duration::new(11_111_111_111_111_111_111, 111_111_111))]
    #[case::more_than_maximum_amount_of_seconds_digits_then_maximum_duration(&format!("{}.0e-1022", "1".repeat(1043)), Duration::MAX)]
    #[case::amount_of_nano_seconds_digits_then_capped(&format!("0.{}9e+1023", "0".repeat(1032)), Duration::ZERO)]
    #[case::maximum_amount_of_nano_seconds_digits_then_not_capped(&format!("0.{}9e+1023", "0".repeat(1031)), Duration::new(0, 9))]
    fn test_parse_duration_when_arguments_contain_exponent(
        #[case] source: &str,
        #[case] expected: Duration,
    ) {
        let duration = parse_duration(source).unwrap();
        assert_eq!(duration, expected);
    }

    #[rstest]
    #[case::no_number("1e")]
    #[case::invalid_number("1e+F")]
    #[case::exponent_overflow_error_high("1e1024")]
    #[case::exponent_overflow_error_low("1e-1023")]
    #[case::exponent_parse_i16_overflow_error(&format!("1e{}", i16::MIN as i32 - 1))]
    #[should_panic]
    fn test_parse_duration_when_arguments_with_illegal_exponent_then_error(#[case] source: &str) {
        parse_duration(source).unwrap();
    }

    #[rstest]
    #[case::no_rounding("1.99999999999999999", Duration::new(1, 999_999_999))]
    #[case::high_value_no_swallow_fract(&format!("{}.1", u64::MAX),Duration::new(u64::MAX, 100_000_000) )]
    fn test_parse_duration_when_precision_of_float_would_be_insufficient_then_still_parse_exact(
        #[case] source: &str,
        #[case] expected: Duration,
    ) {
        let duration = parse_duration(source).unwrap();
        assert_eq!(duration, expected);
    }

    #[rstest]
    #[case::lower_than_min_nanos("1.0000000001", Duration::new(1, 0))]
    #[case::max_digits_of_nanos("1.99999999999", Duration::new(1, 999_999_999))]
    #[case::higher_than_max_seconds(&format!("{}", u64::MAX as u128 + 1), Duration::MAX)]
    #[case::higher_than_max_seconds_with_fraction(&format!("{}.0", u64::MAX as u128 + 1), Duration::MAX)]
    fn test_parse_duration_when_arguments_are_capped_then_max_duration_or_min_nanos(
        #[case] source: &str,
        #[case] expected: Duration,
    ) {
        let duration = parse_duration(source).unwrap();
        assert_eq!(duration, expected);
    }

    #[rstest]
    #[case::plus_zero("+0", Duration::ZERO)]
    #[case::plus_zero_with_fraction("+0.0", Duration::ZERO)]
    #[case::minus_zero("-0", Duration::ZERO)]
    #[case::minus_zero_with_fraction("-0.0", Duration::ZERO)]
    #[case::plus_one_with_fraction("+1.0", Duration::new(1, 0))]
    fn test_parse_duration_when_arguments_have_a_sign(
        #[case] source: &str,
        #[case] expected: Duration,
    ) {
        let duration = parse_duration(source).unwrap();
        assert_eq!(duration, expected);
    }

    #[rstest]
    #[case::infinity_short("inf")]
    #[case::infinity_short_case_insensitive("iNf")]
    #[case::infinity_long("infinity")]
    #[case::infinity_long_case_insensitive("InfiNitY")]
    fn test_parse_duration_when_arguments_are_infinity_values(#[case] source: &str) {
        let duration = parse_duration(source).unwrap();
        assert_eq!(duration, Duration::MAX);
    }

    #[rstest]
    #[case::negative_infinity_short("-inf")]
    #[case::negative_infinity_long("-infinity")]
    #[case::incomplete_infinity("infin")]
    #[case::infinity_with_number("inf1.0")]
    #[should_panic]
    fn test_parse_duration_when_arguments_are_illegal_infinity_values_then_error(
        #[case] source: &str,
    ) {
        parse_duration(source).unwrap();
    }
}
