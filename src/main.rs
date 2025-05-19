pub mod modules;

use modules::clock::Clock;
use modules::workspace::{WorkspaceMessage, Workspaces};

use iced::widget::{container, horizontal_space, row};
use iced::{Element, Length, Subscription, Task, Theme};
use iced_layershell::Application;
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::to_layer_message;

pub fn main() -> Result<(), iced_layershell::Error> {
    let height = 40;

    Bar::run(Settings {
        layer_settings: LayerShellSettings {
            size: Some((0, height)),
            exclusive_zone: height as i32,
            margin: (4, 4, 0, 0),
            anchor: Anchor::Top | Anchor::Left | Anchor::Right,
            keyboard_interactivity: iced_layershell::reexport::KeyboardInteractivity::None,
            ..Default::default()
        },
        ..Default::default()
    })
}

struct Bar {
    workspace: Workspaces,
    clock: Clock,
}

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    ClockTick,
    WorkspaceDispatch(WorkspaceMessage),
}

impl Application for Bar {
    type Message = Message;
    type Flags = ();
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(_flags: ()) -> (Self, Task<Message>) {
        (
            Self {
                workspace: Workspaces::new(),
                clock: Clock::new(),
            },
            Task::none(),
        )
    }

    fn namespace(&self) -> String {
        String::from("Iced Status Bar")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        Subscription::batch(vec![
            self.clock.subscription(),
            self.workspace.subscription().map(Message::WorkspaceDispatch)
        ])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ClockTick => self.clock.update(),
            // Message::WorkspaceChangedTo(id) => self.workspace.change_to(id),
            // Message::Scroll(dy) => self.workspace.scroll_by(dy),
            Message::WorkspaceDispatch(message) => self.workspace.update(message),
            _ => Task::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        container(row![
            horizontal_space(),
            self.workspace.view(),
            horizontal_space(),
            self.clock.view()
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn style(&self, theme: &Self::Theme) -> iced_layershell::Appearance {
        use iced_layershell::Appearance;
        Appearance {
            // background_color: Color::new(0.0, 0.0, 0.0, 0.8),
            background_color: theme.palette().background.scale_alpha(0.8),
            text_color: theme.palette().text,
        }
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}


