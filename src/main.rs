use iced::font;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::widget::{rich_text, span};
use iced::{color, Font};
use iced::{widget, Task};
use std::path::PathBuf;

use proyecto_1::parser::*;
use proyecto_1::{emulator::Storage, error::Error};

fn main() -> iced::Result {
    iced::application("Emulator", Emulator::update, Emulator::view).run_with(Emulator::new)
}

#[derive(Default)]
struct Emulator {
    storage: Storage,
}

#[derive(Debug, Clone)]
enum Message {
    OpenFile,
    FilePicked(Result<Vec<PathBuf>, Error>),
    StoreFiles(Result<Vec<(String, Vec<u8>)>, Error>),
    DialogResult(rfd::MessageDialogResult),
}

impl Emulator {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                storage: Storage::new(525),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFile => Task::perform(pick_file(), Message::FilePicked),
            Message::FilePicked(Ok(paths)) => Task::perform(read_files(paths), Message::StoreFiles),
            Message::FilePicked(Err(error)) => {
                let dialog = rfd::AsyncMessageDialog::new()
                    .set_level(rfd::MessageLevel::Error)
                    .set_title("Error")
                    .set_description(format!("{}", error))
                    .set_buttons(rfd::MessageButtons::Ok)
                    .show();

                Task::perform(dialog, Message::DialogResult)
            }
            Message::StoreFiles(Ok(files)) => {
                for (file_name, data) in files {
                    println!("{:?}", read_file(&data));
                    let result = self.storage.store_files(&file_name, data.len(), data);
                    if let Err(error) = result {
                        let dialog = rfd::AsyncMessageDialog::new()
                            .set_level(rfd::MessageLevel::Warning)
                            .set_title("Memory Warning")
                            .set_description(format!("{}", error))
                            .set_buttons(rfd::MessageButtons::Ok)
                            .show();

                        return Task::perform(dialog, Message::DialogResult);
                    }
                }
                Task::none()
            }
            Message::StoreFiles(Err(error)) => {
                let dialog = rfd::AsyncMessageDialog::new()
                    .set_level(rfd::MessageLevel::Error)
                    .set_title("Error")
                    .set_description(format!("{}", error))
                    .set_buttons(rfd::MessageButtons::Ok)
                    .show();

                Task::perform(dialog, Message::DialogResult)
            }
            Message::DialogResult(_result) => Task::none(),
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let menu_bar = row![
            button("File").on_press(Message::OpenFile),
            widget::Space::new(iced::Length::Shrink, iced::Length::Fill)
        ]
        .height(40)
        .padding([5, 10]);
        let mut files = column![].padding([5, 10]);

        for (index, (file_name, _, _)) in self.storage.used.iter().enumerate() {
            files = files.push(rich_text([
                span(index).font(Font {
                    weight: font::Weight::Bold,
                    ..Font::default()
                }),
                span(" "),
                span(file_name),
            ]));
        }

        // Show the list of files
        let files_display = container(scrollable(files))
            .height(iced::Length::Fill)
            .width(220)
            .style(container::rounded_box);

        let mut memory = column![].padding([5, 10]);
        for (index, data) in self.storage.data.chunks(8).enumerate() {
            let mut spans =
                vec![span(format!("{:02X}", index))
                    .color(color!(0xff0000))
                    .font(Font {
                        weight: font::Weight::Bold,
                        ..Font::default()
                    })];
            spans.append(
                &mut data
                    .iter()
                    .map(|x| {
                        span(format!("\t{:02X}", x)).font(Font {
                            weight: font::Weight::Bold,
                            ..Font::default()
                        })
                    })
                    .collect::<Vec<_>>(),
            );
            memory = memory.push(rich_text(spans));
        }

        let memory_display = container(scrollable(memory).width(iced::Length::Fill))
            .height(iced::Length::Fill)
            .width(320)
            .style(container::rounded_box);

        widget::container(column![
            menu_bar,
            row![
                files_display,
                memory_display,
                widget::Space::new(iced::Length::Fill, iced::Length::Fill)
            ]
            .spacing(40)
            .padding([10, 10])
        ])
        .center_x(iced::Length::Fill)
        .center_y(iced::Length::Fill)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
    }
}

async fn read_files(files: Vec<PathBuf>) -> Result<Vec<(String, Vec<u8>)>, Error> {
    let mut files_content: Vec<(String, Vec<u8>)> = vec![];
    for path in files {
        let file_name = path.file_name();
        let file_name = file_name.ok_or(Error::NotFile)?;

        let contents = tokio::fs::read(&path)
            .await
            .map_err(|error| error.kind())
            .map_err(Error::IO)?;

        files_content.push((format!("{:?}", file_name).to_string(), contents));
    }

    Ok(files_content)
}

async fn pick_file() -> Result<Vec<PathBuf>, Error> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Choose a file...")
        .add_filter("Assembly File", &["asm"])
        .pick_files()
        .await
        .ok_or(Error::DialogClosed)?;

    Ok(handle
        .iter()
        .map(|f| f.path().to_owned())
        .collect::<Vec<PathBuf>>())
}