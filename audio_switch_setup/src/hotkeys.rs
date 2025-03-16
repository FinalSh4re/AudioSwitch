use anyhow::{Context, Result};
use inquire::Select;
use win_hotkey::keys::{ModifiersKey, VirtualKey};

#[derive(Debug)]
pub struct Hotkey {
    pub modifier: Option<ModifiersKey>,
    pub main_key: VirtualKey,
}

impl Hotkey {
    fn new(modifier: Option<ModifiersKey>, main_key: VirtualKey) -> Self {
        Self { modifier, main_key }
    }
}

impl std::fmt::Display for Hotkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // <Modifier>+<Hotkey>
        write!(
            f,
            "{}{}",
            &self
                .modifier
                .map_or_else(|| "".to_string(), |i| format!("{}+", i.to_string())),
            &self.main_key.to_string()
        )
    }
}

pub fn create_hotkey() -> Result<Hotkey> {
    let modifier_keys = vec!["ALT", "CTRL", "SHIFT", "WIN"]
        .iter()
        .map(|it| it.to_string())
        .collect::<Vec<String>>();

    let mut num_keys = (0..10).map(|it| it.to_string()).collect::<Vec<String>>();
    let mut alpha_keys = (65..91)
        .map(|it| char::from_u32(it).unwrap().to_string())
        .collect::<Vec<String>>();

    let mut special_keys = vec![
        "BACKSPACE",
        "CLEAR",
        "RETURN",
        "PAUSE",
        "CAPITAL",
        "ESC",
        "SPACE",
        "PRIOR",
        "NEXT",
        "END",
        "HOME",
        "TAB",
        "UP",
        "LEFT",
        "RIGHT",
        "DOWN",
        "SELECT",
        "PRINT",
        "EXECUTE",
        "SNAPSHOT",
        "INSERT",
        "DELETE",
        "HELP",
        "APPS",
        "SLEEP",
        "NUMPAD0",
        "NUMPAD1",
        "NUMPAD2",
        "NUMPAD3",
        "NUMPAD4",
        "NUMPAD5",
        "NUMPAD6",
        "NUMPAD7",
        "NUMPAD8",
        "NUMPAD9",
        "NUMPADMULTIPLY",
        "NUMPADADD",
        "NUMPADSEPARATOR",
        "NUMPADSUBTRACT",
        "NUMPADDECIMAL",
        "NUMPADDIVIDE",
        "F1",
        "F2",
        "F3",
        "F4",
        "F5",
        "F6",
        "F7",
        "F8",
        "F9",
        "F10",
        "F11",
        "F12",
        "F13",
        "F14",
        "F15",
        "F16",
        "F17",
        "F18",
        "F19",
        "F20",
        "F21",
        "F22",
        "F23",
        "F24",
        "NUMLOCK",
        "SCROLL",
        "BROWSER_BACK",
        "BROWSER_FORWARD",
        "BROWSER_REFRESH",
        "BROWSER_STOP",
        "BROWSER_SEARCH",
        "BROWSER_FAVORITES",
        "BROWSER_HOME",
        "VOLUME_MUTE",
        "VOLUME_DOWN",
        "VOLUME_UP",
        "MEDIA_NEXT_TRACK",
        "MEDIA_PREV_TRACK",
        "MEDIA_STOP",
        "MEDIA_PLAY_PAUSE",
        "LAUNCH_MAIL",
        "LAUNCH_MEDIA_SELECT",
        "LAUNCH_APP1",
        "LAUNCH_APP2",
        ";",
        "+",
        ",",
        "-",
        ".",
        "/",
        "`",
        "[",
        "\\",
        "]",
        "'",
        "ATTN",
        "CRSEL",
        "EXSEL",
        "PLAY",
        "ZOOM",
    ]
    .iter()
    .map(|it| it.to_string())
    .collect::<Vec<String>>();

    let mut keys = Vec::new();
    keys.append(&mut num_keys);
    keys.append(&mut alpha_keys);
    keys.append(&mut special_keys);

    let modifier_key = Select::new("Select optional Modifier:", modifier_keys)
        .with_help_message("Press Esc to select no Modifier key...")
        .prompt_skippable()?;
    let hotkey = Select::new("Select Hotkey:", keys)
        .prompt()
        .context("No Hotkey defined.")?;

    Ok(Hotkey::new(
        modifier_key.map(|i| ModifiersKey::from_keyname(&i).unwrap()),
        VirtualKey::from_keyname(&hotkey).unwrap(),
    ))
}
