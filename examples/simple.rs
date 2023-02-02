// Copyright (c) 2023 Joining7943 <joining@posteo.de>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::time::Duration;

use fundu::{DurationParser, TimeUnit};

fn main() {
    let input = "100y";
    let duration = DurationParser::with_no_time_units()
        .time_unit(TimeUnit::Year)
        .parse(input)
        .unwrap();

    assert_eq!(duration, Duration::new(31_536_00000, 0))
}
