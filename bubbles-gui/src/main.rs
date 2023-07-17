use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::ops::{Index, IndexMut};

use bubbles_core::bubble::Bubble;
use bubbles_core::status::BubbleStatus;
use bubbles_core::today;
use bubbles_core::web::{Request, Response};
use chrono::NaiveDate;
use iced::widget::{self, button, column, radio, row, Button, Column};
use iced::{executor, Application, Command, Element, Length, Sandbox, Settings, Theme};

fn main() -> iced::Result {
    Bubbles::run(Settings::default())
}

struct Bubbles {
    addr: SocketAddr,
    day: NaiveDate,
    bubbles: Vec<Bubble>,
}

// const BUBBLE_FILE: &str = "/home/jack/Dropbox/notes/bubbles/bubbles.txt";
async fn async_fetch_bubbles(addr: SocketAddr) -> Result<Vec<Bubble>, ()> {
    let result = fetch_bubbles(addr).await;
    result.map_err(|e| {
        println!("Bubble error: {e}");
    })
}

async fn fetch_bubbles(addr: SocketAddr) -> anyhow::Result<Vec<Bubble>> {
    let mut socket = TcpStream::connect(addr)?;
    let req = Request::GetInfo;
    serde_json::to_writer(&mut socket, &req)?;
    let _ = socket.write("\n".as_bytes())?;
    println!("Wrote bubbles");
    println!("{:?}", serde_json::to_string(&req));

    let response: Response = serde_json::from_reader(&mut socket)?;
    println!("Got response");
    match response {
        Response::Bubbles(v) => return Ok(v),
        _ => panic!("Bad response"),
    }
}

async fn send_bubbles(addr: SocketAddr) -> Result<(), ()> {
    Ok(())
}

#[derive(Clone, Debug)]
enum Message {
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
        let addr = "127.0.0.1:45531".parse().unwrap();
        // let values = TcpStream::connect(addr);

        (
            Self {
                addr,
                day: today(),
                bubbles: Default::default(),
            },
            Command::perform(async_fetch_bubbles(addr), Message::RecvBubbles),
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
                Err(e) => {
                    println!("Error fetching bubbles");
                }
            },
            Message::SendBubbles => {
                return Command::perform(send_bubbles(self.addr), Message::BubbleReceipt)
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
        widget::container(widget::text(b.name.clone())).width(Length::FillPortion(2)),
        widget::container(widget::text(brief_front)).width(Length::FillPortion(1)),
        widget::container(widget::text(brief_back)).width(Length::FillPortion(3)),
        radio("unknown", BubbleStatus::Unknown, selected, |x| {
            Message::SetBubble(x, i)
        }),
        radio("full", BubbleStatus::Full, selected, |x| {
            Message::SetBubble(x, i)
        }),
        radio("halffull", BubbleStatus::HalfFull, selected, |x| {
            Message::SetBubble(x, i)
        }),
        radio("empty", BubbleStatus::Empty, selected, |x| {
            Message::SetBubble(x, i)
        }),
    ]
    .into();
    r
}
