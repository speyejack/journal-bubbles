use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::status::BubbleStatus;

#[derive(Serialize, Deserialize, Clone, Debug)]
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
