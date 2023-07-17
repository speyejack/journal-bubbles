use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Write},
    str::FromStr,
};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BubbleStatus {
    Unknown,
    Empty,
    HalfFull,
    Full,
}

impl Display for BubbleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BubbleStatus::Empty => f.write_char('○'),
            BubbleStatus::HalfFull => f.write_char('◐'),
            BubbleStatus::Full => f.write_char('●'),
            BubbleStatus::Unknown => f.write_char('◌'),
        }
    }
}

impl FromStr for BubbleStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x" => Ok(BubbleStatus::Full),
            "/" => Ok(BubbleStatus::HalfFull),
            "o" => Ok(BubbleStatus::Empty),
            "?" => Ok(BubbleStatus::Unknown),
            _ => Err(anyhow::anyhow!("Bad bubble char")),
        }
    }
}
