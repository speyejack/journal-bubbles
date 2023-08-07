#![feature(try_blocks)]
use std::borrow::Cow;
use std::ops::IndexMut;

use bubbles_core::bubble::Bubble;
use bubbles_core::status::BubbleStatus;
use bubbles_core::today;

use chrono::{DateTime, Days, NaiveDate, Timelike};
use iced::alignment::Horizontal;
use iced::widget::{self, radio, row, Button, Column, Text};
use iced::{executor, Alignment, Application, Command, Element, Length, Settings, Theme};
use reqwest::{Client, Response};
use web_sys::console::log_1 as log;

fn main() -> iced::Result {
    Bubbles::run(Settings::default())
}

pub struct Bubbles<'a> {
    cli: Client,
    day: NaiveDate,
    bubbles: Vec<Bubble>,
    status_str: Cow<'a, str>,
    day_offset: u32,
    day_str: Cow<'a, str>,
}

const ADDR: &str = "/bubbles";

fn base_url() -> String {
    web_sys::window().unwrap().location().origin().unwrap()
}

async fn async_fetch_bubbles(cli: Client, offset: u32) -> Result<Vec<Bubble>, ()> {
    let result: anyhow::Result<_> = try {
        let url = format!("{}{}/{}/{}", base_url(), ADDR, "get", offset);
        cli.get(url).send().await?.json::<Vec<Bubble>>().await?
    };
    result.map_err(|e| log(&format!("Fetch Bubble error: {e}").into()))
}

async fn async_send_bubbles(cli: Client, bubbles: Vec<Bubble>) -> bool {
    let result: anyhow::Result<Response> = try {
        let url = format!("{}{}/{}", base_url(), ADDR, "set");
        let ret = cli.post(url).json(&bubbles).send().await?;
        ret
    };

    result
        .map(|x| x.status().is_success())
        .map_err(|e| log(&format!("Send Bubble error: {e}").into()))
        .unwrap_or(false)
}

#[derive(Clone, Debug)]
pub enum Message {
    RecvBubbles(Result<Vec<Bubble>, ()>),
    SetBubble(BubbleStatus, usize),
    SendBubbles,
    OffsetDay(i32),
    BubbleReceipt(bool),
}

impl Application for Bubbles<'_> {
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let cli = Client::new();

        let day_offset = if chrono::Local::now().hour() < 12 {
            0
        } else {
            1
        };
        let day: NaiveDate = today() - Days::new(day_offset.into());
        let day_str = Cow::Owned(day.format("%A, %Y-%m-%d").to_string());

        (
            Self {
                cli: cli.clone(),
                day,
                bubbles: Default::default(),
                day_offset,
                day_str,
                status_str: Cow::Borrowed("Fetching bubbles"),
            },
            Command::perform(
                async_fetch_bubbles(cli, day_offset.into()),
                Message::RecvBubbles,
            ),
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
                Ok(b) => {
                    self.bubbles = b;
                    self.status_str = Cow::Borrowed("Unset bubbles")
                }
                Err(_e) => self.status_str = Cow::Borrowed("Failed to fetch bubbles!"),
            },
            Message::SendBubbles => {
                self.status_str = Cow::Borrowed("Sending bubbles");
                return Command::perform(
                    async_send_bubbles(self.cli.clone(), self.bubbles.clone()),
                    Message::BubbleReceipt,
                );
            }
            Message::BubbleReceipt(succ) => {
                let val = if succ {
                    "Set bubbles!"
                } else {
                    "Bubbles failed to set!"
                };
                self.status_str = Cow::Borrowed(val);
            }
            Message::OffsetDay(o) => {
                self.day_offset = self.day_offset.saturating_add_signed(o);
                self.day = today() - Days::new(self.day_offset.into());
                self.day_str = Cow::Owned(self.day.format("%A, %Y-%m-%d").to_string());
                self.status_str = Cow::Borrowed("Fetching bubbles");
                return Command::perform(
                    async_fetch_bubbles(self.cli.clone(), self.day_offset),
                    Message::RecvBubbles,
                );
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let mut elements = vec![];
        elements.push(
            row![
                Button::new(Text::new("<").horizontal_alignment(Horizontal::Center))
                    .on_press(Message::OffsetDay(1))
                    .width(Length::Fixed(40.0)),
                Text::new(self.day_str.as_ref())
                    .horizontal_alignment(Horizontal::Center)
                    .width(Length::Fill),
                Button::new(Text::new(">").horizontal_alignment(Horizontal::Center))
                    .on_press(Message::OffsetDay(-1))
                    .width(Length::Fixed(40.0)),
            ]
            .into(),
        );
        for (i, b) in self.bubbles.iter().enumerate() {
            let r = create_row(i, b, self.day);
            elements.push(r);
        }
        elements.push(Button::new("Send").on_press(Message::SendBubbles).into());
        elements.push(Text::new(self.status_str.as_ref()).into());
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
