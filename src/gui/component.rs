// SPDX-FileCopyrightText: 2022 The ReGreet Authors
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Setup for using the greeter as a Relm4 component

#![allow(deprecated)]

use std::{env, path::PathBuf};

use relm4::{
    AsyncComponentSender,
    component::{AsyncComponent, AsyncComponentParts},
    gtk::prelude::*,
    prelude::*,
};
use tracing::{debug, info, warn};

use crate::config::BgFit;

use super::messages::{CommandMsg, InputMsg, UserSessInfo};
use super::model::{Greeter, InputMode, Updates};
use super::templates::Ui;

const AVATAR_SIZE: i32 = 80;
const DEFAULT_CSS: &str = include_str!("../../regreet.css");

/// Load GTK settings from the greeter config.
fn setup_settings(model: &Greeter, root: &gtk::ApplicationWindow) {
    let settings = root.settings();
    let config = if let Some(config) = model.config.get_gtk_settings() {
        config
    } else {
        return;
    };

    debug!(
        "Setting dark theme: {}",
        config.application_prefer_dark_theme
    );
    settings.set_gtk_application_prefer_dark_theme(config.application_prefer_dark_theme);

    if let Some(cursor_theme) = &config.cursor_theme_name {
        debug!("Setting cursor theme: {cursor_theme}");
        settings.set_gtk_cursor_theme_name(config.cursor_theme_name.as_deref());
    };

    debug!("Setting cursor blink: {}", config.cursor_blink);
    settings.set_gtk_cursor_blink(config.cursor_blink);

    if let Some(font) = &config.font_name {
        debug!("Setting font: {font}");
        settings.set_gtk_font_name(config.font_name.as_deref());
    };

    if let Some(icon_theme) = &config.icon_theme_name {
        debug!("Setting icon theme: {icon_theme}");
        settings.set_gtk_icon_theme_name(config.icon_theme_name.as_deref());
    };

    if let Some(theme) = &config.theme_name {
        debug!("Setting theme: {theme}");
        settings.set_gtk_theme_name(config.theme_name.as_deref());
    };
}

/// Setup the background widget based on the configured asset.
fn setup_background(model: &Greeter, widgets: &GreeterWidgets) {
    let background_path = model.config.get_background();
    let is_video = model.config.is_video_background();
    let has_background = background_path.is_some();

    debug!(
        "background path={:?}, is_video={}, has_background={}",
        background_path, is_video, has_background
    );

    widgets.ui.background_box.set_visible(has_background);
    widgets.ui.background_image.set_visible(has_background && !is_video);
    widgets.ui.background_video.set_visible(has_background && is_video);

    if is_video {
        if let Some(path) = background_path {
            debug!("setting video filename: {}", path);
            
            // Get all monitors and create video for each
            let mut monitor_count = 0;
            let display = widgets.ui.display();
            for _monitor_item in display
                .monitors()
                .into_iter()
                .filter_map(|item| {
                    item.ok()
                        .and_then(|object| object.downcast::<gtk::gdk::Monitor>().ok())
                })
                .filter(gtk::gdk::Monitor::is_valid)
            {
                // Create a Picture widget for each monitor
                let picture = gtk::Picture::new();
                picture.set_hexpand(true);
                picture.set_vexpand(true);
                picture.set_halign(gtk::Align::Fill);
                picture.set_valign(gtk::Align::Fill);
                picture.set_content_fit(gtk::ContentFit::Cover);
                
                // Set up the video
                let media = gtk::MediaFile::for_filename(&path);
                media.set_loop(true);
                media.set_muted(true);
                media.play();
                picture.set_paintable(Some(&media));
                
                // Add to background box
                widgets.ui.background_box.append(&picture);
                monitor_count += 1;
            }
            
            // If any monitors detected, hide the template background_video and use per-monitor videos
            if monitor_count > 0 {
                debug!("Monitors detected ({monitor_count}), using per-monitor videos");
                widgets.ui.background_video.set_visible(false);
            } else {
                debug!("No monitors detected, using template background_video");
                let media = gtk::MediaFile::for_filename(&path);
                media.set_loop(true);
                media.set_muted(true);
                media.play();
                widgets.ui.background_video.set_paintable(Some(&media));
            }
        }
    } else {
        if let Some(path) = background_path {
            debug!("setting image filename: {}", path);
        }
        widgets.ui.background_image.set_filename(background_path);
    }
}

fn profile_avatar_paths(username: Option<&str>, demo: bool) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(username) = username {
        paths.push(PathBuf::from(format!("/home/{username}/.face")));
        paths.push(PathBuf::from(format!("/var/lib/regreet/faces/{username}")));
        paths.push(PathBuf::from(format!(
            "/var/lib/AccountsService/icons/{username}"
        )));
    }

    if demo {
        if let Ok(username) = env::var("USER") {
            paths.push(PathBuf::from(format!("/home/{username}/.face")));
        }
    }

    paths
}

fn set_profile_avatar_picture(profile_avatar: &gtk::Picture, username: Option<&str>, demo: bool) {
    let face_path = profile_avatar_paths(username, demo)
        .into_iter()
        .find(|path| path.exists());

    let Some(face_path) = face_path else {
        debug!("profile avatar not found for user: {:?}", username);
        profile_avatar.set_paintable(gtk::gdk::Paintable::NONE);
        return;
    };

    match gtk::gdk_pixbuf::Pixbuf::from_file_at_scale(
        &face_path,
        AVATAR_SIZE,
        AVATAR_SIZE,
        true,
    ) {
        Ok(pixbuf) => {
            let texture = gtk::gdk::Texture::for_pixbuf(&pixbuf);
            debug!("setting profile avatar: {}", face_path.display());
            profile_avatar.set_paintable(Some(&texture));
        }
        Err(err) => {
            warn!("couldn't load profile avatar '{}': {}", face_path.display(), err);
            profile_avatar.set_paintable(gtk::gdk::Paintable::NONE);
        }
    }
}

fn set_profile_avatar(model: &Greeter, widgets: &GreeterWidgets, username: Option<&str>) {
    set_profile_avatar_picture(&widgets.ui.profile_avatar, username, model.demo);
}

fn center_combo_box_text(combo_box: &gtk::ComboBoxText) {
    for cell in combo_box.cells() {
        cell.set_xalign(0.5);
    }
}

/// Populate the user and session combo boxes with entries.
fn setup_users_sessions(model: &Greeter, widgets: &GreeterWidgets) {
    // The user that is shown during initial login
    let mut initial_username = None;

    // Populate the usernames combo box.
    for (user, username) in model.sys_util.get_users().iter() {
        debug!("Found user: {user}");
        if initial_username.is_none() {
            initial_username = Some(username.clone());
        }
        widgets.ui.usernames_box.append(Some(username), user);
    }

    // Populate the sessions combo box.
    for session in model.sys_util.get_sessions().keys() {
        debug!("Found session: {session}");
        widgets.ui.sessions_box.append(Some(session), session);
    }

    center_combo_box_text(&widgets.ui.sessions_box);

    // If the last user is known, show their login initially.
    if let Some(last_user) = model.cache.get_last_user() {
        initial_username = Some(last_user.to_string());
    } else if let Some(user) = &initial_username {
        info!("Using first found user '{user}' as initial user");
    }

    // Set the user shown initially at login.
    if !widgets
        .ui
        .usernames_box
        .set_active_id(initial_username.as_deref())
    {
        if let Some(user) = initial_username {
            warn!("Couldn't find user '{user}' to set as the initial user");
        }
    }

    set_profile_avatar(model, widgets, widgets.ui.usernames_box.active_id().as_deref());
}

/// The info required to initialize the greeter
pub struct GreeterInit {
    pub config_path: PathBuf,
    pub css_path: PathBuf,
    pub demo: bool,
}

#[relm4::component(pub, async)]
impl AsyncComponent for Greeter {
    type Input = InputMsg;
    type Output = ();
    type Init = GreeterInit;
    type CommandOutput = CommandMsg;

    view! {
        // The `view!` macro needs a proper widget, not a template, as the root.
        #[name = "window"]
        gtk::ApplicationWindow {
            set_visible: true,
            set_decorated: false,
            set_fullscreened: true,

            // Name the UI widget, otherwise the inner children cannot be accessed by name.
            #[name = "ui"]
            #[template]
            Ui {
                #[template_child]
                clock_frame {
                    model.clock.widget(),
                },

                // #[template_child]
                // message_label {
                //     #[track(model.updates.changed(Updates::message()))]
                //     set_label: &model.updates.message,
                // },
                #[template_child]
                session_label {
                    #[track(model.updates.changed(Updates::input_mode()))]
                    set_visible: false,
                },
                #[template_child]
                usernames_box {
                    #[track(
                        model.updates.changed(Updates::manual_user_mode())
                        || model.updates.changed(Updates::input_mode())
                    )]
                    set_sensitive: !model.updates.manual_user_mode && !model.updates.is_input(),
                    #[track(model.updates.changed(Updates::manual_user_mode()))]
                    set_visible: false,
                    connect_changed[
                        sender,
                        username_entry = ui.username_entry.clone(),
                        sessions_box = ui.sessions_box.clone(),
                        session_entry = ui.session_entry.clone(),
                        profile_avatar = ui.profile_avatar.clone(),
                        demo = model.demo,
                    ] => move |this| sender.input(
                        {
                            let username = this.active_id();
                            set_profile_avatar_picture(&profile_avatar, username.as_deref(), demo);
                            Self::Input::UserChanged(
                                UserSessInfo::extract(this, &username_entry, &sessions_box, &session_entry)
                            )
                        }
                    ),
                },
                #[template_child]
                profile_button {
                    connect_clicked[
                        usernames_box = ui.usernames_box.clone(),
                        users_count = model.sys_util.get_users().len() as u32,
                    ] => move |_| {
                        if users_count == 0 {
                            return;
                        }
                        let current = usernames_box.active().unwrap_or(0);
                        usernames_box.set_active(Some((current + 1) % users_count));
                    },
                },
                #[template_child]
                username_entry {
                    #[track(
                        model.updates.changed(Updates::manual_user_mode())
                        || model.updates.changed(Updates::input_mode())
                    )]
                    set_sensitive: model.updates.manual_user_mode && !model.updates.is_input(),
                    #[track(model.updates.changed(Updates::manual_user_mode()))]
                    set_visible: false,
                },
                #[template_child]
                sessions_box {
                    #[track(
                        model.updates.changed(Updates::manual_sess_mode())
                        || model.updates.changed(Updates::input_mode())
                    )]
                    set_visible: !model.updates.manual_sess_mode,
                    #[track(model.updates.changed(Updates::active_session_id()))]
                    set_active_id: model.updates.active_session_id.as_deref(),
                },
                #[template_child]
                session_entry {
                    #[track(
                        model.updates.changed(Updates::manual_sess_mode())
                        || model.updates.changed(Updates::input_mode())
                    )]
                    set_visible: false,
                },
                #[template_child]
                input_label {
                    #[track(model.updates.changed(Updates::input_mode()))]
                    set_visible: false,
                    #[track(model.updates.changed(Updates::input_prompt()))]
                    set_label: if model.updates.input_prompt.is_empty() {
                        "Password:"
                    } else {
                        &model.updates.input_prompt
                    },
                },
                #[template_child]
                secret_entry {
                    #[track(model.updates.changed(Updates::input_mode()))]
                    set_visible: model.updates.input_mode != InputMode::Visible,
                    #[track(
                        model.updates.changed(Updates::input_mode())
                        && model.updates.input_mode != InputMode::Visible
                    )]
                    grab_focus: (),
                    #[track(model.updates.changed(Updates::input()))]
                    set_text: &model.updates.input,
                    connect_activate[
                        sender,
                        usernames_box = ui.usernames_box.clone(),
                        username_entry = ui.username_entry.clone(),
                        sessions_box = ui.sessions_box.clone(),
                        session_entry = ui.session_entry.clone(),
                    ] => move |this| {
                        sender.input(Self::Input::Login {
                            input: this.text().to_string(),
                            info: UserSessInfo::extract(
                                &usernames_box, &username_entry, &sessions_box, &session_entry
                            ),
                        })
                    }
                },
                #[template_child]
                visible_entry {
                    #[track(model.updates.changed(Updates::input_mode()))]
                    set_visible: model.updates.input_mode == InputMode::Visible,
                    #[track(
                        model.updates.changed(Updates::input_mode())
                        && model.updates.input_mode == InputMode::Visible
                    )]
                    grab_focus: (),
                    #[track(model.updates.changed(Updates::input()))]
                    set_text: &model.updates.input,
                    connect_activate[
                        sender,
                        usernames_box = ui.usernames_box.clone(),
                        username_entry = ui.username_entry.clone(),
                        sessions_box = ui.sessions_box.clone(),
                        session_entry = ui.session_entry.clone(),
                    ] => move |this| {
                        sender.input(Self::Input::Login {
                            input: this.text().to_string(),
                            info: UserSessInfo::extract(
                                &usernames_box, &username_entry, &sessions_box, &session_entry
                            ),
                        })
                    }
                },
                #[template_child]
                user_toggle {
                    #[track(model.updates.changed(Updates::input_mode()))]
                    set_sensitive: model.updates.input_mode == InputMode::None,
                    #[track(model.updates.changed(Updates::input_mode()))]
                    set_visible: false,
                    connect_clicked => Self::Input::ToggleManualUser,
                },
                #[template_child]
                sess_toggle {
                    #[track(model.updates.changed(Updates::input_mode()))]
                    set_visible: false,
                    connect_clicked => Self::Input::ToggleManualSess,
                },
                // #[template_child]
                // cancel_button {
                //     #[track(model.updates.changed(Updates::input_mode()))]
                //     set_visible: model.updates.is_input(),
                //     connect_clicked => Self::Input::Cancel,
                // },
                // #[template_child]
                // login_button {
                //     #[track(
                //         model.updates.changed(Updates::input_mode())
                //         && !model.updates.is_input()
                //     )]
                //     grab_focus: (),
                //     connect_clicked[
                //         sender,
                //         secret_entry = ui.secret_entry.clone(),
                //         visible_entry = ui.visible_entry.clone(),
                //         usernames_box = ui.usernames_box.clone(),
                //         username_entry = ui.username_entry.clone(),
                //         sessions_box = ui.sessions_box.clone(),
                //         session_entry = ui.session_entry.clone(),
                //     ] => move |_| {
                //         sender.input(Self::Input::Login {
                //             input: if secret_entry.is_visible() {
                //                 // This should correspond to `InputMode::Secret`.
                //                 secret_entry.text().to_string()
                //             } else if EntryExt::is_visible(&visible_entry) {
                //                 // This should correspond to `InputMode::Visible`.
                //                 visible_entry.text().to_string()
                //             } else {
                //                 // This should correspond to `InputMode::None`.
                //                 String::new()
                //             },
                //             info: UserSessInfo::extract(
                //                 &usernames_box, &username_entry, &sessions_box, &session_entry
                //             ),
                //         })
                //     }
                // },
                #[template_child]
                error_info {
                    #[track(model.updates.changed(Updates::error()))]
                    set_revealed: model.updates.error.is_some(),
                },
                #[template_child]
                error_label {
                    #[track(model.updates.changed(Updates::error()))]
                    set_label: model.updates.error.as_ref().unwrap_or(&"".to_string()),
                },
                #[template_child]
                reboot_button { connect_clicked => Self::Input::Reboot },
                #[template_child]
                poweroff_button { connect_clicked => Self::Input::PowerOff },
            }
        }
    }

    /// Initialize the greeter.
    async fn init(
        input: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut model = Self::new(&input.config_path, input.demo).await;
        let widgets = view_output!();

        // Make the info bar permanently visible, since it was made invisible during init. The
        // actual visuals are controlled by `InfoBar::set_revealed`.
        widgets.ui.error_info.set_visible(true);

        let content_fit = match model.config.get_background_fit() {
            BgFit::Fill => gtk4::ContentFit::Fill,
            BgFit::Contain => gtk4::ContentFit::Contain,
            BgFit::Cover => gtk4::ContentFit::Cover,
            BgFit::ScaleDown => gtk4::ContentFit::ScaleDown,
        };
        widgets.ui.background_image.set_content_fit(content_fit);
        // Note: Video widget does not have set_content_fit method

        // Cancel any previous session, just in case someone started one.
        if let Err(err) = model.greetd_client.lock().await.cancel_session().await {
            warn!("Couldn't cancel greetd session: {err}");
        };

        model.choose_monitor(widgets.ui.display().name().as_str(), &sender);
        // Fullscreen on all monitors
        root.fullscreen();

        // For some reason, the GTK settings are reset when changing monitors, so apply them after
        // full-screening.
        setup_settings(&model, &root);
        setup_users_sessions(&model, &widgets);

        let default_provider = gtk::CssProvider::new();
        default_provider.load_from_string(DEFAULT_CSS);
        gtk::style_context_add_provider_for_display(
            &widgets.ui.display(),
            &default_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        if input.css_path.exists() {
            debug!("Loading custom CSS from file: {}", input.css_path.display());
            let custom_provider = gtk::CssProvider::new();
            custom_provider.load_from_path(input.css_path);
            gtk::style_context_add_provider_for_display(
                &widgets.ui.display(),
                &custom_provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
            );
        };

        setup_background(&model, &widgets);

        // Set the default behaviour of pressing the Return key to act like the login button.
        // root.set_default_widget(Some(&widgets.ui.login_button));

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        debug!("Got input message: {msg:?}");

        // Reset the tracker for update changes.
        self.updates.reset();

        match msg {
            Self::Input::Login { input, info } => {
                self.sess_info = Some(info);
                self.login_click_handler(&sender, input).await
            }
            Self::Input::UserChanged(info) => {
                self.sess_info = Some(info);
                self.user_change_handler();
            }
            Self::Input::ToggleManualUser => self
                .updates
                .set_manual_user_mode(!self.updates.manual_user_mode),
            Self::Input::ToggleManualSess => self
                .updates
                .set_manual_sess_mode(!self.updates.manual_sess_mode),
            Self::Input::Reboot => self.reboot_click_handler(&sender),
            Self::Input::PowerOff => self.poweroff_click_handler(&sender),
        }
    }

    /// Perform the requested changes when a background task sends a message.
    async fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        debug!("Got command message: {msg:?}");

        // Reset the tracker for update changes.
        self.updates.reset();

        match msg {
            Self::CommandOutput::ClearErr => self.updates.set_error(None),
            Self::CommandOutput::HandleGreetdResponse(response) => {
                self.handle_greetd_response(&sender, response).await
            }
            Self::CommandOutput::MonitorRemoved(display_name) => {
                self.choose_monitor(display_name.as_str(), &sender)
            }
        };
    }
}
