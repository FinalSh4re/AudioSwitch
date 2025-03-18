use anyhow::{Context, Result};
use inquire::validator::Validation;
use inquire::{Confirm, Select, Text};
use win_hotkey::keys::VirtualKey;

use crate::devices::{DeviceType, enumerate_devices};
use crate::hotkeys::Hotkey;

pub fn new_profile() -> Result<()> {
    let profile_name_validator = |input: &str| {
        let config: crate::config::Config = confy::load("AudioSwitch", None)?;

        if config
            .profiles
            .iter()
            .any(|it| *it.profile_name == input.to_string())
        {
            Ok(Validation::Invalid("Profile name already used.".into()))
        } else if input == "" {
            Ok(Validation::Invalid("Profile name can't be empty.".into()))
        } else {
            Ok(Validation::Valid)
        }
    };

    let color_validator = |input: &str| {
        let hex_chars: [char; 16] = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
        ];

        if !input.starts_with("#") {
            Ok(Validation::Invalid(
                "Invalid color code! Hex code needs to start with a #.".into(),
            ))
        } else if input.len() != 7 {
            Ok(Validation::Invalid(
                "Invalid color code! Code needs to be 7 characters long.".into(),
            ))
        } else {
            for c in input.trim_start_matches("#").to_ascii_lowercase().chars() {
                if !hex_chars.contains(&c) {
                    println!("{}", c);
                    return Ok(Validation::Invalid(
                        "Invalid color code! Invalid Hex value, valid characters are: [0-9A-F]"
                            .into(),
                    ));
                }
            }
            Ok(Validation::Valid)
        }
    };

    let available_devices = enumerate_devices()?;

    let profile_name = Text::new("Enter a name for the new profile:")
        .with_validator(profile_name_validator)
        .prompt()
        .context("No profile name specified.")?;

    let color = Text::new("Enter a hex color code (eg. #FFFFFF).")
        .with_help_message("The color is assigned to the tray icon when the profile is active.")
        .with_validator(color_validator)
        .prompt()
        .context("No color specified.")?;

    let output_device = Select::new(
        "Select output device:",
        available_devices
            .iter()
            .filter(|it| it.device_type == DeviceType::Output)
            .collect(),
    )
    .prompt()
    .context("No output device selected")?;

    let input_device = Select::new(
        "Select input device:",
        available_devices
            .iter()
            .filter(|it| it.device_type == DeviceType::Input)
            .collect(),
    )
    .prompt()
    .context("No input device selected")?;

    let mut config: crate::config::Config = confy::load("AudioSwitch", None)?;

    #[allow(unused_assignments)]
    let mut hotkey = Hotkey {
        modifier: None,
        main_key: VirtualKey::Vk0,
    };

    'outer: loop {
        hotkey = crate::hotkeys::create_hotkey()?;

        for p in config.profiles.iter() {
            if p.hotkey.modifier == hotkey.modifier.map(|i| i.to_string())
                && p.hotkey.hotkey == hotkey.main_key.to_string()
            {
                println!(
                    "Hotkey: {} already in use for profile '{}'.",
                    hotkey, p.profile_name
                );
                continue 'outer;
            }
        }

        if let Some(ref h) = config.next_profile {
            if h.modifier == hotkey.modifier.map(|i| i.to_string())
                && h.hotkey == hotkey.main_key.to_string()
            {
                println!(
                    "Hotkey: {} already in use for 'next profile' switch.",
                    hotkey
                );
                continue 'outer;
            }
        }

        if let Some(ref h) = config.previous_profile {
            if h.modifier == hotkey.modifier.map(|i| i.to_string())
                && h.hotkey == hotkey.main_key.to_string()
            {
                println!(
                    "Hotkey: {} already in use for 'previous profile' switch.",
                    hotkey
                );
                continue 'outer;
            }
        }

        break;
    }

    let profile = crate::config::Profile::default()
        .set_profile_id(0)
        .set_profile_name(&profile_name)
        .set_input_device(&input_device.device_id, &input_device.name)
        .set_output_device(&output_device.device_id, &output_device.name)
        .set_hotkey(
            hotkey.modifier.map(|i| i.to_string()),
            hotkey.main_key.to_string(),
        )
        .set_profile_color(color);

    config.profiles.push(profile);

    confy::store("AudioSwitch", None, config)?;

    Ok(())
}

pub fn delete_profile() -> Result<()> {
    let mut config: crate::config::Config = confy::load("AudioSwitch", None)?;

    let mut profiles = Vec::new();

    for p in config.profiles.iter() {
        profiles.push(p.profile_name.clone());
    }

    if let Some(profile_to_delete) =
        Select::new("Select Profile you want to delete:", profiles).prompt_skippable()?
    {
        println!(
            "{}",
            config
                .profiles
                .iter()
                .filter(|i| i.profile_name == profile_to_delete)
                .last()
                .expect("Profile does not exist.")
        );

        if Confirm::new(&format!(
            "Do you want to delete profile {}? (y/n)",
            profile_to_delete
        ))
        .prompt()?
        {
            config.profiles = config
                .profiles
                .into_iter()
                .filter(|it| it.profile_name != profile_to_delete)
                .collect();
        }

        confy::store("AudioSwitch", None, config)?;
    };

    Ok(())
}
