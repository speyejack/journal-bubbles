#![feature(try_blocks)]
use std::ops::IndexMut;

use bubbles_core::bubble::Bubble;
use bubbles_core::status::BubbleStatus;
use bubbles_core::today;

use chrono::NaiveDate;
use iced::widget::{self, radio, row, Button, Column};
use iced::{executor, Application, Command, Element, Length, Settings, Theme};
use reqwest::Client;
use web_sys::console::log_1 as log;

fn main() -> iced::Result {
    Bubbles::run(Settings::default())
}

pub struct Bubbles {
    cli: Client,
    day: NaiveDate,
    bubbles: Vec<Bubble>,
}

const ADDR: &str = "/bubbles";

fn base_url() -> String {
    web_sys::window().unwrap().location().origin().unwrap()
}

async fn async_fetch_bubbles(cli: Client) -> Result<Vec<Bubble>, ()> {
    let result: anyhow::Result<_> = try {
        let url = format!("{}{}/{}/{}", base_url(), ADDR, "get", 0);
        cli.get(url).send().await?.json::<Vec<Bubble>>().await?
    };
    result.map_err(|e| log(&format!("Fetch Bubble error: {e}").into()))
}

async fn async_send_bubbles(cli: Client, bubbles: Vec<Bubble>) -> Result<(), ()> {
    let result: anyhow::Result<()> = try {
        let url = format!("{}{}/{}", base_url(), ADDR, "set");
        cli.post(url).json(&bubbles).send().await?;
    };
    result.map_err(|e| println!("Send Bubble error: {e}"))
}

#[derive(Clone, Debug)]
pub enum Message {
    RecvBubbles(Result<Vec<Bubble>, ()>),
    SetBubble(BubbleStatus, usize),
    SendBubbles,
    BubbleReceipt(Result<(), ()>),
}

impl Application for Bubbles {
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let cli = Client::new();

        (
            Self {
                cli: cli.clone(),
                day: today(),
                bubbles: Default::default(),
            },
            Command::perform(async_fetch_bubbles(cli), Message::RecvBubbles),
        )
    }

    fn title(&self) -> String {
        String::from("Bubbles")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SetBubble(status, index) => {
                self.bubbles.index_mut(index).days.insert(self.day, status);
            }
            Message::RecvBubbles(bu) => match bu {
                Ok(b) => self.bubbles = b,
                Err(_e) => {
                    println!("Error fetching bubbles");
                }
            },
            Message::SendBubbles => {
                return Command::perform(
                    async_send_bubbles(self.cli.clone(), self.bubbles.clone()),
                    Message::BubbleReceipt,
                )
            }
            Message::BubbleReceipt(_) => {}
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let mut elements = vec![];
        for (i, b) in self.bubbles.iter().enumerate() {
            let r = create_row(i, b, self.day);
            elements.push(r);
        }
        elements.push(Button::new("Send").on_press(Message::SendBubbles).into());
        Column::with_children(elements).into()
    }
}

fn create_row(i: usize, b: &Bubble, day: NaiveDate) -> Element<Message> {
    let selected = Some(b.days.get(&day).copied().unwrap_or(BubbleStatus::Unknown));
    let mut brief_parts = b.brief.split('/');
    let brief_front = brief_parts.next().unwrap().to_string();
    let brief_back = brief_parts.next().unwrap().to_string();
    let r = row![
        widget::container(widget::text(b.name.clone())).width(Length::FillPortion(1)),
        widget::container(widget::text(brief_front)).width(Length::FillPortion(1)),
        widget::container(widget::text(brief_back)).width(Length::FillPortion(3)),
        radio("unknown", BubbleStatus::Unknown, selected, |x| {
            Message::SetBubble(x, i)
        }),
        radio("empty", BubbleStatus::Empty, selected, |x| {
            Message::SetBubble(x, i)
        }),
        radio("halffull", BubbleStatus::HalfFull, selected, |x| {
            Message::SetBubble(x, i)
        }),
        radio("full", BubbleStatus::Full, selected, |x| {
            Message::SetBubble(x, i)
        }),
    ]
    .into();
    r
}
