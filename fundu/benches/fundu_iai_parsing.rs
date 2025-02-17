// Copyright (c) 2023 Joining7943 <joining@posteo.de>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use fundu::{Duration, DurationParser};
use iai_callgrind::{black_box, main};

const SMALL_INPUT: &str = "1";
const MIXED_INPUT_7: &str = "1234567.1234567";
const MIXED_INPUT_8: &str = "12345678.12345678";

#[inline(never)]
#[export_name = "__iai_setup::setup_parser"]
fn setup_parser<'a>() -> DurationParser<'a> {
    DurationParser::without_time_units()
}

#[inline(never)]
#[export_name = "__iai_setup::generate_large_input"]
fn generate_large_input() -> String {
    let ones = "1".repeat(1022);
    format!("{}.{}e-1022", &ones, &ones)
}

#[inline(never)]
fn small_input() -> Duration {
    let parser = setup_parser();
    black_box(parser).parse(black_box(SMALL_INPUT)).unwrap()
}

#[inline(never)]
fn mixed_input_7() -> Duration {
    let parser = setup_parser();
    black_box(parser).parse(black_box(MIXED_INPUT_7)).unwrap()
}

#[inline(never)]
fn mixed_input_8() -> Duration {
    let parser = setup_parser();
    black_box(parser).parse(black_box(MIXED_INPUT_8)).unwrap()
}

#[inline(never)]
fn large_input() -> Duration {
    let parser = setup_parser();
    let input = generate_large_input();
    black_box(parser).parse(black_box(&input)).unwrap()
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
        large_input,
);
