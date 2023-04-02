// Copyright (c) 2023 Joining7943 <joining@posteo.de>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use super::repr::{DurationRepr, Fract, Whole};
use crate::config::Config;
use crate::time::TimeUnitsLike;
use crate::{Delimiter, Multiplier, ParseError, TimeUnit};

pub(crate) struct ReprParser<'a> {
    current_pos: usize, // keep first. Has better performance.
    current_byte: Option<&'a u8>,
    config: &'a Config,
    time_units: &'a dyn TimeUnitsLike,
    input: &'a [u8],
}

/// Parse a source string into a [`DurationRepr`].
impl<'a> ReprParser<'a> {
    pub fn new(input: &'a str, config: &'a Config, time_units: &'a dyn TimeUnitsLike) -> Self {
        let input = input.as_bytes();
        Self {
            current_byte: input.first(),
            input,
            current_pos: 0,
            time_units,
            config,
        }
    }

    pub(super) fn parse(
        &'a mut self,
    ) -> Result<(DurationRepr, Option<&'a mut ReprParser>), ParseError> {
        if self.current_byte.is_none() {
            return Err(ParseError::Empty);
        }

        let Config {
            default_unit,
            default_multiplier: _,
            disable_exponent,
            disable_fraction,
            max_exponent,
            min_exponent,
            number_is_optional,
            allow_delimiter,
            disable_infinity,
            parse_multiple,
        } = *self.config;

        let mut duration_repr = DurationRepr {
            unit: default_unit,
            number_is_optional,
            ..Default::default()
        };

        // parse the sign if present
        if self.parse_sign_is_negative()? {
            duration_repr.is_negative = true;
        }

        // parse infinity or the whole number part of the input
        match self.current_byte {
            Some(byte) if byte.is_ascii_digit() => {
                // the maximum number of digits that need to be considered depending on the
                // exponent: max(-exponent) = abs(i16::MIN) + max_digits(u64::MAX) =
                // 20 + 9 (nano seconds) + 1 + alignment at modulo 8
                let max = ((min_exponent as isize).abs() + 32) as usize;

                // // Using `len()` is a rough (but always correct) estimation for an upper bound.
                // // However, using maybe more memory than needed spares the costly memory
                // reallocations
                duration_repr.digits = Some(Vec::with_capacity(
                    max.min(self.input.len() - self.current_pos),
                ));
                duration_repr.whole = duration_repr
                    .digits
                    .as_mut()
                    .map(|digits| self.parse_whole(digits));
            }
            Some(byte) if *byte == b'.' => {}
            Some(_)
                if !disable_infinity
                    && self
                        .peek(3)
                        .map_or(false, |bytes| bytes.eq_ignore_ascii_case(b"inf")) =>
            {
                // SAFETY: We just checked with peek() that there are at least 3 bytes
                unsafe { self.advance_by(3) }
                self.parse_infinity_remainder(parse_multiple)?;
                duration_repr.is_infinite = true;

                match self.current_byte {
                    Some(_) if parse_multiple.is_some() => return Ok((duration_repr, Some(self))),
                    Some(byte) => {
                        return Err(ParseError::Syntax(
                            self.current_pos,
                            format!("Expected end of input but found '{}'", *byte as char),
                        ));
                    }
                    None => return Ok((duration_repr, None)),
                }
            }
            Some(_) if number_is_optional => {}
            Some(_) => {
                // SAFETY: The input str is utf-8 and we have only parsed ascii characters so far
                return Err(ParseError::Syntax(
                    self.current_pos,
                    format!("Invalid input: '{}'", unsafe {
                        self.get_remainder_str_unchecked()
                    }),
                ));
            }
            None => {
                return Err(ParseError::Syntax(
                    self.current_pos,
                    "Unexpected end of input".to_string(),
                ));
            }
        }

        // parse the fraction number part of the input
        match self.current_byte {
            Some(byte) if *byte == b'.' && !disable_fraction => {
                self.advance();
                let fract = match self.current_byte {
                    Some(byte) if byte.is_ascii_digit() => {
                        let needed = self.input.len() - self.current_pos;
                        let digits = match duration_repr.digits.as_mut() {
                            Some(digits) if digits.capacity() - digits.len() >= needed => digits,
                            Some(digits) => {
                                let max = (max_exponent as usize) + 25;
                                digits
                                    .try_reserve_exact(max.min(needed))
                                    .expect("Failed to allocate memory");
                                digits
                            }
                            None => {
                                let max = (max_exponent as usize) + 25;
                                duration_repr.digits = Some(Vec::with_capacity(max.min(needed)));
                                duration_repr.digits.as_mut().unwrap()
                            }
                        };
                        Some(self.parse_fract(digits))
                    }
                    Some(_) | None if duration_repr.whole.is_none() => {
                        // Use the decimal point as anchor for the error position. Subtraction by 1
                        // is safe since we were advancing by one before.
                        return Err(ParseError::Syntax(
                            self.current_pos - 1,
                            "Either the whole number part or the fraction must be present"
                                .to_string(),
                        ));
                    }
                    Some(_) => None,
                    None => return Ok((duration_repr, None)),
                };
                duration_repr.fract = fract;
            }
            Some(byte) if *byte == b'.' => {
                return Err(ParseError::Syntax(
                    self.current_pos,
                    "No fraction allowed".to_string(),
                ));
            }
            Some(_) => {}
            None => return Ok((duration_repr, None)),
        }

        // TODO: what about time units starting with an 'e'??
        // parse the exponent of the input if present
        match self.current_byte {
            Some(byte) if byte.eq_ignore_ascii_case(&b'e') && !disable_exponent => {
                self.advance();
                duration_repr.exponent = self.parse_exponent()?;
            }
            Some(byte) if byte.eq_ignore_ascii_case(&b'e') => {
                return Err(ParseError::Syntax(
                    self.current_pos,
                    "No exponent allowed".to_string(),
                ));
            }
            Some(_) => {}
            None => return Ok((duration_repr, None)),
        }

        // If allow_delimiter is Some and there are any delimiters between the number and the time
        // unit, the delimiters are consumed before trying to parse the time units
        match (self.current_byte, allow_delimiter) {
            (Some(byte), Some(delimiter)) if delimiter(*byte) => {
                self.advance();
                // TODO: replace with try_consume_delimiter
                self.consume_delimiter(delimiter);
            }
            (Some(_), _) => {}
            (None, _) => return Ok((duration_repr, None)),
        }

        // parse the time unit if present
        match self.current_byte {
            Some(_) if !self.time_units.is_empty() => {
                if let Some((unit, multi)) = self.parse_time_unit(parse_multiple)? {
                    duration_repr.unit = unit;
                    duration_repr.multiplier = multi;
                }
            }
            Some(byte) if parse_multiple.is_none() => {
                return Err(ParseError::TimeUnit(
                    self.current_pos,
                    format!("No time units allowed but found: '{}'", *byte as char),
                ));
            }
            // If multiple is Some and self.time_units is empty we don't need to try to parse time
            // units
            Some(_) => {}
            None => return Ok((duration_repr, None)),
        }

        // check we've reached the end of input
        match (self.current_byte, parse_multiple) {
            (Some(byte), Some(delimiter)) if delimiter(*byte) => self
                .try_consume_delimiter(delimiter)
                .map(|_| (duration_repr, Some(self))),
            (Some(_), Some(_)) => Ok((duration_repr, Some(self))),
            (Some(_), None) => unreachable!("Parsing time units consumes the rest of the input"), /* cov:excl-line */
            (None, _) => Ok((duration_repr, None)),
        }
    }

    #[inline]
    fn advance(&mut self) {
        self.current_pos += 1;
        self.current_byte = self.input.get(self.current_pos);
    }

    #[inline]
    unsafe fn advance_by(&mut self, num: usize) {
        self.current_pos += num;
        self.current_byte = self.input.get(self.current_pos);
    }

    #[inline]
    fn peek(&self, num: usize) -> Option<&[u8]> {
        self.input.get(self.current_pos..self.current_pos + num)
    }

    #[inline]
    fn get_remainder(&self) -> &[u8] {
        &self.input[self.current_pos..]
    }

    #[inline]
    unsafe fn get_remainder_str_unchecked(&self) -> &str {
        std::str::from_utf8_unchecked(self.get_remainder())
    }

    #[inline]
    fn finish(&mut self) {
        self.current_pos = self.input.len();
        self.current_byte = None
    }

    /// This method is based on the work of Daniel Lemire and his blog post
    /// <https://lemire.me/blog/2018/09/30/quickly-identifying-a-sequence-of-digits-in-a-string-of-characters/>
    fn is_8_digits(&self) -> bool {
        self.input
            .get(self.current_pos..(self.current_pos + 8))
            .map_or(false, |digits| {
                let ptr = digits.as_ptr() as *const u64;
                // SAFETY: We just ensured there are 8 bytes
                let num = u64::from_le(unsafe { ptr.read_unaligned() });
                (num & (num.wrapping_add(0x0606060606060606)) & 0xf0f0f0f0f0f0f0f0)
                    == 0x3030303030303030
            })
    }

    fn parse_8_digits(&mut self) -> Option<u64> {
        self.input
            .get(self.current_pos..(self.current_pos + 8))
            .and_then(|digits| {
                let ptr = digits.as_ptr() as *const u64;
                // SAFETY: We just ensured there are 8 bytes
                let num = u64::from_le(unsafe { ptr.read_unaligned() });
                if (num & (num.wrapping_add(0x0606060606060606)) & 0xf0f0f0f0f0f0f0f0)
                    == 0x3030303030303030
                {
                    unsafe { self.advance_by(8) }
                    Some(num - 0x3030303030303030)
                } else {
                    None
                }
            })
    }

    fn consume_delimiter(&mut self, delimiter: Delimiter) {
        while let Some(byte) = self.current_byte {
            if delimiter(*byte) {
                self.advance()
            } else {
                break;
            }
        }
    }

    fn try_consume_delimiter(&mut self, delimiter: Delimiter) -> Result<(), ParseError> {
        debug_assert!(delimiter(*self.current_byte.unwrap())); // cov:excl-line

        let start = self.current_pos;
        self.advance();
        while let Some(byte) = self.current_byte {
            if delimiter(*byte) {
                self.advance()
            } else {
                break;
            }
        }

        match self.current_byte {
            None if self.current_pos - start > 0 => Err(ParseError::Syntax(
                start,
                "Input may not end with a delimiter".to_string(),
            )),
            Some(_) | None => Ok(()),
        }
    }

    #[inline]
    fn parse_time_unit(
        &mut self,
        multiple: Option<Delimiter>,
    ) -> Result<Option<(TimeUnit, Multiplier)>, ParseError> {
        // cov:excl-start
        debug_assert!(
            self.current_byte.is_some(),
            "Don't call this function without being sure there's at least 1 byte remaining"
        ); // cov:excl-stop

        match multiple {
            Some(delimiter) => {
                let start = self.current_pos;
                while let Some(byte) = self.current_byte {
                    if delimiter(*byte) || byte.is_ascii_digit() {
                        break;
                    }
                    self.advance();
                }

                let string =
                    std::str::from_utf8(&self.input[start..self.current_pos]).map_err(|error| {
                        ParseError::TimeUnit(
                            start + error.valid_up_to(),
                            "Invalid utf-8 when applying the delimiter".to_string(),
                        )
                    })?;

                if string.is_empty() {
                    Ok(None)
                } else {
                    match self.time_units.get(string) {
                        None => Err(ParseError::TimeUnit(
                            start,
                            format!("Invalid time unit: '{string}'"),
                        )),
                        some_time_unit => Ok(some_time_unit),
                    }
                }
            }
            None => {
                // SAFETY: The input of `parse` is &str and therefore valid utf-8 and we have read
                // only ascii characters up to this point.
                let string = unsafe { self.get_remainder_str_unchecked() };
                let result = match self.time_units.get(string) {
                    None => Err(ParseError::TimeUnit(
                        self.current_pos,
                        format!("Invalid time unit: '{string}'"),
                    )),
                    some_time_unit => Ok(some_time_unit),
                };
                self.finish();
                result
            }
        }
    }

    #[inline]
    fn parse_whole(&mut self, digits: &mut Vec<u8>) -> Whole {
        debug_assert!(
            self.current_byte
                .map_or(false, |byte| byte.is_ascii_digit())
        );

        let mut capacity = digits.capacity();
        let mut strip_leading_zeroes = true;
        if capacity >= 8 && self.is_8_digits() {
            let mut counter = 0;
            let ptr = digits.as_ptr() as *mut u64;
            while let Some(eight) = self.parse_8_digits() {
                if capacity >= 8 && (!strip_leading_zeroes || eight != 0) {
                    // SAFETY: We just ensured there is enough capacity in the vector
                    unsafe { ptr.add(counter).write_unaligned(u64::from_le(eight)) }
                    counter += 1;
                    strip_leading_zeroes = false;
                    capacity -= 8;
                }
            }

            // SAFETY: counter * 8 results always within the reserved space for the vector.
            unsafe { digits.set_len(counter << 3) }
        // capacity is smaller than 8 or there are no 8 digits
        } else {
            let digit = self.current_byte.unwrap() - b'0';
            if digit != 0 {
                digits.push(digit);
                strip_leading_zeroes = false;
            }
            self.advance();
        }

        while let Some(byte) = self.current_byte {
            let digit = byte.wrapping_sub(b'0');
            if digit < 10 {
                if capacity > 0 && (!strip_leading_zeroes || digit != 0) {
                    digits.push(digit);
                    strip_leading_zeroes = false;
                    // no capacity decrement needed since `max` is aligned at modulo 8
                }
                self.advance();
            } else {
                break;
            }
        }

        Whole(digits.len())
    }

    #[inline]
    fn parse_fract(&mut self, digits: &mut Vec<u8>) -> Fract {
        debug_assert!(
            self.current_byte
                .map_or(false, |byte| byte.is_ascii_digit())
        );

        let mut capacity = digits.capacity() - digits.len();
        let start = digits.len();
        if capacity >= 8 && self.is_8_digits() {
            let mut counter = 0;
            let mut ptr = digits.as_ptr() as *const u8;
            unsafe { ptr = ptr.add(start) };
            let ptr = ptr as *mut u64;
            while let Some(eight) = self.parse_8_digits() {
                if capacity >= 8 {
                    // SAFETY: We just ensured capacity >= 8
                    unsafe { ptr.add(counter).write_unaligned(u64::from_le(eight)) }
                    counter += 1;
                    capacity -= 8;
                }
            }

            // SAFETY: counter * 8 results always within the reserved space for the vector.
            unsafe { digits.set_len(start + (counter << 3)) }
        } else {
            let digit = self.current_byte.unwrap() - b'0';
            digits.push(digit);
            self.advance();
        }

        while let Some(byte) = self.current_byte {
            let digit = byte.wrapping_sub(b'0');
            if digit < 10 {
                if capacity > 0 {
                    digits.push(digit);
                    // no capacity decrement needed
                }
                self.advance();
            } else {
                break;
            }
        }

        Fract(start, digits.len())
    }

    #[inline]
    fn parse_infinity_remainder(&mut self, multiple: Option<Delimiter>) -> Result<(), ParseError> {
        match (self.current_byte, multiple) {
            (Some(byte), Some(delimiter)) if delimiter(*byte) => {
                return self.try_consume_delimiter(delimiter);
            }
            (Some(_), None) | (Some(_), Some(_)) => {}
            (None, _) => return Ok(()),
        }

        let expected = b"inity";
        for byte in expected.iter() {
            match self.current_byte {
                Some(current) if current.eq_ignore_ascii_case(byte) => self.advance(),
                // wrong character
                Some(current) => {
                    return Err(ParseError::Syntax(
                        self.current_pos,
                        format!(
                            "Error parsing infinity: Invalid character '{}'",
                            *current as char
                        ),
                    ));
                }
                None => {
                    return Err(ParseError::Syntax(
                        self.current_pos,
                        "Error parsing infinity: Premature end of input".to_string(),
                    ));
                }
            }
        }

        if let (Some(byte), Some(delimiter)) = (self.current_byte, multiple) {
            if delimiter(*byte) {
                return self.try_consume_delimiter(delimiter);
            } else {
                return Err(ParseError::Syntax(
                    self.current_pos,
                    format!(
                        "Error parsing infinity: Expected a delimiter but found '{}'",
                        *byte as char
                    ),
                ));
            }
        }

        Ok(())
    }

    /// Parse and consume the sign if present. Return true if sign is negative.
    #[inline]
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
            None => Err(ParseError::Syntax(
                self.current_pos,
                "Unexpected end of input".to_string(),
            )),
        }
    }

    #[inline]
    fn parse_exponent(&mut self) -> Result<i16, ParseError> {
        let is_negative = self.parse_sign_is_negative()?;
        self.current_byte.ok_or_else(|| {
            ParseError::Syntax(
                self.current_pos,
                "Expected exponent but reached end of input".to_string(),
            )
        })?;

        let mut exponent = 0i16;
        while let Some(byte) = self.current_byte {
            let digit = byte.wrapping_sub(b'0');
            if digit < 10 {
                exponent = if is_negative {
                    match exponent
                        .checked_mul(10)
                        .and_then(|e| e.checked_sub(digit as i16))
                    {
                        Some(exponent) => exponent,
                        None => return Err(ParseError::NegativeExponentOverflow),
                    }
                } else {
                    match exponent
                        .checked_mul(10)
                        .and_then(|e| e.checked_add(digit as i16))
                    {
                        Some(exponent) => exponent,
                        None => return Err(ParseError::PositiveExponentOverflow),
                    }
                };
                self.advance();
            } else {
                break;
            }
        }

        Ok(exponent)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    struct TimeUnitsFixture;

    // cov:excl-start This is just a fixture
    impl TimeUnitsLike for TimeUnitsFixture {
        fn is_empty(&self) -> bool {
            true
        }

        fn get(&self, _: &str) -> Option<(TimeUnit, Multiplier)> {
            None
        }
    } // cov:excl-stop

    #[rstest]
    #[case::zeroes("00000000")]
    #[case::nines("99999999")]
    #[case::mixed("012345678")]
    #[case::more_than_8_digits("0123456789")]
    fn test_duration_repr_parse_is_8_digits_when_8_digits(#[case] input: &str) {
        let config = Config::new();
        let parser = ReprParser::new(input, &config, &TimeUnitsFixture);
        assert!(parser.is_8_digits());
    }

    #[rstest]
    #[case::empty("")]
    #[case::less_than_8("0000000")]
    #[case::all_forward_slash("////////")] // '/' = 0x2F one below '0'
    #[case::all_double_point("::::::::")] // ':' = 0x3A one above '9'
    #[case::one_not_digit("a0000000")]
    fn test_duration_repr_parse_is_8_digits_when_not_8_digits(#[case] input: &str) {
        let config = Config::new();
        let parser = ReprParser::new(input, &config, &TimeUnitsFixture);
        assert!(!parser.is_8_digits());
    }

    #[rstest]
    #[case::zeros("00000000", Some(0x0000000000000000))]
    #[case::one("00000001", Some(0x0100000000000000))]
    #[case::ten_millions("10000000", Some(0x0000000000000001))]
    #[case::nines("99999999", Some(0x0909090909090909))]
    fn test_duration_repr_parser_parse_8_digits(
        #[case] input: &str,
        #[case] expected: Option<u64>,
    ) {
        let config = Config::new();
        let mut parser = ReprParser::new(input, &config, &TimeUnitsFixture);
        assert_eq!(parser.parse_8_digits(), expected);
    }

    #[rstest]
    #[case::empty("", None)]
    #[case::one_non_digit_char("a0000000", None)]
    #[case::less_than_8_digits("9999999", None)]
    fn test_duration_repr_parser_parse_8_digits_when_not_8_digits(
        #[case] input: &str,
        #[case] expected: Option<u64>,
    ) {
        let config = Config::new();
        let mut parser = ReprParser::new(input, &config, &TimeUnitsFixture);
        assert_eq!(parser.parse_8_digits(), expected);
        assert_eq!(parser.get_remainder(), input.as_bytes());
        assert_eq!(parser.current_byte, input.as_bytes().first());
        assert_eq!(parser.current_pos, 0);
    }

    #[test]
    fn test_duration_repr_parser_parse_8_digits_when_more_than_8() {
        let config = Config::new();
        let mut parser = ReprParser::new("00000000a", &config, &TimeUnitsFixture);
        assert_eq!(parser.parse_8_digits(), Some(0));
        assert_eq!(parser.get_remainder(), &[b'a']);
        assert_eq!(parser.current_byte, Some(&b'a'));
        assert_eq!(parser.current_pos, 8);
    }

    #[rstest]
    #[case::zero("0", vec![])]
    #[case::one("1", vec![1])]
    #[case::nine("9", vec![9])]
    #[case::ten("10", vec![1,0])]
    #[case::eight_leading_zeroes("00000000", vec![])]
    #[case::fifteen_leading_zeroes("000000000000000", vec![])]
    #[case::ten_with_leading_zeros_when_eight_digits("00000010", vec![0,0,0,0,0,0,1,0])]
    #[case::ten_with_leading_zeros_when_nine_digits("000000010", vec![0,0,0,0,0,0,0,1,0])]
    #[case::mixed_number("12345", vec![1,2,3,4,5])]
    #[case::max_8_digits("99999999", vec![9,9,9,9,9,9,9,9])]
    #[case::max_8_digits_minus_one("99999998", vec![9,9,9,9,9,9,9,8])]
    #[case::min_nine_digits("100000000", vec![1,0,0,0,0,0,0,0,0])]
    #[case::min_nine_digits_plus_one("100000001", vec![1,0,0,0,0,0,0,0,1])]
    #[case::eight_zero_digits_start("0000000011111111", vec![1,1,1,1,1,1,1,1])]
    #[case::eight_zero_digits_end("1111111100000000", vec![1,1,1,1,1,1,1,1,0,0,0,0,0,0,0,0])]
    #[case::eight_zero_digits_middle("11111111000000001", vec![1,1,1,1,1,1,1,1,0,0,0,0,0,0,0,0,1])]
    #[case::max_16_digits("9999999999999999", vec![9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9])]
    fn test_duration_repr_parser_parse_whole(
        #[case] input: &str,
        #[case] expected_digits: Vec<u8>,
    ) {
        let config = Config::new();
        let mut parser = ReprParser::new(input, &config, &TimeUnitsFixture);
        let mut digits = Vec::with_capacity(input.len());
        assert_eq!(parser.parse_whole(&mut digits), Whole(digits.len()));
        assert_eq!(digits, expected_digits);
    }

    #[test]
    fn test_duration_repr_parser_parse_whole_when_more_than_max() {
        let config = Config::new();
        let input = &"1".repeat(i16::MAX as usize + 100);
        let expected = vec![1u8; i16::MAX as usize + 33];
        let mut parser = ReprParser::new(input, &config, &TimeUnitsFixture);
        assert_eq!(parser.parse().unwrap().0.digits.unwrap(), expected);
    }

    #[test]
    fn test_duration_repr_parser_parse_fract_when_more_than_max() {
        let input = format!(".{}", "1".repeat(i16::MAX as usize + 100));
        let expected = vec![1u8; i16::MAX as usize + 25];
        let config = Config::new();
        let mut parser = ReprParser::new(&input, &config, &TimeUnitsFixture);
        let result = parser.parse().unwrap();
        let digits = result.0.digits.unwrap();
        assert_eq!(digits.len(), expected.len());
        assert_eq!(digits, expected);
    }

    #[rstest]
    #[case::zero("0", vec![0])]
    #[case::one("1", vec![1])]
    #[case::nine("9", vec![9])]
    #[case::ten("10", vec![1,0])]
    #[case::leading_zero("01", vec![0,1])]
    #[case::leading_zeroes("001", vec![0,0,1])]
    #[case::eight_leading_zeros("000000001", vec![0,0,0,0,0,0,0,0,1])]
    #[case::mixed_number("12345", vec![1,2,3,4,5])]
    #[case::max_8_digits("99999999", vec![9,9,9,9,9,9,9,9])]
    #[case::max_8_digits_minus_one("99999998", vec![9,9,9,9,9,9,9,8])]
    #[case::nine_digits("123456789", vec![1,2,3,4,5,6,7,8,9])]
    fn test_duration_repr_parser_parse_fract(#[case] input: &str, #[case] expected: Vec<u8>) {
        let config = Config::new();
        let mut parser = ReprParser::new(input, &config, &TimeUnitsFixture);
        let mut digits = Vec::with_capacity(input.len());
        assert_eq!(parser.parse_fract(&mut digits), Fract(0, input.len()));
        assert_eq!(digits, expected)
    }
}
