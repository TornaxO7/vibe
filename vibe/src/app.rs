// SPDX-License-Identifier:  GPL-2.0-only

use crate::config::Config;
use crate::fl;
use cosmic::app::{context_drawer, Core, Task};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::{Alignment, Length, Padding, Subscription};
use cosmic::widget::segmented_button::Entity;
use cosmic::widget::{self, menu, nav_bar, text_editor};
use cosmic::{cosmic_theme, theme, Application, ApplicationExt, Element};
use futures_util::SinkExt;
use notify::event::{CreateKind, RemoveKind};
use notify::{EventKind, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use tracing::{debug, error, info};
use vibe_daemon::config::{OutputConfig, ShaderCode};

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

type OutputName = String;

#[derive(Debug, Default, Clone)]
struct SectionAmountBars {
    input: String,
    is_valid: bool,
}

impl SectionAmountBars {
    // Just a guess
    const AMOUNT_MIN_BARS: NonZeroUsize = NonZeroUsize::new(10).unwrap();

    pub fn set_input(&mut self, new_input: String) {
        self.is_valid = new_input
            .parse::<NonZeroUsize>()
            .map(|num| num >= Self::AMOUNT_MIN_BARS)
            .unwrap_or(false);
        self.input = new_input;
    }
}

impl From<&OutputConfig> for SectionAmountBars {
    fn from(value: &OutputConfig) -> Self {
        Self {
            input: value.amount_bars.to_string(),
            is_valid: true,
        }
    }
}

/*
#[derive(Debug, Default, Clone)]
struct SectionFrequencyRange {
    min: String,
    max: String,
    is_valid: bool,
}

impl SectionFrequencyRange {
    const MIN_FREQ: NonZeroUsize = NonZeroUsize::new(1).unwrap();
    const MAX_FREQ: NonZeroUsize = NonZeroUsize::new(20_000).unwrap();

    pub fn set_min(&mut self, new_input: String) {
        self.is_valid = new_input
            .parse::<NonZeroUsize>()
            .map(|num| {
                let is_within_valid_range = (Self::MIN_FREQ..Self::MAX_FREQ).contains(&num);
                let is_smaller_than_max = self
                    .max
                    .parse::<NonZeroUsize>()
                    .map(|max| num < max)
                    .unwrap_or(false);

                is_within_valid_range && is_smaller_than_max
            })
            .unwrap_or(false);

        self.min = new_input;
    }

    pub fn set_max(&mut self, new_input: String) {
        self.is_valid = new_input
            .parse::<NonZeroUsize>()
            .map(|num| {
                let is_within_valid_range = (Self::MIN_FREQ..Self::MAX_FREQ).contains(&num);
                let is_bigger_than_min = self
                    .min
                    .parse::<NonZeroUsize>()
                    .map(|min| num > min)
                    .unwrap_or(false);

                is_within_valid_range && is_bigger_than_min
            })
            .unwrap_or(false);

        self.max = new_input;
    }
}

impl From<&OutputConfig> for SectionFrequencyRange {
    fn from(value: &OutputConfig) -> Self {
        Self {
            min: value.frequency_range.start.to_string(),
            max: value.frequency_range.end.to_string(),
            is_valid: true,
        }
    }
}
*/

#[derive(Debug, Default)]
struct SectionShaderCode {
    selected: usize,
    code: text_editor::Content,
}

impl SectionShaderCode {
    const SELECTIONS: &[&str] = &["WGSL", "GLSL"];
}

impl From<&OutputConfig> for SectionShaderCode {
    fn from(value: &OutputConfig) -> Self {
        if let Some(code) = &value.shader_code {
            match code {
                ShaderCode::Wgsl(code) => Self {
                    selected: 0,
                    code: text_editor::Content::with_text(code),
                },
                ShaderCode::Glsl(code) => Self {
                    selected: 1,
                    code: text_editor::Content::with_text(code),
                },
            }
        } else {
            Self {
                code: text_editor::Content::new(),
                selected: 0,
            }
        }
    }
}

/// Holds the state of each input field within the config page of an output.
#[derive(Debug, Default)]
struct OutputSectionState {
    amount_bars: SectionAmountBars,
    editor: SectionShaderCode,
}

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// Contains items assigned to the nav bar panel.
    nav: nav_bar::Model,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    // Configuration data that persists between application runs.
    config: Config,

    output_section_state: OutputSectionState,

    nav_ids: HashMap<OutputName, Entity>,
    output_configs: HashMap<Entity, OutputConfig>,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    OpenRepositoryUrl,
    ToggleContextPage(ContextPage),
    UpdateConfig(Config),
    LaunchUrl(String),
    AddConfigs(Vec<OutputName>),
    RemoveConfigs(Vec<OutputName>),

    SetAmountBars(String),
    // SetMinFreq(String),
    // SetMaxFreq(String),
    EditShaderCode(text_editor::Action),
    SelectShaderLang(usize),
    Todo,
}

/// Create a COSMIC application from the app model
impl Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "de.tornaxo7.vibe";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        // nav.insert()
        //     .text(fl!("page-id", num = 3))
        //     .data::<Page>(Page::Page3)
        //     .icon(icon::from_name("applications-games-symbolic"));

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            nav: nav_bar::Model::default(),
            nav_ids: HashMap::new(),
            key_binds: HashMap::new(),
            output_configs: HashMap::new(),

            output_section_state: OutputSectionState::default(),

            // Optional configuration file for an application.
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
        };

        let startup_commands = {
            let set_window_title = app.update_title();

            let collect_configs = Task::future(async {
                let config_dir = vibe_daemon::config_directory();

                let dir_walker = match std::fs::read_dir(config_dir) {
                    Ok(walker) => walker,
                    Err(err) => {
                        panic!("Couldn't create directory walker to load the configs frorm the output-config directory: {}", err);
                    }
                };

                let output_names: Vec<String> = dir_walker
                    .into_iter()
                    .filter(|entry| entry.is_ok())
                    .map(|entry| entry.unwrap().path())
                    .filter(|path| path.is_file())
                    .map(|path| path.file_stem().unwrap().to_string_lossy().into_owned())
                    .collect();

                cosmic::app::message::app(Message::AddConfigs(output_names))
            });

            set_window_title.chain(collect_configs)
        };

        (app, startup_commands)
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    /// Enables the COSMIC application to create a nav bar with this model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::context_drawer(
                self.about(),
                Message::ToggleContextPage(ContextPage::About),
            )
            .title(fl!("about")),
        })
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<Self::Message> {
        let column = widget::column().width(Length::Fill).height(Length::Fill);

        let id = self.nav.active();
        let an_output_is_selected = self.output_configs.get(&id).is_some();
        let column = if an_output_is_selected {
            let amount_bars = {
                let state = self.output_section_state.amount_bars.clone();

                let mut amount_bars = widget::text_input("Amount bars", state.input)
                    .label(format!("Amount of bars (>= {})", SectionAmountBars::AMOUNT_MIN_BARS))
                    .on_submit(Message::Todo)
                    .on_input(move |new_input| Message::SetAmountBars(new_input))
                    .padding(Padding::new(10.))
                    .helper_text("Set the amount of bars which should be passed to the shader (in order to display them).");

                if !state.is_valid {
                    amount_bars = amount_bars.error("Your input isn't a positive integer!");
                }

                amount_bars
            };

            // let freq_range = {
            //     let state = self.output_section_state.freq_range.clone();

            //     let min_text = {
            //         let mut min_text: TextInput<'_, Message> =
            //             widget::text_input("Minimum (default: 50 (Hz))", state.min)
            //                 .on_input(move |input| Message::SetMinFreq(input))
            //                 .label("Min frequency (>= 20 (Hz))");

            //         if !state.is_valid {
            //             min_text = min_text.error("Invalid value");
            //         }

            //         min_text
            //     };

            //     let max_text = {
            //         let mut max_text: TextInput<'_, Message> =
            //             widget::text_input("Maximum (default 20.000 (Hz))", state.max)
            //                 .on_input(move |input| Message::SetMaxFreq(input))
            //                 .label("Max frequency (<= 20.000 (Hz))");

            //         if !state.is_valid {
            //             max_text = max_text.error("Invalid value");
            //         }

            //         max_text
            //     };

            //     let freq_range = widget::flex_row(vec![
            //         widget::text::text("Frequency range").into(),
            //         min_text.into(),
            //         max_text.into(),
            //     ]);

            //     freq_range
            // };

            let shader_editor = {
                let state = &self.output_section_state.editor;

                widget::text_editor(&state.code)
                    .placeholder("Write the shader!")
                    .on_action(Message::EditShaderCode)
            };

            column
                .push(amount_bars)
                .push(widget::divider::horizontal::default())
                // .push(freq_range)
                // .push(widget::divider::horizontal::default())
                .push(widget::divider::horizontal::default())
                .push(shader_editor)
        } else {
            let title = widget::text::title1("hello there")
                .width(Length::Fill)
                .center();

            column.push(title)
        };

        column.into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They are started at the
    /// beginning of the application, and persist through its lifetime.
    fn subscription(&self) -> Subscription<Self::Message> {
        struct OutputListener;

        Subscription::batch(vec![
            // Create a subscription which emits updates through a channel.
            Subscription::run_with_id(
                std::any::TypeId::of::<OutputListener>(),
                cosmic::iced::stream::channel(4, {
                    move |mut channel| {
                        // look after output configs which are getting added or removed by the daemon
                        let mut watcher = notify::recommended_watcher(
                            move |watcher_event: notify::Result<notify::Event>| match watcher_event
                            {
                                Ok(event) => {
                                    if event.kind == EventKind::Create(CreateKind::File) {
                                        info!("Found new output configs: {:#?}", &event.paths);
                                        let output_names = event
                                            .paths
                                            .into_iter()
                                            .filter(|path| path.is_file())
                                            .map(|path| {
                                                path.file_stem()
                                                    .unwrap()
                                                    .to_string_lossy()
                                                    .into_owned()
                                            })
                                            .collect();

                                        let _ = channel.send(Message::AddConfigs(output_names));
                                    } else if event.kind == EventKind::Remove(RemoveKind::File) {
                                        info!("Output got removed at {:#?}", &event.paths);
                                        let output_names = event
                                            .paths
                                            .into_iter()
                                            .filter(|path| path.is_file())
                                            .map(|path| {
                                                path.file_stem()
                                                    .unwrap()
                                                    .to_string_lossy()
                                                    .into_owned()
                                            })
                                            .collect();

                                        let _ = channel.send(Message::RemoveConfigs(output_names));
                                    }
                                }
                                Err(err) => {
                                    error!("Watch error: {}", err);
                                }
                            },
                        )
                        .expect("Create output config listener");

                        let config_path = vibe_daemon::config_directory();

                        async move {
                            debug!("Start watcher in directory: {:?}", &config_path);
                            watcher
                                .watch(&config_path, RecursiveMode::NonRecursive)
                                .unwrap();

                            futures_util::future::pending().await
                        }
                    }
                }),
            ),
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ])
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::OpenRepositoryUrl => {
                _ = open::that_detached(REPOSITORY);
            }

            Message::Todo => {
                todo!()
            }

            Message::EditShaderCode(action) => {
                self.output_section_state.editor.code.perform(action)
            }
            Message::SelectShaderLang(index) => self.output_section_state.editor.selected = index,
            Message::SetAmountBars(input) => {
                self.output_section_state.amount_bars.set_input(input);
            }
            // Message::SetMinFreq(input) => self.output_section_state.freq_range.set_min(input),
            // Message::SetMaxFreq(input) => self.output_section_state.freq_range.set_max(input),
            Message::AddConfigs(output_names) => {
                for output_name in output_names {
                    let (config, _config_path) = match vibe_daemon::config::load(&output_name) {
                        Ok(config) => config.expect("Output name has config"),
                        Err(err) => {
                            error!("Couldn't load config of output '{}': {}", output_name, err);
                            continue;
                        }
                    };

                    let id = self.nav.insert().text(output_name.clone()).id();
                    self.nav_ids.insert(output_name.clone(), id);
                    self.output_configs.insert(id, config);
                }
            }
            Message::RemoveConfigs(output_names) => {
                for output_name in output_names {
                    let id = self.nav_ids.remove(&output_name).unwrap();
                    self.nav.remove(id);
                    self.output_configs.remove(&id);
                }
            }

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },
        }

        Task::none()
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        // Activate the page in the model.
        self.nav.activate(id);

        if let Some(config) = self.output_configs.get(&id) {
            self.output_section_state.amount_bars = SectionAmountBars::from(config);
            // self.output_section_state.freq_range = SectionFrequencyRange::from(config);
            self.output_section_state.editor = SectionShaderCode::from(config);
        }

        self.update_title()
    }
}

impl AppModel {
    /// The about page for this app.
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let icon = widget::svg(widget::svg::Handle::from_memory(APP_ICON));

        let title = widget::text::title3(fl!("app-title"));

        let hash = env!("VERGEN_GIT_SHA");
        let short_hash: String = hash.chars().take(7).collect();
        let date = env!("VERGEN_GIT_COMMIT_DATE");

        let link = widget::button::link(REPOSITORY)
            .on_press(Message::OpenRepositoryUrl)
            .padding(0);

        widget::column()
            .push(icon)
            .push(title)
            .push(link)
            .push(
                widget::button::link(fl!(
                    "git-description",
                    hash = short_hash.as_str(),
                    date = date
                ))
                .on_press(Message::LaunchUrl(format!("{REPOSITORY}/commits/{hash}")))
                .padding(0),
            )
            .align_x(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<Message> {
        let mut window_title = fl!("app-title");

        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
        }

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}
