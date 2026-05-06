// SPDX-FileCopyrightText: 2022 The ReGreet Authors
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Support for multiple windows, one per monitor

use std::cell::RefCell;
use std::rc::Rc;
use gtk::prelude::*;
use gtk::gdk::{Display, Monitor};

use crate::config::Config;

/// Information about a monitor and its window
#[derive(Clone)]
pub struct MonitorWindow {
    pub name: String,
    pub monitor: Option<Monitor>,
    pub window: Option<gtk::ApplicationWindow>,
}

/// Get all connected monitors
pub fn get_monitors(display_name: &str) -> Vec<MonitorWindow> {
    let display = match gtk::gdk::Display::open(Some(display_name)) {
        Some(display) => display,
        None => {
            tracing::error!("Couldn't get display with name: {display_name}");
            return Vec::new();
        }
    };

    let mut monitors = Vec::new();
    for monitor_item in display
        .monitors()
        .into_iter()
        .filter_map(|item| {
            item.ok()
                .and_then(|object| object.downcast::<Monitor>().ok())
        })
        .filter(Monitor::is_valid)
    {
        let name = format!("{:?}", monitor_item);
        monitors.push(MonitorWindow {
            name,
            monitor: Some(monitor_item),
            window: None,
        });
    }

    monitors
}

/// Create secondary windows for each monitor (except the first)
pub fn create_secondary_windows(
    app: &gtk::Application,
    config: &Config,
    display_name: &str,
) -> Vec<Rc<RefCell<MonitorWindow>>> {
    let mut all_monitors = get_monitors(display_name);

    // Skip first monitor (primary window already handles it)
    let secondary_monitors: Vec<_> = all_monitors.into_iter().skip(1).collect();

    let mut windows = Vec::new();

    for mut mon_info in secondary_monitors {
        if let Some(monitor) = &mon_info.monitor {
            let window = gtk::ApplicationWindow::new(app);
            window.set_application(Some(app));
            window.set_title(Some("ReGreet Secondary"));
            window.set_hide_on_close(false);

            // Setup window appearance
            window.set_decorated(false);
            window.set_deletable(false);
            window.set_modal(false);

            // Fullscreen on this monitor
            window.fullscreen_on_monitor(monitor);

            window.present();

            mon_info.window = Some(window);
            windows.push(Rc::new(RefCell::new(mon_info)));
        }
    }

    windows
}
