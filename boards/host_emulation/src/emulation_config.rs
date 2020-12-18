use clap::{App, Arg};
use std::path::{Path, PathBuf};
use std::vec::Vec;
use tempfile;

use crate::Result;

pub static mut HOST_CONFIG: Option<&'static Config> = None;

pub struct AppInfo {
    bin_path: PathBuf,
}

impl AppInfo {
    fn new(path: &str) -> AppInfo {
        AppInfo {
            bin_path: PathBuf::from(path),
        }
    }

    pub fn bin_path(&self) -> &Path {
        self.bin_path.as_path()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    NONE,
    ERROR,
    WARNING,
    INFO,
    DEBUG,
}

impl LogLevel {
    pub fn from(lvl: u8) -> LogLevel {
        if lvl > 4 {
            return LogLevel::DEBUG;
        }
        match lvl {
            0 => LogLevel::NONE,
            1 => LogLevel::ERROR,
            2 => LogLevel::WARNING,
            3 => LogLevel::INFO,
            4 => LogLevel::DEBUG,
            _ => LogLevel::NONE,
        }
    }
}

pub struct Config {
    runtime_path: tempfile::TempDir,
    apps: Vec<AppInfo>,
    pub emulation_log_level: LogLevel,
    pub app_log_level: u8,
}

impl Config {
    pub fn get() -> &'static Config {
        unsafe { HOST_CONFIG.unwrap() }
    }

    pub fn set(config: &'static Config) {
        let config_set = unsafe { HOST_CONFIG.is_some() };
        if config_set {
            panic!("Can't set config twice");
        }
        unsafe {
            HOST_CONFIG = Some(config);
        }
    }

    pub fn from_cmd_line_args() -> Result<Config> {
        let arg_match = App::new("The Tock kernel")
            .arg(
                Arg::with_name("apps")
                    .short("a")
                    .long("apps")
                    .help("A comma separated list of path names to app binaries")
                    .takes_value(true)
                    .multiple(true)
                    .use_delimiter(true)
                    .required(false),
            )
            .arg(
                Arg::with_name("emulation_log")
                    .short("e")
                    .long("emulation_log")
                    .takes_value(true)
                    .help("Log level 0 for no logs, 1 errors, 2 warnings, 3 info, 4 dbg")
                    .required(false),
            )
            .arg(
                Arg::with_name("app_log")
                    .short("p")
                    .long("app_log")
                    .multiple(true)
                    .takes_value(true)
                    .help("Log level 0 for no logs, 1 errors, 2 warnings, 3 info, 4 dbg")
                    .required(false),
            )
            .get_matches();

        let runtime_path = tempfile::tempdir()?;

        let app_log_level = arg_match.value_of("app_log").unwrap_or("0");
        let app_log_level = app_log_level.parse::<u8>().unwrap();

        let apps: Vec<AppInfo> = match arg_match.values_of("apps") {
            Some(app_list) => app_list.map(|app| AppInfo::new(app)).collect(),
            None => Vec::default(),
        };

        let emulation_log_level = arg_match.value_of("emulation_log").unwrap_or("0");
        let emulation_log_level = emulation_log_level.parse::<u8>().unwrap();
        let emulation_log_level = LogLevel::from(emulation_log_level);

        Ok(Config {
            runtime_path,
            apps,
            emulation_log_level,
            app_log_level,
        })
    }

    pub fn syscall_rx_path(&self) -> PathBuf {
        self.runtime_path.path().join(Path::new("kernel_rx"))
    }

    pub fn syscall_tx_path(&self) -> PathBuf {
        self.runtime_path.path().join(Path::new("kernel_tx"))
    }

    pub fn apps(&self) -> &Vec<AppInfo> {
        &self.apps
    }
}
