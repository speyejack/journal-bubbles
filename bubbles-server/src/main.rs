#![feature(try_blocks)]
use bubbles_core::{bubble::Bubble, today};
use chrono::Days;
use rocket::{
    fs::{relative, FileServer},
    get, post, routes,
    serde::json::Json,
};
use std::fs::{File, OpenOptions};

const BUBBLE_FILE: &str = "./bubbles.json";

#[get("/get/<day_offset>")]
fn get_bubbles(day_offset: Option<u64>) -> Option<Json<Vec<Bubble>>> {
    let diff = day_offset.map(|x| today() - Days::new(x));
    let out: anyhow::Result<Vec<Bubble>> = try {
        let file = File::open(BUBBLE_FILE)?;
        let mut bubbles: Vec<Bubble> = serde_json::from_reader(file)?;

        bubbles.iter_mut().for_each(|x| {
            let out = diff.and_then(|d| x.days.remove(&d));
            x.days.clear();
            if let Some(b) = out {
                x.days.insert(diff.unwrap(), b);
            }
        });

        bubbles
    };

    out.ok().map(Json)
}

#[post("/set", data = "<new_bubbles>")]
fn set_bubbles(new_bubbles: Json<Vec<Bubble>>) -> Option<Json<Vec<Bubble>>> {
    let out: anyhow::Result<Vec<Bubble>> = try {
        let file = File::open(BUBBLE_FILE)?;
        let mut bubbles: Vec<Bubble> = serde_json::from_reader(file)?;
        let new_bubbles = new_bubbles.into_inner();

        bubbles
            .iter_mut()
            .zip(new_bubbles)
            .for_each(|(f, s)| f.days.extend(s.days));

        let file = OpenOptions::new().truncate(true).open(BUBBLE_FILE)?;
        serde_json::to_writer(file, &bubbles)?;

        bubbles
    };

    out.ok().map(Json)
}

#[rocket::launch]
fn rocket_main() -> _ {
    // rocket::build()
    let config = rocket::Config {
        port: 54438,
        address: "0.0.0.0".parse().unwrap(),
        ..Default::default()
    };

    rocket::custom(&config)
        .mount("/bubbles", routes![get_bubbles, set_bubbles])
        .mount("/", FileServer::from(relative!("../dist")))
}
