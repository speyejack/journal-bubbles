use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Display, Write},
    str::FromStr,
};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize)]
pub struct Bubble {
    pub name: String,
    pub description: String,
    pub brief: String,
    pub days: HashMap<NaiveDate, BubbleStatus>,
}

impl Bubble {
    pub fn new<T: ToString>(name: T, description: T) -> Self {
        Bubble {
            name: name.to_string(),
            description: description.to_string(),
            brief: "".to_string(),
            days: Default::default(),
        }
    }
    pub fn make_row(&self, days: &[NaiveDate]) -> Vec<String> {
        let mut vec = Vec::new();
        vec.push(self.name.clone());
        vec.push(self.brief.clone());
        let statuses = days.iter().map(|x| {
            self.days
                .get(x)
                .unwrap_or(&BubbleStatus::Unknown)
                .to_string()
        });

        vec.extend(statuses);
        vec
    }
}
