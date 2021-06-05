#![windows_subsystem = "windows"]
use anyhow::Result;
use log::{LevelFilter, info};
mod config;
mod downloader;
mod def;
mod server;
mod app;

fn main() -> Result<()> {
    env_logger::Builder::new().filter_level(LevelFilter::Info).init();    
    let result = app::start_app(1);
    info!("程序结束: {}", result);
    Ok(())
}