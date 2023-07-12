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
use tabled::{builder::Builder, object::Columns, Alignment, Modify, Style};

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
    match first.as_deref() {
        Some("show") | None => {
            let is_simple = args().nth(2).map(|x| &x == "simple").unwrap_or(true);
            let data: Vec<Bubble> = serde_json::from_reader(File::open(BUBBLE_FILE)?)?;
            let days: Vec<_> = (today() - Duration::days(6)).iter_days().take(7).collect();

            let mut builder = Builder::default();
            let mut columns = vec!["Tasks".to_string(), "Brief".to_string()];
            if is_simple {
                columns.remove(1);
            }

            columns.extend(days.iter().map(|x| x.format("%a\n%m/%d").to_string()));

            builder.set_columns(columns);
            for bubble in data {
                let mut row = bubble.make_row(&days);

                if is_simple {
                    row.remove(1);
                }
                builder.add_record(row);
            }

            let mut table = builder.build();
            table.with(Style::rounded());
            table.with(Alignment::center());
            table.with(Modify::new(Columns::single(0)).with(Alignment::right()));
            println!("{table}")
        }
        Some("explain") => {
            let bubbles: Vec<Bubble> = serde_json::from_reader(File::open(BUBBLE_FILE)?)?;

            let mut builder = Builder::default();
            let columns = vec!["Task", "Brief", "Description"];
            let columns: Vec<String> = columns.into_iter().map(|x| x.to_string()).collect();

            let mut brief_sizes = [0; 2];

            for bubble in &bubbles {
                let mut split_brief = bubble.brief.split('/');
                brief_sizes[0] = brief_sizes[0].max(split_brief.next().unwrap().chars().count());
                brief_sizes[1] = brief_sizes[1].max(split_brief.next().unwrap().chars().count());
            }

            builder.set_columns(columns);
            for bubble in bubbles {
                let mut old_brief = bubble.brief.split('/');

                let mut brief = String::new();
                brief.reserve(brief_sizes[0] + brief_sizes[1]);
                let first = old_brief.next().unwrap();
                for _ in 0..(brief_sizes[0] - (first.chars().count())) {
                    brief.push(' ');
                }
                brief.push_str(&bubble.brief);
                let second = old_brief.next().unwrap();
                for _ in 0..(brief_sizes[1] - (second.chars().count())) {
                    brief.push(' ');
                }

                builder.add_record([bubble.name, brief, bubble.description]);
            }

            let mut table = builder.build();
            table.with(Style::rounded());
            table.with(Alignment::center());
            table.with(Modify::new(Columns::single(0)).with(Alignment::right()));
            table.with(Modify::new(Columns::single(2)).with(Alignment::left()));
            println!("{table}")
        }
        Some("init") => {
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
        }
        Some("count") => {
            let bubbles: Vec<Bubble> = serde_json::from_reader(File::open(BUBBLE_FILE)?)?;

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
        }
        Some(day) => {
            let mut bubbles: Vec<Bubble> = serde_json::from_reader(File::open(BUBBLE_FILE)?)?;

            let week_day = Weekday::from_str(day).unwrap();
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
    };
    Ok(())
}
