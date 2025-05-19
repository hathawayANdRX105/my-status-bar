use hyprland::dispatch::{Dispatch, DispatchType, WorkspaceIdentifierWithSpecial};
use hyprland::shared::{HyprData, HyprDataActive, HyprDataVec};
use iced::futures::SinkExt;
use iced::mouse::ScrollDelta;
use iced::widget::{button, container, mouse_area, row, text};
use iced::{Alignment, Element, Length, Subscription, Task, Theme, stream};

use crate::Message;

#[derive(Debug, Clone)]
pub enum WorkspaceMessage {
    ChangeTo(usize),
    ScrollBy(f32),
    UpdateTotalWorkspaceSize(usize),
}

pub struct Workspaces {
    total_workspaces: usize,
    focus_at: usize,
}

impl Workspaces {
    pub fn new() -> Self {
        Self {
            total_workspaces: 1,
            focus_at: 1,
        }
    }

    pub fn subscription(&self) -> Subscription<WorkspaceMessage> {
        Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                // 1. get the initial workspaces size
                let size = get_workspace_size().await;
                sender
                    .send(WorkspaceMessage::UpdateTotalWorkspaceSize(size))
                    .await
                    .unwrap_or_else(|err| {
                        eprintln!("Failed to get total workspace number:{err}");
                    });


                // 2. add workspaces event listen for getting total count of workspaces when switching active.
                let mut listener = hyprland::event_listener::AsyncEventListener::new();

                let listen_sender = sender.clone();
                listener.add_workspace_added_handler(move |_| {
                    let mut sender = listen_sender.clone();
                    Box::pin(async move {
                        let size = get_workspace_size().await;
                        sender
                            .send(WorkspaceMessage::UpdateTotalWorkspaceSize(size))
                            .await
                            .unwrap_or_else(|err| {
                                eprintln!("Failed to get total workspace number:{err}");
                            });
                    })
                });

                let listen_sender = sender.clone();
                listener.add_workspace_deleted_handler(move |_| {
                    let mut sender = listen_sender.clone();
                    Box::pin(async move {
                        let size = get_workspace_size().await;
                        sender
                            .send(WorkspaceMessage::UpdateTotalWorkspaceSize(size))
                            .await
                            .unwrap_or_else(|err| {
                                eprintln!("Failed to get total workspace number:{err}");
                            });
                    })
                });

                // TODO:update active window info

                listener
                    .start_listener_async()
                    .await
                    .expect("Failed to listen for hyprland events.");
            })
        })
    }

    fn change_to(&mut self, id: usize) -> Task<Message> {
        if id == self.focus_at {
            return Task::none();
        }

        self.focus_at = id;
        Dispatch::call(DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(
            id as i32,
        )))
        .expect("Unreachable Workspace id");

        Task::none()
    }

    fn scroll_by(&mut self, dy: f32) -> Task<Message> {
        let cycle_id = if dy > 0.0 {
            // Mouse wheel scroll up, calculate the previous workspace id
            (self.focus_at - 1 + self.total_workspaces - 1) % self.total_workspaces + 1
        } else {
            // Mouse wheel scroll down, calculate the next workspace id
            (self.focus_at - 1 + 1) % self.total_workspaces + 1
        };
        self.change_to(cycle_id)
    }

    pub fn update(&mut self, message: WorkspaceMessage) -> Task<Message> {
        match message {
            WorkspaceMessage::ChangeTo(id) => self.change_to(id),
            WorkspaceMessage::ScrollBy(dy) => self.scroll_by(dy),
            WorkspaceMessage::UpdateTotalWorkspaceSize(size) => {self.total_workspaces = size; Task::none()}
        }
    }

    pub fn view(&self) -> Element<Message> {
        mouse_area(
            container(
                row((0..self.total_workspaces).map(|i| {
                    button(text(i).center())
                        .on_press(Message::WorkspaceDispatch(WorkspaceMessage::ChangeTo(
                            i + 1,
                        )))
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
            ScrollDelta::Lines { x: _, y: dy } => {
                Message::WorkspaceDispatch(WorkspaceMessage::ScrollBy(dy))
            }
            ScrollDelta::Pixels { x: _, y: dy } => {
                Message::WorkspaceDispatch(WorkspaceMessage::ScrollBy(dy))
            }
        })
        .into()
    }
}

impl Default for Workspaces {
    fn default() -> Self {
        Self::new()
    }
}





fn get_active_workspace_id() -> usize {
    if let Ok(active_workspace) = hyprland::data::Workspace::get_active() {
        return active_workspace.id as usize;
    }

    1
}

async fn get_workspace_size() -> usize {
    let Ok(workspaces) = hyprland::data::Workspaces::get_async().await else {
        return 1;
    };

    workspaces
        .to_vec()
        .iter()
        .map(|x| x.id)
        .max()
        .expect("Can't get any workspace id.") as usize
}
