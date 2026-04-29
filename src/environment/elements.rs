#![allow(dead_code)]

pub const BEACH_CHUNKS: &[&str] = &[
    "....", "..", "..", "..", ".,", ",.", "::", "~~", "~~", "o.", ".o", "o", "@", "''",
];

pub const GARDEN_CHUNKS: &[&str] = &[
    "....", "..", "..", "..", ".,", "`.", "''", "^^", "^^", "^^", "**", "++", "vv", "()",
];

pub const ROCKY_CHUNKS: &[&str] = &[
    "....", "..", "..", ".o", "o.", "oo", "O.", ".O", "O", "::", "~~", "~~",
];

pub const MINIMAL_CHUNKS: &[&str] = &["....", "...", "..", "..", "..", "..", ".,", "^^", "''"];

pub const SUN: &[&str] = &[r"  \*/  ", r"-- O --", r"  /*\  "];

pub const MOON_SMALL: &[&str] = &[r"  ,-,", r" /.(", r" \ {", r"  `-`"];

pub const CLOUD_SMALL: &[&str] = &[r"  .--.  ", r" (    ) ", r"  `--'  "];

pub const CLOUD_LARGE: &[&str] = &[
    r"   .---.   ",
    r"  (     )  ",
    r" (       ) ",
    r"  `-----'  ",
];

pub const STAR_CHARS: &[char] = &['*', '+', '.', '*', '.', '+', '*'];
