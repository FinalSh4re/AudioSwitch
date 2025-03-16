mod config;
mod devices;
mod hotkeys;
mod profiles;
mod tasks;

use std::ffi::OsString;
use std::fs::DirBuilder;
use std::io::{self, Write};
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Result, anyhow};
use config::Config;
use inquire::Select;
use sysinfo::System;
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize};

fn main() -> Result<()> {
    print_hero();
    main_menu()?;

    Ok(())
}

fn main_menu() -> Result<()> {
    let menu = vec![
        Menu::Install,
        Menu::AddProfile,
        Menu::DeleteProfiles,
        Menu::Uninstall,
        Menu::Quit,
    ];

    loop {
        match Select::new("AudioSwitch Main Menu:", menu.iter().collect()).prompt() {
            Ok(Menu::AddProfile) => { 
                profiles::new_profile()?;
                restart_service()?;
            },
            Ok(Menu::DeleteProfiles) => {
                profiles::delete_profile()?;
                restart_service()?;
            } ,
            Ok(Menu::Install) => {
                install_program()?;
            }
            Ok(Menu::Uninstall) => {
                uninstall()?;
                break;
            }
            _ => break,
        }
    }

    print!("Press Enter to continue...");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    Ok(())
}

#[derive(Debug, PartialEq)]
enum Menu {
    Install,
    AddProfile,
    DeleteProfiles,
    Uninstall,
    Quit,
}

impl std::fmt::Display for Menu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Menu::Install => write!(f, "Install AudioSwitch and add to autostart."),
            Menu::AddProfile => write!(f, "Add Audio Profile."),
            Menu::DeleteProfiles => write!(f, "Delete existing Profile."),
            Menu::Uninstall => write!(f, "Uninstall AudioSwitch and remove from autostart."),
            Menu::Quit => write!(f, "Quit."),
        }
    }
}

fn install_program() -> Result<()> {
    let mut cfg: Config = confy::load("AudioSwitch", None)?;

    if cfg.installed {
        println!("    Program already installed.");
        return Ok(());
    }

    let service_bin = include_bytes!(env!("CARGO_BIN_FILE_AUDIO_SWITCH_SERVICE"));
    let install_dir = directories::BaseDirs::new()
        .unwrap()
        .data_local_dir()
        .join("AudioSwitch");

    DirBuilder::new().recursive(true).create(&install_dir)?;

    let install_file_path = &install_dir.join("AudioSwitchService.exe");
    std::fs::write(&install_file_path, service_bin)?;

    let current_exe = std::env::current_exe()?;
    std::fs::copy(current_exe, &install_dir.join("AudioSwitchSetup.exe"))?;

    println!("    Copied Program Files... Done ✔");

    std::os::windows::fs::symlink_file(
        &install_dir.join("AudioSwitchSetup.exe"),
        directories::UserDirs::new()
            .expect("Failed creating UserDir Instance.")
            .desktop_dir()
            .ok_or(anyhow!("Failed to get Desktop path."))?
            .join("AudioSwitch.exe"),
    )?;

    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)?;
    }

    tasks::create_autostart_task(&install_file_path)?;

    unsafe {
        CoUninitialize();
    }

    println!("    Creating Autostart Entry... Done ✔");

    cfg.installed = true;

    confy::store("AudioSwitch", None, cfg)?;

    print!("    Starting Service...");
    io::stdout().flush().expect("Failed to flush stdout");

    std::process::Command::new(&install_file_path).spawn()?;

    println!(" Done ✔");
    io::stdout().flush().expect("Failed to flush stdout");

    println!("    > Program was sucessfully installed!");

    Ok(())
}

fn uninstall() -> Result<()> {
    let cfg: Config = confy::load("AudioSwitch", None)?;

    if !cfg.installed {
        println!("    Program not installed.");
        return Ok(());
    }

    if !inquire::Confirm::new("Are you sure you want to uninstall AudioSwitch? This will remove all user settings and hotkeys! (y/n)").prompt()? {
        println!("    Uninstaller aborted!");
        return Ok(());
    }

    let mut sys = System::new_all();
    sys.refresh_all();
    let service_name = OsString::from_str("AudioSwitchService.exe")?;

    for p in sys.processes_by_exact_name(&service_name) {
        print!("    Stopping running Service...");
        io::stdout().flush().expect("Failed to flush stdout");

        p.kill();

        sleep(Duration::from_secs(5));
        println!(" Done ✔");
    }

    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)?;
    }

    tasks::delete_task()?;

    println!("    Removing Autostart Entry... Done ✔");

    unsafe {
        CoUninitialize();
    }

    let install_path = directories::BaseDirs::new()
        .unwrap()
        .data_local_dir()
        .join("AudioSwitch")
        .join("AudioSwitchService.exe");

    match std::fs::remove_file(install_path) {
        Ok(_) => {
            println!("    Removing Service File... Done ✔")
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("Nothing to delete, program already uninstalled?")
        }
        Err(e) => eprintln!("Error while uninstalling: {}", e),
    }

    let desktop_symlink = directories::UserDirs::new()
        .expect("Failed creating UserDir Instance.")
        .desktop_dir()
        .ok_or(anyhow!("Failed to get Desktop path."))?
        .join("AudioSwitch.exe");

    match std::fs::remove_file(desktop_symlink) {
        Ok(_) => {
            println!("    Removing Desktop Shortcut... Done ✔")
        }
        _ => {}
    }

    let cfg_path = confy::get_configuration_file_path("AudioSwitch", None)?;
    std::fs::remove_file(cfg_path)?;

    println!("    Cleaning up configuration files... Done ✔");

    self_replace::self_delete()?;

    println!("    > Program sucessfully uninstalled!");

    Ok(())
}

fn restart_service() -> Result<()> {

    let service_file_path = directories::BaseDirs::new()
        .unwrap()
        .data_local_dir()
        .join("AudioSwitch")
        .join("AudioSwitchService.exe");

    let mut sys = System::new_all();
    sys.refresh_all();
    let service_name = OsString::from_str("AudioSwitchService.exe")?;

    for p in sys.processes_by_exact_name(&service_name) {
        print!("    Restarting Service...");
        io::stdout().flush().expect("Failed to flush stdout");

        p.kill();
        sleep(Duration::from_secs(5));

        std::process::Command::new(&service_file_path).spawn()?;
        sleep(Duration::from_secs(2));

        println!(" Done ✔");
    }
    Ok(())
}

fn print_hero() {
    let hero = r#"
    _             _ _      ____          _ _       _
   / \  _   _  __| (_) ___/ ___|_      _(_) |_ ___| |__
  / _ \| | | |/ _` | |/ _ \___ \ \ /\ / / | __/ __| '_ \
 / ___ \ |_| | (_| | | (_) |__) \ V  V /| | || (__| | | |
/_/   \_\__,_|\__,_|_|\___/____/ \_/\_/ |_|\__\___|_| |_|
    "#;

    println!("{}", hero);
}
