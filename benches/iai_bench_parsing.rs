// Copyright (c) 2023 Joining7943 <joining@posteo.de>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::time::Duration;

use fundu::{DurationParser, ParseError};
use iai_callgrind::{black_box, main};

type Result<T> = std::result::Result<T, ParseError>;

const LARGE_INPUT: &str =
    "11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111.\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111111111111111111111\
    11111111111111111111111111111111111111111111111111111111111111e-1022";
const SMALL_INPUT: &str = "1";

#[inline(never)]
fn small_default_time_units() -> Result<Duration> {
    DurationParser::new().parse(black_box(SMALL_INPUT))
}

#[inline(never)]
fn small_without_time_units() -> Result<Duration> {
    DurationParser::without_time_units().parse(black_box(SMALL_INPUT))
}

#[inline(never)]
fn large_default_time_units() -> Result<Duration> {
    DurationParser::new().parse(black_box(LARGE_INPUT))
}

#[inline(never)]
fn large_without_time_units() -> Result<Duration> {
    DurationParser::without_time_units().parse(black_box(LARGE_INPUT))
}

main!(
    small_default_time_units,
    small_without_time_units,
    large_default_time_units,
    large_without_time_units,
);
