use chrono::{Datelike, Local, NaiveDate};

pub mod bubble;
pub mod status;
pub mod web;

pub fn today() -> NaiveDate {
    let today = Local::now();
    NaiveDate::from_yo_opt(today.year(), today.ordinal()).unwrap()
}
