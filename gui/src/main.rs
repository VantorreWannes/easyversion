use std::path::PathBuf;

use directories::ProjectDirs;
use easyversion::{
    APPLICATION, ORGANIZATION, QUALIFIER,
    operations::{Version, clean, history, save, split},
    store::FileStore,
};
use iced::{
    Alignment, Background, Border, Color, Element, Length, Padding, Task, Theme,
    widget::{Column, Space, button, column, container, row, rule, scrollable, text, text_input},
};

const BG_SIDEBAR: Color = Color::from_rgb(0.12, 0.12, 0.13);
const BG_MAIN: Color = Color::from_rgb(0.16, 0.16, 0.18);
const BG_CARD: Color = Color::from_rgb(0.20, 0.20, 0.22);
const BG_CARD_HOVER: Color = Color::from_rgb(0.24, 0.24, 0.26);
const BORDER_COLOR: Color = Color::from_rgb(0.28, 0.28, 0.30);

const TEXT_PRIMARY: Color = Color::from_rgb(0.95, 0.95, 0.95);
const TEXT_MUTED: Color = Color::from_rgb(0.65, 0.65, 0.68);

const ACCENT_BRAND: Color = Color::from_rgb(0.40, 0.35, 0.95);
const ACCENT_BRAND_HOVER: Color = Color::from_rgb(0.48, 0.43, 1.0);

const ACCENT_DANGER: Color = Color::from_rgb(0.90, 0.35, 0.35);
const ACCENT_DANGER_HOVER: Color = Color::from_rgb(1.0, 0.45, 0.45);

const COLOR_ADDED: Color = Color::from_rgb(0.30, 0.85, 0.50);
const COLOR_REMOVED: Color = Color::from_rgb(0.95, 0.40, 0.40);
const COLOR_MODIFIED: Color = Color::from_rgb(0.95, 0.75, 0.30);

macro_rules! style_container {
    ($name:ident, $bg:expr, $bc:expr, $bw:expr, $br:expr) => {
        fn $name(_: &Theme) -> container::Style {
            container::Style {
                background: Some(Background::Color($bg)),
                border: Border {
                    color: $bc,
                    width: $bw,
                    radius: $br.into(),
                },
                text_color: Some(TEXT_PRIMARY),
                ..Default::default()
            }
        }
    };
}

macro_rules! style_btn {
    ($name:ident, $bg:expr, $tc:expr, $bc:expr, $bw:expr, $hbg:expr, $htc:expr, $hbc:expr) => {
        fn $name(_: &Theme, status: button::Status) -> button::Style {
            let base = button::Style {
                background: Some(Background::Color($bg)),
                text_color: $tc,
                border: Border {
                    color: $bc,
                    width: $bw,
                    radius: 8.0.into(),
                },
                ..Default::default()
            };
            match status {
                button::Status::Hovered => button::Style {
                    background: Some(Background::Color($hbg)),
                    text_color: $htc,
                    border: Border {
                        color: $hbc,
                        width: $bw,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                },
                button::Status::Disabled => button::Style {
                    background: Some(Background::Color(Color::from_rgb(0.25, 0.25, 0.25))),
                    text_color: Color::from_rgb(0.6, 0.6, 0.6),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                },
                _ => base,
            }
        }
    };
}

style_container!(style_sidebar_bg, BG_SIDEBAR, Color::TRANSPARENT, 0.0, 0.0);
style_container!(style_main_bg, BG_MAIN, Color::TRANSPARENT, 0.0, 0.0);
style_container!(style_card, BG_CARD, BORDER_COLOR, 1.0, 12.0);
style_container!(
    style_card_loading,
    Color::from_rgb(0.18, 0.24, 0.35),
    Color::from_rgb(0.35, 0.45, 0.70),
    1.0,
    8.0
);
style_container!(
    style_diff_bg,
    Color::from_rgb(0.14, 0.14, 0.15),
    Color::TRANSPARENT,
    0.0,
    8.0
);
style_container!(
    style_card_danger,
    Color::from_rgb(0.22, 0.12, 0.12),
    ACCENT_DANGER,
    1.0,
    12.0
);
style_container!(
    style_card_dashed,
    Color::TRANSPARENT,
    BORDER_COLOR,
    2.0,
    16.0
);

style_btn!(
    style_btn_primary,
    ACCENT_BRAND,
    Color::WHITE,
    Color::TRANSPARENT,
    0.0,
    ACCENT_BRAND_HOVER,
    Color::WHITE,
    Color::TRANSPARENT
);
style_btn!(
    style_btn_secondary,
    Color::TRANSPARENT,
    TEXT_PRIMARY,
    BORDER_COLOR,
    1.0,
    BORDER_COLOR,
    TEXT_PRIMARY,
    BORDER_COLOR
);
style_btn!(
    style_btn_danger,
    ACCENT_DANGER,
    Color::WHITE,
    Color::TRANSPARENT,
    0.0,
    ACCENT_DANGER_HOVER,
    Color::WHITE,
    Color::TRANSPARENT
);
style_btn!(
    style_btn_ghost,
    Color::TRANSPARENT,
    TEXT_MUTED,
    Color::TRANSPARENT,
    0.0,
    BG_CARD,
    TEXT_PRIMARY,
    Color::TRANSPARENT
);
style_btn!(
    style_btn_ghost_active,
    Color::from_rgb(0.20, 0.18, 0.35),
    TEXT_PRIMARY,
    ACCENT_BRAND,
    1.0,
    Color::from_rgb(0.20, 0.18, 0.35),
    TEXT_PRIMARY,
    ACCENT_BRAND
);
style_btn!(
    style_btn_ghost_danger,
    Color::TRANSPARENT,
    TEXT_MUTED,
    Color::TRANSPARENT,
    0.0,
    Color::from_rgb(0.20, 0.10, 0.10),
    ACCENT_DANGER,
    Color::TRANSPARENT
);
style_btn!(
    style_card_btn,
    BG_CARD,
    TEXT_PRIMARY,
    BORDER_COLOR,
    1.0,
    BG_CARD_HOVER,
    TEXT_PRIMARY,
    BORDER_COLOR
);
style_btn!(
    style_card_btn_active,
    BG_CARD_HOVER,
    TEXT_PRIMARY,
    ACCENT_BRAND,
    2.0,
    BG_CARD_HOVER,
    TEXT_PRIMARY,
    ACCENT_BRAND
);

pub fn main() -> iced::Result {
    iced::application(
        EasyVersionApp::new,
        EasyVersionApp::update,
        EasyVersionApp::view,
    )
    .window_size(iced::Size::new(1150.0, 850.0))
    .theme(Theme::Dark)
    .run()
}

#[derive(Debug, Clone)]
struct Stores {
    data: FileStore,
    history: FileStore,
}

#[derive(Debug, Clone)]
struct VersionSummary {
    added: usize,
    removed: usize,
    modified: usize,
}

#[derive(Debug, Clone)]
struct VersionChanges {
    added: Vec<String>,
    removed: Vec<String>,
    modified: Vec<String>,
}

#[derive(Debug, Clone)]
enum ExpandedDiffState {
    Loading,
    Loaded(VersionChanges),
}

struct ProjectState {
    path: PathBuf,
    history: easyversion::model::History,
    changes_summary: Vec<VersionSummary>,
    comment_input: String,
    search_query: String,
    status_message: Option<(String, bool)>,
    confirming_unlink: bool,
    expanded_version: Option<(usize, ExpandedDiffState)>,
    processing_action: Option<String>,
}

enum AppState {
    Loading,
    Error(String),
    Active {
        stores: Stores,
        project: Option<ProjectState>,
        recent_projects: Vec<PathBuf>,
    },
}

impl AppState {
    fn ctx(&mut self) -> Option<(&mut Stores, &mut Option<ProjectState>, &mut Vec<PathBuf>)> {
        if let AppState::Active {
            stores,
            project,
            recent_projects,
        } = self
        {
            Some((stores, project, recent_projects))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    StoresLoaded(Result<Stores, String>),
    PickFolder,
    FolderSelected(Option<PathBuf>),
    #[allow(dead_code)]
    RefreshProject,
    CommentChanged(String),
    SearchChanged(String),
    SaveVersion,
    ToggleVersionExpansion(usize),
    DiffLoaded(usize, VersionChanges),
    PickExtractFolder(usize),
    ExtractFolderSelected(usize, Option<PathBuf>),
    PromptUnlinkProject,
    CancelUnlinkProject,
    ConfirmUnlinkProject,
    SaveComplete(Result<(), String>),
    ExtractComplete(Result<(), String>),
    CleanComplete(Result<(), String>),
    ClearStatus,
}

struct EasyVersionApp {
    state: AppState,
}

impl EasyVersionApp {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                state: AppState::Loading,
            },
            Task::perform(load_stores(), Message::StoresLoaded),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::StoresLoaded(Ok(stores)) => {
                self.state = AppState::Active {
                    stores,
                    project: None,
                    recent_projects: Vec::new(),
                };
                Task::none()
            }
            Message::StoresLoaded(Err(e)) => {
                self.state = AppState::Error(e);
                Task::none()
            }
            Message::PickFolder => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Select your Art/Music/Video Project Folder")
                        .pick_folder()
                        .await
                        .map(|h| h.path().to_path_buf())
                },
                Message::FolderSelected,
            ),
            Message::FolderSelected(Some(path)) => {
                if let Some((stores, project, recent_projects)) = self.state.ctx() {
                    let hist = history(&stores.history, &path)
                        .unwrap_or_default()
                        .unwrap_or_default();
                    let summaries = calculate_summaries(&hist);
                    recent_projects.retain(|p| p != &path);
                    recent_projects.insert(0, path.clone());

                    *project = Some(ProjectState {
                        path,
                        history: hist,
                        changes_summary: summaries,
                        comment_input: String::new(),
                        search_query: String::new(),
                        status_message: None,
                        confirming_unlink: false,
                        expanded_version: None,
                        processing_action: None,
                    });
                }
                Task::none()
            }
            Message::FolderSelected(None) => Task::none(),
            Message::RefreshProject => {
                if let Some((_, Some(ws), _)) = self.state.ctx() {
                    let path = ws.path.clone();
                    return Task::perform(async move { Some(path) }, Message::FolderSelected);
                }
                Task::none()
            }
            Message::CommentChanged(text) => {
                if let Some((_, Some(ws), _)) = self.state.ctx() {
                    ws.comment_input = text;
                }
                Task::none()
            }
            Message::SearchChanged(text) => {
                if let Some((_, Some(ws), _)) = self.state.ctx() {
                    ws.search_query = text;
                }
                Task::none()
            }
            Message::ToggleVersionExpansion(index) => {
                if let Some((_, Some(ws), _)) = self.state.ctx() {
                    if let Some((current_index, _)) = &ws.expanded_version {
                        if *current_index == index {
                            ws.expanded_version = None;
                            return Task::none();
                        }
                    }

                    ws.expanded_version = Some((index, ExpandedDiffState::Loading));
                    let current_manifest = ws.history.snapshots[index].manifest.clone();
                    let prev_manifest = if index > 0 {
                        Some(ws.history.snapshots[index - 1].manifest.clone())
                    } else {
                        None
                    };
                    let base_path = ws.path.clone();

                    return Task::perform(
                        async move {
                            calculate_single_diff_async(base_path, prev_manifest, current_manifest)
                                .await
                        },
                        move |changes| Message::DiffLoaded(index, changes),
                    );
                }
                Task::none()
            }
            Message::DiffLoaded(index, changes) => {
                if let Some((_, Some(ws), _)) = self.state.ctx() {
                    if let Some((current_index, _)) = &ws.expanded_version {
                        if *current_index == index {
                            ws.expanded_version = Some((index, ExpandedDiffState::Loaded(changes)));
                        }
                    }
                }
                Task::none()
            }
            Message::SaveVersion => {
                if let Some((stores, Some(ws), _)) = self.state.ctx() {
                    if ws.processing_action.is_some() {
                        return Task::none();
                    }
                    ws.processing_action =
                        Some("Saving new version... This may take a moment.".to_string());
                    ws.status_message = None;

                    let comment =
                        (!ws.comment_input.trim().is_empty()).then(|| ws.comment_input.clone());
                    let data_store = stores.data.clone();
                    let history_store = stores.history.clone();
                    let path = ws.path.clone();

                    return Task::perform(
                        async move {
                            save(&data_store, &history_store, &path, comment)
                                .map_err(|e| e.to_string())
                        },
                        Message::SaveComplete,
                    );
                }
                Task::none()
            }
            Message::PickExtractFolder(index) => Task::perform(
                async move {
                    let target_path = rfd::AsyncFileDialog::new()
                        .set_title("Select an EMPTY folder to save this copy into")
                        .pick_folder()
                        .await
                        .map(|h| h.path().to_path_buf());
                    (index, target_path)
                },
                |(index, target_path)| Message::ExtractFolderSelected(index, target_path),
            ),
            Message::ExtractFolderSelected(index, Some(target_path)) => {
                if let Some((stores, Some(ws), _)) = self.state.ctx() {
                    if ws.processing_action.is_some() {
                        return Task::none();
                    }
                    ws.processing_action = Some("Extracting files...".to_string());
                    ws.status_message = None;

                    let data_store = stores.data.clone();
                    let history_store = stores.history.clone();
                    let source_path = ws.path.clone();

                    return Task::perform(
                        async move {
                            split(
                                &data_store,
                                &history_store,
                                &source_path,
                                &target_path,
                                Version::Specific(index),
                            )
                            .map_err(|e| e.to_string())
                        },
                        Message::ExtractComplete,
                    );
                }
                Task::none()
            }
            Message::ExtractFolderSelected(_, None) => Task::none(),
            Message::PromptUnlinkProject => {
                if let Some((_, Some(ws), _)) = self.state.ctx() {
                    ws.confirming_unlink = true;
                }
                Task::none()
            }
            Message::CancelUnlinkProject => {
                if let Some((_, Some(ws), _)) = self.state.ctx() {
                    ws.confirming_unlink = false;
                }
                Task::none()
            }
            Message::ConfirmUnlinkProject => {
                if let Some((stores, Some(ws), _)) = self.state.ctx() {
                    ws.processing_action =
                        Some("Unlinking project and removing history...".to_string());
                    ws.status_message = None;

                    let data_store = stores.data.clone();
                    let history_store = stores.history.clone();
                    let path = ws.path.clone();

                    return Task::perform(
                        async move {
                            clean(&data_store, &history_store, &path).map_err(|e| e.to_string())
                        },
                        Message::CleanComplete,
                    );
                }
                Task::none()
            }
            Message::SaveComplete(result) => {
                if let Some((stores, Some(ws), _)) = self.state.ctx() {
                    ws.processing_action = None;
                    ws.expanded_version = None;
                    match result {
                        Ok(()) => {
                            ws.comment_input.clear();
                            ws.status_message =
                                Some(("Success! New version safely stored.".to_string(), false));
                            if let Ok(Some(hist)) = history(&stores.history, &ws.path) {
                                ws.changes_summary = calculate_summaries(&hist);
                                ws.history = hist;
                            } else {
                                ws.history = easyversion::model::History::default();
                                ws.changes_summary = vec![];
                            }
                        }
                        Err(e) => {
                            ws.status_message =
                                Some((format!("Oops, something went wrong: {}", e), true));
                        }
                    }
                }
                Task::none()
            }
            Message::ExtractComplete(result) => {
                if let Some((_, Some(ws), _)) = self.state.ctx() {
                    ws.processing_action = None;
                    match result {
                        Ok(()) => {
                            ws.status_message = Some((
                                "Success! Copy extracted safely to the new folder.".to_string(),
                                false,
                            ));
                        }
                        Err(e) => {
                            ws.status_message =
                                Some((format!("Oops, extraction failed: {}", e), true));
                        }
                    }
                }
                Task::none()
            }
            Message::CleanComplete(result) => {
                if let Some((_, project, recent_projects)) = self.state.ctx() {
                    if let Some(ws) = project {
                        ws.processing_action = None;
                        ws.confirming_unlink = false;
                        match result {
                            Ok(()) => {
                                let removed_path = ws.path.clone();
                                recent_projects.retain(|p| p != &removed_path);
                                *project = None;
                            }
                            Err(e) => {
                                ws.status_message =
                                    Some((format!("Failed to unlink project: {}", e), true));
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::ClearStatus => {
                if let Some((_, Some(ws), _)) = self.state.ctx() {
                    ws.status_message = None;
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.state {
            AppState::Loading => container(text("Starting Up...").size(24).color(TEXT_MUTED))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(style_main_bg)
                .into(),
            AppState::Error(e) => container(
                text(format!("App Error: {}", e))
                    .size(20)
                    .color(ACCENT_DANGER),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(style_main_bg)
            .into(),
            AppState::Active {
                project,
                recent_projects,
                ..
            } => container(row![
                sidebar(project, recent_projects),
                rule::vertical(1),
                if let Some(ws) = project {
                    main_content(ws)
                } else {
                    welcome_screen()
                }
            ])
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
        }
    }
}

fn sidebar<'a>(
    project: &'a Option<ProjectState>,
    recent_projects: &'a [PathBuf],
) -> Element<'a, Message> {
    let is_processing = project
        .as_ref()
        .map_or(false, |p| p.processing_action.is_some());
    let mut sidebar_col: Column<'_, Message, Theme, iced::Renderer> = column![
        column![
            text("EasyVersion").size(28).color(TEXT_PRIMARY),
            text("Safety for Creatives").size(13).color(ACCENT_BRAND),
        ]
        .padding(Padding {
            top: 10.0,
            right: 0.0,
            bottom: 25.0,
            left: 0.0
        }),
        button(
            row![text("📂").size(18), text(" Open Project").size(16)]
                .spacing(8)
                .align_y(Alignment::Center)
        )
        .width(Length::Fill)
        .padding([14, 15])
        .style(style_btn_primary)
        .on_press_maybe(if is_processing {
            None
        } else {
            Some(Message::PickFolder)
        }),
        Space::new().width(0.0).height(35.0),
        text("RECENT PROJECTS").size(12).color(TEXT_MUTED),
        Space::new().width(0.0).height(10.0),
    ]
    .spacing(5)
    .padding([30, 25])
    .width(Length::Fixed(280.0))
    .height(Length::Fill);

    let mut recents_list: Column<'_, Message, Theme, iced::Renderer> = column![];
    for path in recent_projects {
        let folder_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let is_active = project.as_ref().map_or(false, |ws| ws.path == *path);
        let btn = button(column![
            text(folder_name).size(15).color(if is_active {
                ACCENT_BRAND
            } else {
                TEXT_PRIMARY
            }),
            text(path.display().to_string()).size(11).color(TEXT_MUTED)
        ])
        .width(Length::Fill)
        .padding(12)
        .style(if is_active {
            style_btn_ghost_active
        } else {
            style_btn_ghost
        })
        .on_press_maybe(if is_processing || is_active {
            None
        } else {
            Some(Message::FolderSelected(Some(path.clone())))
        });
        recents_list = recents_list.push(btn);
    }

    sidebar_col = sidebar_col.push(scrollable(recents_list.spacing(6)).height(Length::Fill));
    container(sidebar_col)
        .style(style_sidebar_bg)
        .height(Length::Fill)
        .into()
}

fn welcome_screen<'a>() -> Element<'a, Message> {
    let content: Column<'_, Message, Theme, iced::Renderer> = column![
        text("👋").size(60),
        Space::new().width(0.0).height(20.0),
        text("Welcome to EasyVersion").size(32),
        Space::new().width(0.0).height(10.0),
        text("The easiest way to backup your creative projects.")
            .size(18)
            .color(TEXT_MUTED),
        Space::new().width(0.0).height(40.0),
        container(column![
            row![
                text("1. ").color(ACCENT_BRAND).size(18),
                text("Click 'Open Project' to select your art/music folder.").size(16)
            ],
            Space::new().width(0.0).height(15.0),
            row![
                text("2. ").color(ACCENT_BRAND).size(18),
                text("Save new versions whenever you make major changes.").size(16)
            ],
            Space::new().width(0.0).height(15.0),
            row![
                text("3. ").color(ACCENT_BRAND).size(18),
                text("Extract copies of old versions safely without overwriting.").size(16)
            ],
        ])
        .padding(35)
        .style(style_card)
    ]
    .align_x(Alignment::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(style_main_bg)
        .into()
}

fn main_content<'a>(ws: &'a ProjectState) -> Element<'a, Message> {
    let is_processing = ws.processing_action.is_some();
    let mut main_col: Column<'_, Message, Theme, iced::Renderer> = column![];

    let mut header_row = row![
        column![
            text(
                ws.path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            )
            .size(32),
            Space::new().width(0.0).height(4.0),
            text(ws.path.display().to_string())
                .size(14)
                .color(TEXT_MUTED),
        ]
        .width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    if !ws.confirming_unlink {
        header_row = header_row.push(
            button(row![text("🔌").size(16), text(" Unlink").size(14)].spacing(8))
                .padding([10, 16])
                .style(style_btn_ghost_danger)
                .on_press_maybe(if is_processing {
                    None
                } else {
                    Some(Message::PromptUnlinkProject)
                }),
        );
    }
    main_col = main_col.push(header_row);

    if ws.confirming_unlink {
        main_col = main_col.push(Space::new().width(0.0).height(10.0));
        main_col = main_col.push(
            container(column![
                text("⚠️ Stop Tracking Project?").size(16).color(ACCENT_DANGER),
                Space::new().width(0.0).height(6.0),
                text("This removes the project from EasyVersion and deletes its backup history to save space. Your actual project files are completely safe.").size(14).color(TEXT_MUTED),
                Space::new().width(0.0).height(15.0),
                row![
                    button("Cancel").padding([8, 16]).style(style_btn_secondary).on_press_maybe(if is_processing { None } else { Some(Message::CancelUnlinkProject) }),
                    Space::new().width(Length::Fill).height(0.0),
                    button("Yes, Unlink & Clean").padding([8, 16]).style(style_btn_danger).on_press_maybe(if is_processing { None } else { Some(Message::ConfirmUnlinkProject) }),
                ]
            ]).padding(20).style(style_card_danger)
        );
    }

    if let Some(loading_msg) = &ws.processing_action {
        main_col = main_col.push(
            container(
                row![
                    text("⏳").size(22),
                    Space::new().width(12.0).height(0.0),
                    text(loading_msg).size(16).color(TEXT_PRIMARY)
                ]
                .align_y(Alignment::Center),
            )
            .padding([15, 20])
            .width(Length::Fill)
            .style(style_card_loading),
        );
    }

    if let Some((msg, is_error)) = &ws.status_message {
        let color = if *is_error {
            ACCENT_DANGER
        } else {
            COLOR_ADDED
        };
        let icon = if *is_error { "❌" } else { "✅" };
        main_col = main_col.push(
            container(
                row![
                    text(icon).size(16),
                    Space::new().width(10.0).height(0.0),
                    text(msg).size(16).color(color),
                    Space::new().width(Length::Fill).height(0.0),
                    button("Got it")
                        .style(style_btn_ghost)
                        .on_press(Message::ClearStatus)
                ]
                .align_y(Alignment::Center),
            )
            .padding([12, 18])
            .style(style_card),
        );
    }

    let save_box = container(column![
        text("Save a New Version").size(20),
        Space::new().width(0.0).height(6.0),
        text("About to try something crazy? Save a version now so you can always come back here.")
            .size(14)
            .color(TEXT_MUTED),
        Space::new().width(0.0).height(20.0),
        row![
            text_input(
                "Name this version (e.g. 'Finished the main vocals')",
                &ws.comment_input
            )
            .on_input(Message::CommentChanged)
            .on_submit(Message::SaveVersion)
            .padding(14)
            .size(16),
            button(text("💾 Save Version").size(16))
                .padding([14, 30])
                .style(style_btn_primary)
                .on_press_maybe(if is_processing {
                    None
                } else {
                    Some(Message::SaveVersion)
                }),
        ]
        .spacing(15)
    ])
    .padding(30)
    .style(style_card);
    main_col = main_col
        .push(save_box)
        .push(Space::new().width(0.0).height(10.0));

    main_col = main_col.push(
        row![
            text("Time Machine").size(24).width(Length::Fill),
            text_input("🔍 Find a version...", &ws.search_query)
                .on_input(Message::SearchChanged)
                .padding(12)
                .size(14)
                .width(Length::Fixed(280.0)),
        ]
        .align_y(Alignment::Center),
    );

    if ws.history.snapshots.is_empty() {
        main_col = main_col.push(
            container(
                column![
                    text("🎨").size(50),
                    Space::new().width(0.0).height(15.0),
                    text("Your canvas is blank!").size(22),
                    Space::new().width(0.0).height(5.0),
                    text("Work on your files in your regular apps (Photoshop, Ableton, etc.)")
                        .color(TEXT_MUTED)
                        .size(16),
                    text("When you want to save a backup, click 'Save Version' above.")
                        .color(TEXT_MUTED)
                        .size(16)
                ]
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .padding(80)
            .center_x(Length::Fill)
            .style(style_card_dashed),
        );
    } else {
        let mut history_list: Column<'_, Message, Theme, iced::Renderer> = column![];
        let query = ws.search_query.to_lowercase();

        for (i, snapshot) in ws.history.snapshots.iter().enumerate().rev() {
            let label = snapshot
                .comment
                .clone()
                .unwrap_or_else(|| format!("Version {}", i + 1));
            if !query.is_empty() && !label.to_lowercase().contains(&query) {
                continue;
            }

            let summary = ws.changes_summary.get(i).unwrap();
            let is_expanded = ws
                .expanded_version
                .as_ref()
                .map_or(false, |(idx, _)| *idx == i);
            let total_changes = summary.added + summary.removed + summary.modified;

            let mut card_content: Column<'_, Message, Theme, iced::Renderer> = column![
                row![
                    column![
                        text(label).size(20),
                        Space::new().width(0.0).height(8.0),
                        text(format!(
                            "{} files in project  •  {} files changed",
                            snapshot.manifest.files.len(),
                            total_changes
                        ))
                        .size(14)
                        .color(TEXT_MUTED),
                    ]
                    .width(Length::Fill),
                    button(row![text("📦").size(14), text(" Extract Copy...").size(14)].spacing(8))
                        .padding([12, 20])
                        .style(style_btn_secondary)
                        .on_press_maybe(if is_processing {
                            None
                        } else {
                            Some(Message::PickExtractFolder(i))
                        }),
                ]
                .align_y(Alignment::Center)
            ];

            if is_expanded {
                card_content = card_content
                    .push(Space::new().width(0.0).height(20.0))
                    .push(rule::horizontal(1))
                    .push(Space::new().width(0.0).height(20.0));

                if let Some((_, diff_state)) = &ws.expanded_version {
                    match diff_state {
                        ExpandedDiffState::Loading => {
                            card_content = card_content.push(
                                container(row![text("⏳ Loading changes...").color(TEXT_MUTED)])
                                    .padding([15, 20])
                                    .style(style_diff_bg)
                                    .width(Length::Fill),
                            );
                        }
                        ExpandedDiffState::Loaded(changes) => {
                            let mut diff_col: Column<'_, Message, Theme, iced::Renderer> =
                                column![];
                            if total_changes == 0 {
                                diff_col = diff_col.push(
                                    text("No files were changed in this version.")
                                        .size(14)
                                        .color(TEXT_MUTED),
                                );
                            } else {
                                let mut rendered = 0;
                                let max_render = 150;
                                for file in &changes.added {
                                    if rendered >= max_render {
                                        break;
                                    }
                                    diff_col = diff_col.push(row![
                                        text("✨ Added: ").color(COLOR_ADDED),
                                        text(file).size(14).color(TEXT_PRIMARY)
                                    ]);
                                    rendered += 1;
                                }
                                for file in &changes.modified {
                                    if rendered >= max_render {
                                        break;
                                    }
                                    diff_col = diff_col.push(row![
                                        text("📝 Changed: ").color(COLOR_MODIFIED),
                                        text(file).size(14).color(TEXT_PRIMARY)
                                    ]);
                                    rendered += 1;
                                }
                                for file in &changes.removed {
                                    if rendered >= max_render {
                                        break;
                                    }
                                    diff_col = diff_col.push(row![
                                        text("🗑️ Removed: ").color(COLOR_REMOVED),
                                        text(file).size(14).color(TEXT_MUTED)
                                    ]);
                                    rendered += 1;
                                }
                                if total_changes > max_render {
                                    diff_col = diff_col.push(
                                        text(format!(
                                            "... and {} more files",
                                            total_changes - max_render
                                        ))
                                        .size(14)
                                        .color(TEXT_MUTED),
                                    );
                                }
                            }
                            card_content = card_content.push(
                                container(diff_col.spacing(8))
                                    .padding([15, 20])
                                    .style(style_diff_bg)
                                    .width(Length::Fill),
                            );
                        }
                    }
                }
            }

            let card_btn = button(card_content)
                .width(Length::Fill)
                .padding(25)
                .style(if is_expanded {
                    style_card_btn_active
                } else {
                    style_card_btn
                })
                .on_press_maybe(if is_processing {
                    None
                } else {
                    Some(Message::ToggleVersionExpansion(i))
                });

            history_list = history_list.push(card_btn);
        }
        main_col = main_col.push(
            scrollable(history_list.spacing(15))
                .height(Length::Fill)
                .width(Length::Fill),
        );
    }

    container(main_col.padding([40, 50]).spacing(25))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style_main_bg)
        .into()
}

fn calculate_summaries(history: &easyversion::model::History) -> Vec<VersionSummary> {
    let mut summaries = Vec::with_capacity(history.snapshots.len());
    for i in 0..history.snapshots.len() {
        let snapshot = &history.snapshots[i];
        let mut added = 0;
        let mut removed = 0;
        let mut modified = 0;
        let prev_manifest = if i > 0 {
            Some(&history.snapshots[i - 1].manifest)
        } else {
            None
        };

        if let Some(prev) = prev_manifest {
            for (path, id) in &snapshot.manifest.files {
                match prev.files.get(path) {
                    Some(prev_id) if prev_id != id => modified += 1,
                    None => added += 1,
                    _ => {}
                }
            }
            for path in prev.files.keys() {
                if !snapshot.manifest.files.contains_key(path) {
                    removed += 1;
                }
            }
        } else {
            added = snapshot.manifest.files.len();
        }
        summaries.push(VersionSummary {
            added,
            removed,
            modified,
        });
    }
    summaries
}

async fn calculate_single_diff_async(
    base_path: PathBuf,
    prev_manifest: Option<easyversion::model::Manifest>,
    current_manifest: easyversion::model::Manifest,
) -> VersionChanges {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();

    let format_path = |p: &PathBuf| -> String {
        p.strip_prefix(&base_path)
            .unwrap_or(p)
            .to_string_lossy()
            .to_string()
    };

    if let Some(prev) = prev_manifest {
        for (path, id) in &current_manifest.files {
            match prev.files.get(path) {
                Some(prev_id) if prev_id != id => modified.push(format_path(path)),
                None => added.push(format_path(path)),
                _ => {}
            }
        }
        for path in prev.files.keys() {
            if !current_manifest.files.contains_key(path) {
                removed.push(format_path(path));
            }
        }
    } else {
        for path in current_manifest.files.keys() {
            added.push(format_path(path));
        }
    }

    added.sort();
    removed.sort();
    modified.sort();
    VersionChanges {
        added,
        removed,
        modified,
    }
}

async fn load_stores() -> Result<Stores, String> {
    let project_directories = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or_else(|| "No home directory could be found".to_string())?;
    let data_directory = project_directories.data_local_dir();
    let data = FileStore::new(&data_directory.join("data"))
        .map_err(|e| format!("Failed to load data store: {}", e))?;
    let history = FileStore::new(&data_directory.join("history"))
        .map_err(|e| format!("Failed to load history store: {}", e))?;
    Ok(Stores { data, history })
}
