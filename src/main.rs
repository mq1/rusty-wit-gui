// SPDX-FileCopyrightText: 2023 Manuel Quarneti <hi@mq1.eu>
// SPDX-License-Identifier: GPL-3.0-only

mod drives;
mod style;
mod util;

use std::path::PathBuf;

use drives::Drive;
use iced::{
    executor,
    widget::{button, column, container, horizontal_space, pick_list, row, text, vertical_space},
    Application, Command, Element, Length, Settings, Theme,
};
use util::Game;

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

#[derive(Debug, Clone)]
enum View {
    DriveSelection,
    Games(Drive),
    Progress(String),
}

struct App {
    view: View,
    drives: Vec<Drive>,
    selected_drive: Option<Drive>,
}

#[derive(Debug, Clone)]
enum Message {
    DriveSelected(Drive),
    OpenDrive,
    AddGames,
    AddingGames((Vec<PathBuf>, usize)),
    DeleteGame(Game),
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let drives = drives::list().unwrap();
        let first_drive = drives[0].clone();

        (
            Self {
                view: View::DriveSelection,
                drives,
                selected_drive: Some(first_drive),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("rusty-wit-gui")
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::DriveSelected(drive) => {
                self.selected_drive = Some(drive);
            }
            Message::OpenDrive => {
                if let Some(drive) = &self.selected_drive {
                    let wit_path = util::get_wit_path(&drive.mount_point).unwrap();
                    if !wit_path.exists() {
                        self.view = View::Progress(String::from("Downloading wit..."));

                        let mount_point = drive.mount_point.clone();
                        return Command::perform(
                            async move { util::download_wit(&mount_point).unwrap() },
                            |_| Message::OpenDrive,
                        );
                    }

                    self.view = View::Games(drive.clone());
                }
            }
            Message::AddGames => {
                self.view = View::Progress(String::from("Adding games..."));

                return Command::perform(
                    async {
                        let games = util::select_games();

                        (games, 0)
                    },
                    Message::AddingGames,
                );
            }
            Message::AddingGames((games, index)) => {
                let drive = self.selected_drive.as_ref().unwrap().to_owned();
                let len = games.len();

                if index == len {
                    self.view = View::Games(drive);
                    return Command::none();
                } else {
                    let text = format!("Adding game {} of {}", index + 1, len);
                    self.view = View::Progress(text);

                    return Command::perform(
                        async move {
                            util::add_game(&drive.mount_point, &games[index]).unwrap();

                            (games, index + 1)
                        },
                        Message::AddingGames,
                    );
                }
            }
            Message::DeleteGame(game) => {
                if let Some(drive) = &self.selected_drive {
                    util::remove_game(&drive.mount_point, &game).unwrap();
                    return self.update(Message::OpenDrive);
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        match &self.view {
            View::DriveSelection => {
                let pick_list = pick_list(
                    &self.drives,
                    self.selected_drive.clone(),
                    Message::DriveSelected,
                );

                let open_drive_button = button("Open").on_press(Message::OpenDrive);

                column![
                    vertical_space(Length::Fill),
                    row![
                        horizontal_space(Length::Fill),
                        text("Select a drive").size(30),
                        horizontal_space(Length::Fill)
                    ],
                    row![
                        horizontal_space(Length::Fill),
                        pick_list,
                        open_drive_button,
                        horizontal_space(Length::Fill),
                    ]
                    .spacing(10),
                    vertical_space(Length::Fill),
                ]
                .padding(10)
                .spacing(10)
                .into()
            }
            View::Games(drive) => {
                let games = util::get_games(&drive.mount_point).unwrap();

                let list = column(
                    games
                        .iter()
                        .map(|game| {
                            container(
                                row![
                                    text(format!(
                                        "{}: {} ({} GiB)",
                                        game.id, game.title, game.size
                                    )),
                                    horizontal_space(Length::Fill),
                                    button("Delete").on_press(Message::DeleteGame(game.clone())),
                                ]
                                .padding(10),
                            )
                            .style(style::card())
                            .into()
                        })
                        .collect(),
                );

                column![
                    text(format!("Games on {}", drive.name)).size(30),
                    list,
                    vertical_space(Length::Fill),
                    row![
                        text(format!(
                            "Using {}/{} GiB",
                            drive.available_space, drive.total_space
                        )),
                        horizontal_space(Length::Fill),
                        button("Add game(s)").on_press(Message::AddGames),
                    ],
                ]
                .padding(10)
                .spacing(10)
                .into()
            }
            View::Progress(progress) => column![
                vertical_space(Length::Fill),
                row![
                    horizontal_space(Length::Fill),
                    text(progress).size(30),
                    horizontal_space(Length::Fill)
                ],
                vertical_space(Length::Fill),
            ]
            .padding(10)
            .spacing(10)
            .into(),
        }
    }
}
