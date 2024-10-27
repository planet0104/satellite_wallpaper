#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
mod config;
mod downloader;
mod app;
mod def;
mod server;
mod ui;


pub fn main() -> Result<()>{
    #[cfg(not(target_os = "android"))]
    app::run()?;
    Ok(())
}