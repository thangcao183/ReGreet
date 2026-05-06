// SPDX-FileCopyrightText: 2022 The ReGreet Authors
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shared state between multiple greeter windows

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use greetd_ipc::{Response};
use tokio::sync::Mutex;
use tracing::error;

use crate::cache::Cache;
use crate::client::GreetdClient;
use crate::config::Config;
use crate::sysutil::SysUtil;

/// Shared state between all greeter windows (auth logic)
pub struct SharedGreeter {
    pub greetd_client: Arc<Mutex<GreetdClient>>,
    pub sys_util: SysUtil,
    pub cache: Cache,
    pub config: Config,
    pub demo: bool,
}

impl SharedGreeter {
    pub async fn new(config_path: &Path, demo: bool) -> Result<Self, String> {
        let config = Config::new(config_path);

        let greetd_client = Arc::new(Mutex::new(
            GreetdClient::new(demo)
                .await
                .map_err(|e| format!("Couldn't initialize greetd client: {e}"))?,
        ));

        let sys_util = SysUtil::new(&config, demo)
            .await
            .map_err(|e| format!("Couldn't read available users and sessions: {e}"))?;

        Ok(Self {
            greetd_client,
            sys_util,
            cache: Cache::new(),
            config,
            demo,
        })
    }

    /// Run a command and log any errors in a background thread.
    pub fn run_cmd(command: &[String]) {
        let command = command.to_vec();
        tokio::spawn(async move {
            let mut process = Command::new(&command[0]);
            process.args(command[1..].iter());
            match process.output() {
                Ok(output) => {
                    if !output.status.success() {
                        if let Ok(err) = std::str::from_utf8(&output.stderr) {
                            error!("Failed to launch command: {err}")
                        } else {
                            error!("Failed to launch command: {:?}", output.stderr)
                        }
                    }
                }
                Err(err) => error!("Failed to launch command: {err}"),
            }
        });
    }
}
