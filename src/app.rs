// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::{window::Id, Limits, Subscription};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::prelude::*;
use cosmic::widget;
use cosmic::iced::Length;
use futures_util::SinkExt;
use serde::Deserialize;
use serde_json;
use cosmic::iced::widget::container;
use cosmic::iced::Alignment;

#[derive(Debug, Clone, Deserialize)]
pub struct MenuItem {
    pub title: String,

    #[serde(rename = "type")]
    pub item_type: Option<String>,

    pub command: Option<String>,
    pub submenu: Option<Vec<MenuItem>>,
    pub icon: Option<String>,
    pub terminal: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct MenuConfig {
    pub menu: Vec<MenuItem>,
    pub icon: Option<String>,
    pub terminal: Option<String>,
}

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
#[derive(Default)]
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// The popup id.
    popup: Option<Id>,
    /// Configuration data that persists between application runs.
    config: Config,
    /// Example row toggler.
    example_row: bool,
    menus: Vec<MenuItem>,
    applet_icon: String,
    expanded: Option<String>, // controla submenu aberto
    hovered: Option<String>,
    terminal: String,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    SubscriptionChannel,
    UpdateConfig(Config),
    ToggleExampleRow(bool),
    MenuClicked(String),
    HoverItem(String),
    UnhoverItem,
}


fn load_menu() -> MenuConfig {
    let user_path = user_config_path();

    let default_data = include_str!("../resources/commands.json");

    if !user_path.exists() {
        std::fs::write(&user_path, default_data)
            .expect("failed to write default config");
    }

    let data = std::fs::read_to_string(&user_path)
        .expect("failed to read user json");

    serde_json::from_str(&data)
        .unwrap_or_else(|_| serde_json::from_str(default_data).unwrap())
}

fn menu_item_style(
    theme: &cosmic::Theme,
    is_hovered: bool,
) -> container::Style {
    let mut style = container::Style::default();

    if is_hovered {
        style.background = Some(
            cosmic::iced::Background::Color(
                theme.cosmic().bg_component_color().into()
            )
        );
    }

    style
}

fn user_config_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").expect("no HOME");
    let dir = std::path::PathBuf::from(home)
        .join(".config/commands-applet");

    std::fs::create_dir_all(&dir).ok();

    dir.join("commands.json")
}

fn run_command_direct(command: &str) {
    if let Err(e) = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "setsid sh -c '{}' >/dev/null 2>&1 < /dev/null &",
            command.replace("'", "'\\''")
        ))
        .spawn()
    {
        eprintln!("Failed to run command: {}", e);
    }
}

fn open_terminal(terminal: &str, command: &str) {
    let cmd = format!(
        "{}; status=$?; [ $status -ne 0 ] && read -p 'Error...'; exec bash",
        command
    );

    let try_spawn = |prog: &str, args: &[&str]| {
        match std::process::Command::new(prog).args(args).spawn() {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Failed to launch {}: {}", prog, e);
                false
            }
        }
    };

    match terminal {
        "gnome-terminal" => {
            if try_spawn("gnome-terminal", &["--", "bash", "-c", &cmd]) {
                return;
            }
        }
        "xterm" => {
            if try_spawn("xterm", &["-e", &cmd]) {
                return;
            }
        }
        "konsole" => {
            if try_spawn("konsole", &["-e", &cmd]) {
                return;
            }
        }
        "auto" | _ => {
            let fallbacks = [
                ("gnome-terminal", vec!["--", "bash", "-c", &cmd]),
                ("xterm", vec!["-e", &cmd]),
                ("konsole", vec!["-e", &cmd]),
            ];

            for (term, args) in fallbacks {
                if try_spawn(term, &args) {
                    return;
                }
            }
        }
    }

    eprintln!("No terminal available");
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "com.github.vinesnts.commands-applet";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let config = load_menu();
        let icon = config.icon
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "display-symbolic".to_string());

        // Construct the app model with the runtime's core.
        let app = AppModel {
            core,
            menus: config.menu.clone(),
            applet_icon: icon,
            terminal: config.terminal.unwrap_or_else(|| "auto".to_string()),
            expanded: None,
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => {
                        // for why in errors {
                        //     tracing::error!(%why, "error loading app config");
                        // }

                        config
                    }
                })
                .unwrap_or_default(),
            ..Default::default()
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// The applet's button in the panel will be drawn using the main view method.
    /// This view should emit messages to toggle the applet's popup window, which will
    /// be drawn using the `view_window` method.
    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button::<Message>(&self.applet_icon)
            .on_press(Message::TogglePopup)
            .into()
    }

    /// The applet's popup window will be drawn using this view method. If there are
    /// multiple poups, you may match the id parameter to determine which popup to
    /// create a view for.
    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let content = self.render_menu(&self.menus, 0, None);

        self.core.applet.popup_container(content).into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-lived async tasks running in the background which
    /// emit messages to the application through a channel. They may be conditionally
    /// activated by selectively appending to the subscription batch, and will
    /// continue to execute for the duration that they remain in the batch.
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::run(|| {
            cosmic::iced::stream::channel(
                4,
                |mut channel: cosmic::iced::futures::channel::mpsc::Sender<Message>| async move {
                    let _ = channel.send(Message::SubscriptionChannel).await;
                    futures_util::future::pending::<()>().await;
                },
            )
        })
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime. The application will not exit until all
    /// tasks are finished.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::SubscriptionChannel => {
                // For example purposes only.
            }
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::MenuClicked(path) => {
                if path == "__edit__" {
                    let path = user_config_path();

                    let cmd = format!("nano {}; exit", path.display());
                    open_terminal(&self.terminal, &cmd);
                    return Task::none();
                }

                if path == "__reload__" {
                    let config = load_menu();
                    self.menus = config.menu;
                    self.applet_icon = config.icon.unwrap_or_else(|| "display-symbolic".to_string());
                    self.terminal = config.terminal.unwrap_or_else(|| "auto".to_string());
                    return Task::none();
                }

                match Self::find_item_by_path(&self.menus, &path) {
                    Some(item) => {

                        if item.submenu.is_some() {

                            if self.expanded.as_ref() == Some(&path) {
                                self.expanded = None;
                            } else {
                                self.expanded = Some(path);
                            }

                            return Task::none();
                        }

                        if let Some(command) = &item.command {
                            let use_terminal = item.terminal.unwrap_or(true);

                            if use_terminal {
                                open_terminal(&self.terminal, command);
                            } else {
                                run_command_direct(command);
                            }
                        }
                    }
                    None => { }
                }
            }
            Message::ToggleExampleRow(toggled) => self.example_row = toggled,
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::HoverItem(title) => {
                self.hovered = Some(title);
            }
            Message::UnhoverItem => {
                self.hovered = None;
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}


impl AppModel {
    fn render_menu<'a>(
        &self,
        items: &'a [MenuItem],
        depth: usize,
        parent: Option<String>,
    ) -> widget::Column<'a, Message> {
        let mut col = widget::column();

        col = col.push(
            widget::container(widget::text(""))
                .height(8)
        );

        for item in items {

            let icon: Element<_> = if let Some(icon_name) = &item.icon {
                self.core.applet.icon_button::<Message>(icon_name).into()
            } else {
                self.core.applet.icon_button("text-x-generic").into()
            };

            let path = if let Some(parent) = &parent {
                format!("{}/{}", parent, item.title)
            } else {
                item.title.clone()
            };
            let is_hovered = self.hovered.as_ref() == Some(&path);

            let item_view = widget::mouse_area(
                widget::container(
                    widget::row()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(
                        widget::container(icon)
                            .width((34 + depth * 16) as u16)
                            .height(30)
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center)
                            .padding([0, 0, 0, (depth * 16) as u16])
                    )
                    .push(
                        widget::container(widget::text(&item.title))
                            .width(Length::Fill)
                    )
                    .width(Length::Fill)
                )
                .width(Length::Fill)
                .height(34)
                .align_y(Alignment::Center)
                .padding([2, 10])
                .style(move |theme| menu_item_style(theme, is_hovered))
            )
            .on_enter(Message::HoverItem(path.clone()))
            .on_exit(Message::UnhoverItem)
            .on_press(Message::MenuClicked(path.clone()));

            col = col.push(item_view);

            if let Some(sub) = &item.submenu {
                if self.expanded.as_ref() == Some(&path) {
                    let sub_col = self.render_menu(sub, depth + 1, Some(path.clone()));
                    col = col.push(sub_col);
                }
            }
        }

        if depth == 0 {
            col = col.push(
                widget::container(widget::text(""))
                    .height(1)
                    .width(Length::Fill)
                    .style(|theme| {
                        let mut style = widget::container::Style::default();
                        style.background = Some(
                            cosmic::iced::Background::Color(
                                theme.cosmic().bg_divider().into()
                            )
                        );
                        style
                    })
            );

            let path = "__edit__".to_string();

            let icon: Element<Message> = self
                .core
                .applet
                .icon_button::<Message>("accessories-text-editor-symbolic")
                .into();

            let is_hovered = self.hovered.as_ref() == Some(&path);

            let item_view = widget::mouse_area(
                widget::container(
                    widget::row()
                        .spacing(8)
                        .align_y(Alignment::Center)
                        .push(
                            widget::container(icon)
                                .width(34)
                                .height(30)
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center)
                        )
                        .push(
                            widget::container(widget::text("Edit commands.json"))
                                .width(Length::Fill)
                        )
                )
                .width(Length::Fill)
                .height(34)
                .align_y(Alignment::Center)
                .padding([2, 10])
                .style(move |theme| menu_item_style(theme, is_hovered))
            )
            .on_enter(Message::HoverItem(path.clone()))
            .on_exit(Message::UnhoverItem)
            .on_press(Message::MenuClicked(path));

            col = col.push(item_view);

            let reload_path = "__reload__".to_string();

            let icon: Element<Message> = self
                .core
                .applet
                .icon_button::<Message>("view-refresh-symbolic")
                .into();

            let is_hovered = self.hovered.as_ref() == Some(&reload_path);

            let reload_item = widget::mouse_area(
                widget::container(
                    widget::row()
                        .spacing(8)
                        .align_y(Alignment::Center)
                        .push(
                            widget::container(icon)
                                .width(34)
                                .height(30)
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center)
                        )
                        .push(
                            widget::container(widget::text("Reload"))
                                .width(Length::Fill)
                        )
                )
                .width(Length::Fill)
                .height(34)
                .align_y(Alignment::Center)
                .padding([2, 10])
                .style(move |theme| menu_item_style(theme, is_hovered))
            )
            .on_enter(Message::HoverItem(reload_path.clone()))
            .on_exit(Message::UnhoverItem)
            .on_press(Message::MenuClicked(reload_path));

            col = col.push(reload_item);
        }

        col = col.push(
            widget::container(widget::text(""))
                .height(8)
        );
        col
    }

    fn find_item_by_path<'a>(
        items: &'a [MenuItem],
        path: &str,
    ) -> Option<&'a MenuItem> {
        let mut parts = path.split('/');

        let mut current_items = items;
        let mut current_item = None;

        while let Some(part) = parts.next() {
            let item = match current_items.iter().find(|i| i.title == part) {
                Some(i) => i,
                None => return None,
            };

            current_item = Some(item);

            // só desce se ainda houver partes
            if parts.clone().next().is_some() {
                if let Some(sub) = &item.submenu {
                    current_items = sub;
                } else {
                    return None;
                }
            }
        }

        current_item
    }
}