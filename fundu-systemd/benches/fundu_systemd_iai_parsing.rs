// Copyright (c) 2023 Joining7943 <joining@posteo.de>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use fundu::{Duration, ParseError};
use fundu_systemd::TimeSpanParser;
use iai_callgrind::{black_box, main};

type Result<T> = std::result::Result<T, ParseError>;

const SMALL_INPUT: &str = "1";
const MIXED_INPUT_7: &str = "1234567.1234567";
const MIXED_INPUT_8: &str = "12345678.12345678";
const MIXED_INPUT_9: &str = "123456789.123456789";

#[inline(never)]
#[export_name = "__iai_setup::setup_parser"]
fn setup_parser<'a>() -> TimeSpanParser<'a> {
    TimeSpanParser::new()
}

#[inline(never)]
#[export_name = "__iai_setup::generate_large_input"]
fn generate_large_input() -> String {
    format!("{0}.{0}", "1".repeat(1022))
}

#[inline(never)]
fn small_input() -> Result<Duration> {
    let parser = setup_parser();
    black_box(parser).parse(black_box(SMALL_INPUT))
}

#[inline(never)]
fn mixed_input_7() -> Result<Duration> {
    let parser = setup_parser();
    black_box(parser).parse(black_box(MIXED_INPUT_7))
}

#[inline(never)]
fn mixed_input_8() -> Result<Duration> {
    let parser = setup_parser();
    black_box(parser).parse(black_box(MIXED_INPUT_8))
}

#[inline(never)]
fn mixed_input_9() -> Result<Duration> {
    let parser = setup_parser();
    black_box(parser).parse(black_box(MIXED_INPUT_9))
}

#[inline(never)]
fn large_input() -> Result<Duration> {
    let parser = setup_parser();
    let input = generate_large_input();
    black_box(parser).parse(black_box(&input))
}

main!(
    callgrind_args =
        "toggle-collect=iai_callgrind::black_box",
        "toggle-collect=__iai_setup::setup_parser",
        "toggle-collect=__iai_setup::generate_large_input";
    functions =
        small_input,
        mixed_input_7,
        mixed_input_8,
        mixed_input_9,
        large_input,
);
