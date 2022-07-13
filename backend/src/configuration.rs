use ini::Ini;
use std::sync::{Arc, Mutex};
use std::path::{Path};

// https://stackoverflow.com/a/50323079
#[cfg(not(target_os = "windows"))]
fn adjust_canonicalization<P: AsRef<Path>>(p: P) -> String {
    p.as_ref().display().to_string()
}

#[cfg(target_os = "windows")]
fn adjust_canonicalization<P: AsRef<Path>>(p: P) -> String {
    const VERBATIM_PREFIX: &str = r#"\\?\"#;
    let p = p.as_ref().display().to_string();
    if p.starts_with(VERBATIM_PREFIX) {
        p[VERBATIM_PREFIX.len()..].to_string()
    } else {
        p
    }
}

#[derive(Clone)]
pub struct Configuration {
    local_conf: Arc<Mutex<Ini>>,
    global_conf: Arc<Mutex<Ini>>
}

pub struct LaunchInfo {
    pub player_name: String,
    pub startup_movie_disabled: bool,
    pub bit_version: String,
}

impl Configuration {
    pub fn load_or_default() -> Self {
        let local_conf = match Ini::load_from_file("RenegadeX-Launcher.ini") {
            Ok(conf) => conf,
            Err(_e) => {
              let mut conf = Ini::new();
              conf.with_section(Some("RenX_Launcher"))
                .set("GameLocation", "../")
                .set("VersionUrl", "https://static.ren-x.com/launcher_data/version/release.json")
                .set("ServerListUrl", "https://serverlist.ren-x.com/servers.jsp")
                .set("LauncherTheme", "dom");
                conf.write_to_file("RenegadeX-Launcher.ini").expect("");
                conf
            }
        };

        let mut config_directory = Configuration::get_global_configuration_directory();
        config_directory.push("Renegade X Launcher.ini");
        let global_conf = match Ini::load_from_file(config_directory) {
            Ok(conf) => {
                conf
            },
            Err(_e) => {
                let mut conf = Ini::new();
                conf.with_section(Some("RenX_Launcher"))
                .set("PlayerName", "UnknownPlayer")
                .set("64-bit-version", "true")
                .set("skipMovies", "false");
                conf
            }
        };
        Self {
            local_conf: Arc::new(Mutex::new(local_conf)), 
            global_conf: Arc::new(Mutex::new(global_conf))
        }
    }
    
    fn save_global(&self) {
        let mut config_directory = Configuration::get_global_configuration_directory();
        std::fs::create_dir_all(&config_directory).expect("Creation of config-directory went wrong!");
        config_directory.push("Renegade X Launcher.ini");
    
        self.global_conf.lock().unwrap().write_to_file(config_directory.to_str().expect("")).expect("");
    }

    pub fn get_game_version(&self) -> String {
        let game_location = self.get_game_location();
        match Ini::load_from_file(format!("{}/UDKGame/Config/DefaultRenegadeX.ini", game_location)) {
          Ok(conf) => {
            let section = conf.section(Some("RenX_Game.Rx_Game".to_owned())).unwrap();
            section.get("GameVersion").unwrap().to_string()
          },
          Err(_e) => {
            "Not installed".to_string()
          }
        }
    }

    // Returns an absolute path to the UDK executable
    pub fn get_game_executable(&self) -> String {
        let mut game_dir = self.get_game_directory_abs();
        let launch_info =  self.get_launch_info();

        game_dir.push("Binaries");
        game_dir.push(format!("Win{}", launch_info.bit_version));
        game_dir.push("UDK");
        game_dir.set_extension("exe");

        // On Windows, adjust the path to strip out the UNC prefix (CreateProcess does not support it)
        adjust_canonicalization(game_dir)
    }

    // Returns the absolute directory to the game
    pub fn get_game_directory_abs(&self) -> std::path::PathBuf {
        let game_location = self.get_game_location();
        let game_dir_abs_path = std::path::PathBuf::from(game_location).canonicalize().expect("Couldn't create absolute path from relative one");
        game_dir_abs_path
    }

    pub fn get_game_location(&self) -> String {
        self.get_local_setting("GameLocation")
    }

    pub fn get_launch_info(&self) -> LaunchInfo {
        let global_conf = self.global_conf.lock().expect("");
        let global_section = global_conf.section(Some("RenX_Launcher".to_owned())).expect("");
        let playername = global_section.get("PlayerName").expect("").clone();
        let startup_movie_disabled = global_section.get("skipMovies").expect("").clone() == "true";
        let bit_version = if global_section.get("64-bit-version").expect("").clone() == "true" { "64" } else { "32" };

        LaunchInfo {
            player_name: playername.to_owned(),
            startup_movie_disabled: startup_movie_disabled.to_owned(),
            bit_version: bit_version.to_owned()
        }
    }

    pub fn get_video_location(&self, map_name: String) -> String {
        let game_location = self.get_game_location();
        let mut absolute_path = std::path::PathBuf::from(game_location).canonicalize().expect("Couldn't create absolute path from relative one");
        absolute_path.push("PreviewVids");
        absolute_path.push(map_name);
        absolute_path.set_extension("avi");
        if !absolute_path.is_file() {
          absolute_path.pop();
          absolute_path.push("Default.avi");
        }
        url::Url::from_file_path(absolute_path).expect("Cannot convert path to a url.").into()
    }

    pub fn get_playername(&self) -> String {
        self.get_global_setting("PlayerName")
    }

    pub fn set_playername(&self, username: &str) {
        self.set_global_setting("PlayerName", username);
    }

    pub fn get_global_setting(&self, setting: &str) -> String {
        let conf_unlocked = self.global_conf.clone();
        let conf = conf_unlocked.lock().expect("");
        let section = conf.section(Some("RenX_Launcher".to_owned())).expect("");
        section.get(&setting).expect("").to_string()
    }

    pub fn set_global_setting(&self, setting: &str, value: &str) {
        let conf_unlocked = self.global_conf.clone();
        let mut conf = conf_unlocked.lock().expect("");
        let mut section = conf.with_section(Some("RenX_Launcher".to_owned()));
        section.set(setting, value);
        drop(conf);

        self.save_global();
    }

    pub fn get_local_setting(&self, setting: &str) -> String {
        let conf_unlocked = self.local_conf.clone();
        let conf = conf_unlocked.lock().expect("");
        let section = conf.section(Some("RenX_Launcher".to_owned())).expect("");
        section.get(&setting).expect("").to_string()
    }

    pub fn get_log_directory(&self) -> String {
        let mut config_directory = Configuration::get_global_configuration_directory();
        config_directory.push("logs");
        config_directory.to_str().expect("").to_owned()
    }

    pub fn get_game_log_directory(&self) -> String {
        format!("{}", self.get_local_setting("GameLocation"))
    }

    fn get_global_configuration_directory() -> std::path::PathBuf {
        let mut config_directory = dirs::config_dir().expect("");
        config_directory.push("Renegade X");
        config_directory
    }

    pub fn get_launcher_theme(&self) -> String {
        self.get_local_setting("LauncherTheme")
    }

    pub fn get_version_url(&self) -> String {
        self.get_local_setting("VersionUrl")
    }

    pub fn get_serverlist_url(&self) -> String {
        self.get_local_setting("ServerListUrl")
    }
}