use iced::{ widget::{container, text}, Alignment, Element, Length, Subscription, Task, Theme};
use iced::time::{self, Duration};
use crate::Message;

pub struct Clock; 

impl Clock {
    pub fn new() -> Self {
        Self
    }

    pub fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_secs(60)).map(|_| Message::ClockTick)
    }

    pub fn update(&mut self) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let now = chrono::Local::now();

        container(
            text(now.format("%H:%M%P").to_string())
                // .size(24)
                .center(),
        )
        .width(Length::Shrink)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .padding(3)
        // .style(|theme: &Theme| container::Style {
        //     border: iced::Border {
        //         color: theme.palette().text,
        //         width: 1.0,
        //         ..Default::default()
        //     },
        //     ..Default::default()
        // })
        .into()
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self
    }
}
