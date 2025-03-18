#![windows_subsystem = "windows"]

mod config;
mod dyn_icon;
mod service;
mod tray;

use anyhow::Result;

pub fn main() -> Result<()> {
    let service = service::Service::new();
    service.run_app();

    Ok(())
}
