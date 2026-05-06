// SPDX-FileCopyrightText: 2022 The ReGreet Authors
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Templates for various GUI components
#![allow(dead_code)] // Silence dead code warnings for UI code that isn't dead
#![allow(deprecated)]

use gtk::prelude::*;
use relm4::{RelmWidgetExt, WidgetTemplate, gtk};

/// Button that ends the greeter (eg. Reboot)
#[relm4::widget_template(pub)]
impl WidgetTemplate for EndButton {
    view! {
        gtk::Button {
            set_focusable: true,
            add_css_class: "destructive-action",
        }
    }
}

/// Label for an entry/combo box
#[relm4::widget_template(pub)]
impl WidgetTemplate for EntryLabel {
    view! {
        gtk::Label {
            set_width_request: 100,
            set_xalign: 1.0,
        }
    }
}

/// Main UI of the greeter
#[relm4::widget_template(pub)]
impl WidgetTemplate for Ui {
    view! {
        gtk::Overlay {
            /// Background media
            #[name = "background_box"]
            gtk::Box {
                set_hexpand: true,
                set_vexpand: true,
                set_halign: gtk::Align::Fill,
                set_valign: gtk::Align::Fill,

                /// Background image
                #[name = "background_image"]
                gtk::Picture {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Fill,
                },

                /// Background video
                #[name = "background_video"]
                gtk::Picture {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Fill,
                },
            },

            /// Main login box
            add_overlay = &gtk::Frame {
                set_halign: gtk::Align::Start,
                set_valign: gtk::Align::Center,
                set_margin_start: 72,
                add_css_class: "login-shell",

                gtk::Grid {
                    add_css_class: "login-grid",
                    set_column_spacing: 0,
                    set_margin_bottom: 0,
                    set_margin_end: 0,
                    set_margin_start: 0,
                    set_margin_top: 0,
                    set_row_spacing: 14,
                    set_width_request: 280,

                    /// Selected user avatar
                    #[name = "profile_button"]
                    attach[0, 0, 1, 1] = &gtk::Button {
                        add_css_class: "profile-button",
                        set_halign: gtk::Align::Center,
                        set_tooltip_text: Some("Switch user"),

                        gtk::Box {
                            add_css_class: "profile-avatar",
                            set_halign: gtk::Align::Center,
                            set_size_request: (88, 88),
                            set_overflow: gtk::Overflow::Hidden,

                            #[name = "profile_avatar"]
                            gtk::Picture {
                                add_css_class: "profile-avatar-image",
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Center,
                                set_size_request: (80, 80),
                                set_content_fit: gtk::ContentFit::Cover,
                                set_can_shrink: true,
                                set_overflow: gtk::Overflow::Hidden,
                            },
                        },
                    },

                    /// Widget to display messages to the user
                    #[name = "message_label"]
                    attach[0, 1, 1, 1] = &gtk::Label {
                        add_css_class: "message-label",
                        set_margin_bottom: 4,

                        // Format all messages in boldface.
                        #[wrap(Some)]
                        set_attributes = &gtk::pango::AttrList {
                            insert: {
                                let mut font_desc = gtk::pango::FontDescription::new();
                                font_desc.set_weight(gtk::pango::Weight::Bold);
                                gtk::pango::AttrFontDesc::new(&font_desc)
                            },
                        },
                    },

                    #[template]
                    attach[0, 2, 1, 1] = &EntryLabel {
                        add_css_class: "field-label",
                        set_label: "Session:",
                        set_height_request: 44,
                        set_visible: false,
                    },

                    /// Label for the sessions widget
                    #[name = "session_label"]
                    #[template]
                    attach[0, 3, 1, 1] = &EntryLabel {
                        add_css_class: "field-label",
                        set_label: "User:",
                        set_height_request: 44,
                        set_visible: false,
                    },

                    /// Widget containing the usernames
                    #[name = "usernames_box"]
                    attach[0, 3, 1, 1] = &gtk::ComboBoxText {
                        add_css_class: "identity-field",
                        set_hexpand: true,
                        set_visible: false,
                    },

                    /// Widget where the user enters the username
                    #[name = "username_entry"]
                    attach[0, 3, 1, 1] = &gtk::Entry {
                        add_css_class: "identity-field",
                        set_hexpand: true,
                        set_visible: false,
                    },

                    /// Widget where the user enters the session
                    #[name = "session_entry"]
                    attach[0, 2, 1, 1] = &gtk::Entry {
                        add_css_class: "identity-field",
                        set_visible: false,
                    },

                    /// Label for the password widget
                    #[name = "input_label"]
                    #[template]
                    attach[0, 4, 1, 1] = &EntryLabel {
                        add_css_class: "field-label",
                        set_label: "Password:",
                        set_height_request: 44,
                        set_visible: false,
                    },

                    /// Widget where the user enters a secret
                    #[name = "secret_entry"]
                    attach[0, 5, 1, 1] = &gtk::PasswordEntry {
                        add_css_class: "password-field",
                        set_show_peek_icon: true,
                    },

                    /// Widget where the user enters something visible
                    #[name = "visible_entry"]
                    attach[0, 5, 1, 1] = &gtk::Entry {
                        add_css_class: "password-field",
                    },

                    /// Button to toggle manual user entry
                    #[name = "user_toggle"]
                    attach[0, 6, 1, 1] = &gtk::ToggleButton {
                        add_css_class: "field-toggle",
                        set_icon_name: "document-edit-symbolic",
                        set_tooltip_text: Some("Manually enter username"),
                        set_visible: false,
                    },

                    /// Button to toggle manual session entry
                    #[name = "sess_toggle"]
                    attach[0, 6, 1, 1] = &gtk::ToggleButton {
                        add_css_class: "field-toggle",
                        set_icon_name: "document-edit-symbolic",
                        set_tooltip_text: Some("Manually enter session command"),
                        set_visible: false,
                    },

                    /// Collection of action buttons (eg. Login)
                    attach[0, 6, 1, 1] = &gtk::Box {
                        set_halign: gtk::Align::Center,
                        set_margin_top: 2,

                        /// Button to cancel password entry
                        #[name = "cancel_button"]
                        gtk::Button {
                            set_focusable: true,
                            set_label: "Cancel",
                        },

                        /// Button to enter the password and login
                        #[name = "login_button"]
                        gtk::Button {
                            set_focusable: true,
                            set_label: "Login",
                            set_receives_default: true,
                            add_css_class: "suggested-action",
                            set_hexpand: true,
                        },
                    },
                },
            },

            /// Session selector
            #[name = "sessions_box"]
            add_overlay = &gtk::ComboBoxText {
                add_css_class: "session-field",
                set_halign: gtk::Align::Start,
                set_valign: gtk::Align::End,
                set_margin_start: 28,
                set_margin_bottom: 28,
            },

            /// Clock widget
            #[name = "clock_frame"]
            add_overlay = &gtk::Frame {
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Start,

                add_css_class: "background",

                // Make it fit cleanly onto the top edge of the screen.
                inline_css: "
                    border-top-right-radius: 0px;
                    border-top-left-radius: 0px;
                    border-top-width: 0px;
                ",
            },

            /// Collection of widgets appearing at the bottom
            add_overlay = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::End,
                set_margin_bottom: 15,
                set_spacing: 15,

                gtk::Frame {
                    /// Notification bar for error messages
                    #[name = "error_info"]
                    gtk::InfoBar {
                        // During init, the info bar closing animation is shown. To hide that, make
                        // it invisible. Later, the code will permanently make it visible, so that
                        // `InfoBar::set_revealed` will work properly with animations.
                        set_visible: false,
                        set_message_type: gtk::MessageType::Error,

                        /// The actual error message
                        #[name = "error_label"]
                        gtk::Label {
                            set_halign: gtk::Align::Center,
                            set_margin_top: 10,
                            set_margin_bottom: 10,
                            set_margin_start: 10,
                            set_margin_end: 10,
                        },
                    }
                },

                /// Collection of buttons that close the greeter (eg. Reboot)
                gtk::Box {
                    set_halign: gtk::Align::Center,
                    set_homogeneous: true,
                    set_spacing: 15,

                    /// Button to reboot
                    #[name = "reboot_button"]
                    #[template]
                    EndButton { set_label: "Reboot" },

                    /// Button to power-off
                    #[name = "poweroff_button"]
                    #[template]
                    EndButton { set_label: "Power Off" },
                },
            },
        }
    }
}
