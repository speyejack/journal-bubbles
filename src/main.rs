use anyhow::Result;
use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env::args,
    fmt::{Display, Write},
    fs::File,
    str::FromStr,
};
use tabled::{
    builder::Builder,
    object::{Column, Columns},
    Alignment, Modify, Style,
};

const BUBBLE_FILE: &str = "./bubbles.txt";

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
enum BubbleStatus {
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
struct Bubble {
    pub name: String,
    pub description: String,
    pub days: HashMap<NaiveDate, BubbleStatus>,
}

impl Bubble {
    pub fn new<T: ToString>(name: T, description: T) -> Self {
        Bubble {
            name: name.to_string(),
            description: description.to_string(),
            days: Default::default(),
        }
    }
    pub fn make_row(&self, days: &[NaiveDate]) -> Vec<String> {
        let mut vec = Vec::new();
        vec.push(self.name.clone());
        let statuses = days.iter().map(|x| {
            self.days
                .get(&x)
                .unwrap_or(&BubbleStatus::Unknown)
                .to_string()
        });

        vec.extend(statuses);
        vec
    }
}

fn today() -> NaiveDate {
    let today = Local::now();
    NaiveDate::from_yo_opt(today.year(), today.ordinal()).unwrap()
}

fn get_last_day(today: NaiveDate, last_day: Weekday) -> NaiveDate {
    let last_week = today - Duration::days(6);

    last_week
        .iter_days()
        .take(7)
        .filter(|x| x.weekday() == last_day)
        .take(1)
        .next()
        .unwrap()
}

fn main() -> Result<()> {
    let first = args().nth(1);
    match first {
        Some(day) => {
            if day == *"init" {
                println!("Initing file");
                let bubbles: Vec<Bubble> = vec![
                    Bubble::new("Review", ""),
                    Bubble::new("Sleep", ""),
                    Bubble::new("Water", ""),
                    Bubble::new("Diet", ""),
                    Bubble::new("Stretch", ""),
                    Bubble::new("Walk", ""),
                    Bubble::new("Clean", ""),
                    Bubble::new("Exercise", ""),
                    Bubble::new("Breath", ""),
                    Bubble::new("Writing", ""),
                ];

                serde_json::to_writer(File::create(BUBBLE_FILE)?, &bubbles)?;
                return Ok(());
            }

            let mut bubbles: Vec<Bubble> = serde_json::from_reader(File::open(BUBBLE_FILE)?)?;

            if day == *"count" {
                let yesterday = today().pred_opt().unwrap();
                let mut count = 0;
                for bubble in bubbles {
                    let status = bubble
                        .days
                        .get(&yesterday)
                        .unwrap_or(&BubbleStatus::Unknown);

                    count += if *status == BubbleStatus::Unknown {
                        1
                    } else {
                        0
                    }
                }
                println!("{count}");
                return Ok(());
            }

            let week_day = Weekday::from_str(&day).unwrap();
            let today = today();
            let last_day = get_last_day(today, week_day);

            let values = args().nth(2).unwrap();
            let statuses: Result<Vec<_>> = values
                .chars()
                .map(|x| BubbleStatus::from_str(&x.to_string()))
                .collect();
            for (bubble, status) in bubbles.iter_mut().zip(statuses?) {
                bubble.days.insert(last_day, status);
            }
            serde_json::to_writer(File::create(BUBBLE_FILE)?, &bubbles)?
        }
        None => {
            let data: Vec<Bubble> = serde_json::from_reader(File::open(BUBBLE_FILE)?)?;
            let days: Vec<_> = (today() - Duration::days(6)).iter_days().take(7).collect();

            let mut builder = Builder::default();
            let mut columns = vec!["Tasks".to_string()];
            columns.extend(days.iter().map(|x| x.format("%a\n%m/%d").to_string()));

            builder.set_columns(columns);
            for bubble in data {
                builder.add_record(bubble.make_row(&days));
            }

            let mut table = builder.build();
            table.with(Style::rounded());
            table.with(Alignment::center());
            table.with(Modify::new(Columns::single(0)).with(Alignment::right()));
            println!("{table}")
        }
    };
    Ok(())
}
