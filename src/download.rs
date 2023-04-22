// SPDX-FileCopyrightText: 2023 Manuel Quarneti <hi@mq1.eu>
// SPDX-License-Identifier: GPL-3.0-only

use iced_native::subscription;

use std::{
    fs::File,
    hash::Hash,
    io::{Read, Write},
    path::PathBuf,
};

// Just a little utility function
pub fn file<I: 'static + Hash + Copy + Send + Sync, T: ToString>(
    id: I,
    url: T,
) -> iced::Subscription<(I, Progress)> {
    subscription::unfold(
        id,
        State::Ready(url.to_string(), url.to_string()),
        move |state| download(id, state),
    )
}

#[derive(Debug, Hash, Clone)]
pub struct Download<I> {
    id: I,
    url: String,
    path: PathBuf,
}

async fn download<I: Copy>(id: I, state: State) -> ((I, Progress), State) {
    match state {
        State::Ready(url, path) => {
            let response = ureq::get(&url).call();

            match response {
                Ok(response) => {
                    let total = response.header("Content-Length").unwrap_or("0").parse::<usize>().unwrap();

                    if total != 0 {
                        (
                            (id, Progress::Started),
                            State::Downloading {
                                reader: response.into_reader(),
                                file: File::create(path).unwrap(),
                                total,
                                downloaded: 0,
                            },
                        )
                    } else {
                        ((id, Progress::Errored), State::Finished)
                    }
                }
                Err(_) => ((id, Progress::Errored), State::Finished),
            }
        }
        State::Downloading {
            mut reader,
            mut file,
            total,
            downloaded,
        } => {
            let mut buffer = Vec::with_capacity(1024);
            let read_bytes = reader.read(&mut buffer);
            file.write_all(&buffer).unwrap();

            match read_bytes {
                Ok(bytes) => {
                    let downloaded = downloaded + bytes;
                    let percentage = (downloaded as f32 / total as f32) * 100.0;

                    (
                        (id, Progress::Advanced(percentage)),
                        State::Downloading {
                            reader,
                            file,
                            total,
                            downloaded,
                        },
                    )
                }
                Err(_) => ((id, Progress::Errored), State::Finished),
            }
        }
        State::Finished => {
            // We do not let the stream die, as it would start a
            // new download repeatedly if the user is not careful
            // in case of errors.
            iced::futures::future::pending().await
        }
    }
}

#[derive(Debug, Clone)]
pub enum Progress {
    Started,
    Advanced(f32),
    Finished,
    Errored,
}

pub enum State {
    Ready(String, String),
    Downloading {
        reader: Box<dyn Read + Send + Sync>,
        file: File,
        total: usize,
        downloaded: usize,
    },
    Finished,
}
