use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub installed: bool,
    pub profiles: Vec<Profile>,
    pub next_profile: Option<HotkeyConfig>,
    pub previous_profile: Option<HotkeyConfig>,
}
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    pub profile_id: u64,
    pub profile_name: String,
    pub input_id: String,
    pub input_name: String,
    pub output_id: String,
    pub output_name: String,
    pub hotkey: HotkeyConfig,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub modifier: Option<String>,
    pub hotkey: String,
}

impl std::fmt::Display for HotkeyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            &self
                .modifier
                .as_ref()
                .map_or_else(|| "".to_string(), |i| format!("{}+", i.to_string())),
            &self.hotkey
        )
    }
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Profile {}:\nInput Device: {}\nOutput Device: {}\nHotkey: {}\n",
            self.profile_name, self.input_name, self.output_name, self.hotkey
        )
    }
}
