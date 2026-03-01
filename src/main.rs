use iced::widget::{button, center, column, row, slider, text, Space};
use iced::window;
use iced::{color, Center, Element, Fill, Font, Size, Task, Theme, Window};
use iced::theme::Palette;

use std::path::PathBuf;
use std::sync::OnceLock;

const FONT_BYTES: &[u8] = include_bytes!("../MouseMemoirs-Regular.ttf");
const FONT: Font = Font::with_name("Mouse Memoirs");

const RUBBERBAND_BIN: &[u8] = include_bytes!("../rubberband");

fn rubberband_path() -> &'static PathBuf {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let dir = std::env::temp_dir().join("rubberband-gui");
        std::fs::create_dir_all(&dir).unwrap();
        let name = if cfg!(windows) { "rubberband.exe" } else { "rubberband" };
        let path = dir.join(name);
        std::fs::write(&path, RUBBERBAND_BIN).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        path
    })
}

fn pink_theme() -> Theme {
    Theme::custom(
        "Pink".to_string(),
        Palette {
            background: color!(0xFFF0F5),
            text: color!(0x5C374C),
            primary: color!(0xE8A0BF),
            success: color!(0xA8D8B9),
            warning: color!(0xF2C57C),
            danger: color!(0xE06C75),
        },
    )
}

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Rubberband GUI")
        .theme(App::theme)
        .font(FONT_BYTES)
        .default_font(FONT)
        .window_size(Size::new(420.0, 620.0))
        .resizable(false)
        .centered()
        .run()
}

struct App {
    time: f64,
    pitch: i32,
    input_file: Option<PathBuf>,
    status: Status,
}

#[derive(Debug, Clone)]
enum Status {
    Idle,
    Processing,
    Done(String),
    Error(String),
}

#[derive(Debug, Clone)]
enum Message {
    TimeChanged(f64),
    ResetTime,
    PitchUp,
    PitchDown,
    PitchReset,
    PickFile,
    FilePicked(Option<PathBuf>),
    Process,
    ProcessDone(Result<String, String>),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                time: 1.0,
                pitch: 0,
                input_file: None,
                status: Status::Idle,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TimeChanged(value) => {
                self.time = (value * 100.0).round() / 100.0;
                Task::none()
            }
            Message::ResetTime => {
                self.time = 1.0;
                Task::none()
            }
            Message::PitchUp => {
                if self.pitch < 12 {
                    self.pitch += 1;
                }
                Task::none()
            }
            Message::PitchDown => {
                if self.pitch > -12 {
                    self.pitch -= 1;
                }
                Task::none()
            }
            Message::PitchReset => {
                self.pitch = 0;
                Task::none()
            }
            Message::PickFile => window::oldest()
                .and_then(|id| window::run(id, pick_file))
                .then(Task::future)
                .map(Message::FilePicked),
            Message::FilePicked(path) => {
                self.input_file = path;
                Task::none()
            }
            Message::Process => {
                let Some(input) = self.input_file.clone() else {
                    return Task::none();
                };
                self.status = Status::Processing;
                let time = self.time;
                let pitch = self.pitch;
                Task::perform(run_rubberband(input, time, pitch), Message::ProcessDone)
            }
            Message::ProcessDone(result) => {
                self.status = match result {
                    Ok(output) => Status::Done(format!("Done: {output}")),
                    Err(e) => Status::Error(e),
                };
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let title = text("Rubberband GUI")
            .size(34)
            .color(color!(0xC06C84))
            .align_x(Center);

        // Time section
        let time_label = text(format!("Time: {:.2}x", self.time))
            .size(24)
            .align_x(Center);

        let time_slider = slider(0.1..=3.0, self.time, Message::TimeChanged)
            .step(0.01)
            .width(320);

        let reset_time_btn = button(
            text("Reset 1x").size(16).align_x(Center),
        )
        .on_press(Message::ResetTime)
        .style(button::secondary)
        .padding([6, 16]);

        // Pitch section
        let pitch_label = text(format!("Pitch: {} st", self.pitch))
            .size(24)
            .align_x(Center);

        let pitch_controls = row![
            button(text("  -  ").size(22).align_x(Center))
                .on_press_maybe((self.pitch > -12).then_some(Message::PitchDown))
                .style(button::secondary)
                .padding([8, 20]),
            button(text("  0  ").size(16).align_x(Center))
                .on_press(Message::PitchReset)
                .style(button::secondary)
                .padding([8, 16]),
            button(text("  +  ").size(22).align_x(Center))
                .on_press_maybe((self.pitch < 12).then_some(Message::PitchUp))
                .style(button::secondary)
                .padding([8, 20]),
        ]
        .spacing(14)
        .align_y(Center);

        // File section
        let file_display = match &self.input_file {
            Some(path) => {
                let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                name
            }
            None => "No file selected".to_string(),
        };

        let pick_btn = button(
            text("Select File").size(20).align_x(Center),
        )
        .on_press(Message::PickFile)
        .style(button::primary)
        .padding([10, 24]);

        let file_label = text(file_display)
            .size(16)
            .color(color!(0x8B6F7E))
            .align_x(Center);

        // Process button
        let is_ready = self.input_file.is_some() && !matches!(self.status, Status::Processing);
        let process_btn = button(
            text("Process").size(22).align_x(Center),
        )
        .on_press_maybe(is_ready.then_some(Message::Process))
        .style(button::primary)
        .padding([12, 48]);

        // Status
        let status_text = match &self.status {
            Status::Idle => String::new(),
            Status::Processing => "Processing...".to_string(),
            Status::Done(msg) => msg.clone(),
            Status::Error(msg) => format!("Error: {msg}"),
        };

        let status_color = match &self.status {
            Status::Error(_) => color!(0xE06C75),
            Status::Done(_) => color!(0xA8D8B9),
            _ => color!(0x8B6F7E),
        };

        let content = column![
            title,
            Space::new().height(14),
            time_label,
            time_slider,
            reset_time_btn,
            Space::new().height(14),
            pitch_label,
            pitch_controls,
            Space::new().height(20),
            pick_btn,
            file_label,
            Space::new().height(20),
            process_btn,
            Space::new().height(8),
            text(status_text).size(15).color(status_color),
        ]
        .spacing(10)
        .padding(30)
        .align_x(Center)
        .width(Fill);

        center(content).into()
    }

    fn theme(&self) -> Theme {
        pink_theme()
    }
}

fn pick_file(window: &dyn Window) -> impl Future<Output = Option<PathBuf>> + use<> {
    let dialog = rfd::AsyncFileDialog::new()
        .set_title("Select audio file")
        .set_parent(&window);

    async move {
        dialog
            .pick_file()
            .await
            .map(|handle| handle.path().to_owned())
    }
}

fn build_output_path(input: &PathBuf, time: f64, pitch: i32) -> PathBuf {
    let stem = input.file_stem().unwrap().to_str().unwrap();
    let ext = input.extension().unwrap().to_str().unwrap();
    let parent = input.parent().unwrap();
    let suffix = format!("_t{:.2}_p{}", time, pitch);
    parent.join(format!("{stem}{suffix}.{ext}"))
}

async fn run_rubberband(input: PathBuf, time: f64, pitch: i32) -> Result<String, String> {
    let output = build_output_path(&input, time, pitch);

    let result = tokio::process::Command::new(rubberband_path())
        .arg("--time")
        .arg(format!("{:.2}", time))
        .arg("--pitch")
        .arg(format!("{}", pitch))
        .arg(&input)
        .arg(&output)
        .output()
        .await
        .map_err(|e| format!("Failed to run rubberband: {e}"))?;

    if result.status.success() {
        Ok(output.display().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        Err(format!("rubberband failed: {stderr}"))
    }
}
