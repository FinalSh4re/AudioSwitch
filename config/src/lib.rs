use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub installed: bool,
    pub profiles: Vec<Profile>,
    pub active_profile: Option<String>,
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
    pub color: Option<String>,
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

impl Profile {
    pub fn set_profile_id(mut self, id: u64) -> Self {
        self.profile_id = id;
        self
    }

    pub fn set_profile_name(mut self, name: &str) -> Self {
        self.profile_name = name.to_string();
        self
    }

    pub fn set_input_device(mut self, id: &str, name: &str) -> Self {
        self.input_id = id.to_string();
        self.input_name = name.to_string();
        self
    }

    pub fn set_output_device(mut self, id: &str, name: &str) -> Self {
        self.output_id = id.to_string();
        self.output_name = name.to_string();
        self
    }

    pub fn set_hotkey(mut self, modifier: Option<String>, hotkey: String) -> Self {
        self.hotkey.modifier = modifier;
        self.hotkey.hotkey = hotkey;
        self
    }

    pub fn set_profile_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }
}
