use hyprland::dispatch::{Dispatch, DispatchType, WorkspaceIdentifierWithSpecial};
use hyprland::shared::{HyprData, HyprDataActive, HyprDataVec};
use iced::mouse::ScrollDelta;
use iced::time::{self, Duration};
use iced::widget::{button, column, container, horizontal_space, mouse_area, row, text};
use iced::{Alignment, Color, Element, Length, Padding, Subscription, Task as Command, Theme};
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
    workspace: WorkSpace,
    clock: Clock,
}

// Because new iced delete the custom command, so now we make a macro crate to generate
// the Command
#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    WorkspaceChangedTo(usize),
    Tick(chrono::DateTime<chrono::Local>),
    Scroll(f32),
}

impl Application for Bar {
    type Message = Message;
    type Flags = ();
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                workspace: WorkSpace::new(),
                clock: Clock::new(),
            },
            Command::none(),
        )
    }

    fn namespace(&self) -> String {
        String::from("Iced Status Bar")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        time::every(Duration::from_secs(1)).map(|_| Message::Tick(chrono::offset::Local::now()))
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick(time) => self.clock.update(time),
            Message::WorkspaceChangedTo(id) => self.workspace.change_to(id),
            Message::Scroll(dy) => self.workspace.scroll_by(dy),
            _ => Command::none(),
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

struct Clock {
    now: chrono::DateTime<chrono::Local>,
}

impl Clock {
    fn new() -> Self {
        Self {
            now: chrono::offset::Local::now(),
        }
    }

    fn subscription() -> Subscription<Message> {
        todo!()
    }

    fn update(&mut self, new_time: chrono::DateTime<chrono::Local>) -> Command<Message> {
        if new_time != self.now {
            self.now = new_time;
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        container(
            text(self.now.format("%H:%M%P").to_string())
                .size(24)
                .center(),
        )
        .width(Length::Shrink)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .padding(3)
        .style(|theme: &Theme| container::Style {
            border: iced::Border {
                color: theme.palette().text,
                width: 1.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }
}

struct WorkSpace {
    total_workspace: usize,
    focus_at: usize,
}

fn get_active_workspace_id() -> usize {
    if let Ok(active_workspace) = hyprland::data::Workspace::get_active() {
        return active_workspace.id as usize;
    }

    1
}
fn get_workspace_size() -> usize {
    if let Ok(workspaces) = hyprland::data::Workspaces::get() {
        return workspaces
            .to_vec()
            .iter()
            .map(|x| x.id)
            .max()
            .expect("Can't get any workspace id.") as usize;
    }

    1
}

impl WorkSpace {
    fn new() -> Self {
        let focus_at = get_active_workspace_id();
        let total_workspace = get_workspace_size();

        Self {
            total_workspace,
            focus_at,
        }
    }

    fn workspace_subscription() {}

    fn change_to(&mut self, id: usize) -> Command<Message> {
        if id == self.focus_at {
            return Command::none();
        }

        self.focus_at = id;
        Dispatch::call(DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(
            id as i32,
        )))
        .expect("Unreachable Workspace id");

        Command::none()
    }

    fn scroll_by(&mut self, dy: f32) -> Command<Message> {
        let cycle_id = if dy > 0.0 {
            // Mouse wheel scroll up, calculate the previous workspace id
            (self.focus_at - 1 + self.total_workspace - 1) % self.total_workspace + 1
        } else {
            // Mouse wheel scroll down, calculate the next workspace id
            (self.focus_at - 1 + 1) % self.total_workspace + 1
        };
        self.change_to(cycle_id)
    }

    fn view(&self) -> Element<Message> {
        mouse_area(
            container(
                row((0..self.total_workspace).map(|i| {
                    button(text(i).center())
                        .on_press(Message::WorkspaceChangedTo(i + 1))
                        .into()
                }))
                .spacing(10)
                .align_y(Alignment::Center)
                .wrap(),
            )
            .style(|theme: &Theme| container::Style {
                border: iced::Border {
                    color: theme.palette().text,
                    width: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .center_y(Length::Fill)
            .height(Length::Fill),
        )
        .on_scroll(|delta| match delta {
            ScrollDelta::Lines { x: _, y } => Message::Scroll(y),
            ScrollDelta::Pixels { x: _, y } => Message::Scroll(y),
        })
        .into()
    }
}
