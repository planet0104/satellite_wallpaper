#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;
use anyhow::Result;
use app::{open_main_window, start_main_window};
use def::APP_NAME;
use log::{LevelFilter, info};
mod config;
mod downloader;
mod def;
mod server;
mod app;
mod ui;

use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIconBuilder, TrayIconEvent, ClickType,
};

fn main() -> Result<()> {
    env_logger::Builder::new().filter_level(LevelFilter::Info).init();

    let args: Vec<String> = std::env::args().collect();
    for arg in &args[1..] {
        let arg = arg.to_lowercase();
        if arg.starts_with("/c") {
            //打开设置页面
            info!("收到 /c参数，打开窗口");
            open_main_window();
            return Ok(());
        }
    }

    crate::server::start_async();

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/res/iconfinder_Globe_31212.png");
    let icon = load_icon(std::path::Path::new(path));

    let event_loop = EventLoopBuilder::new().build();

    let menu = Menu::new();
    menu.append(&MenuItem::new("打开", true, None))?;
    menu.append(&MenuItem::new("退出", true, None))?;

    let _tray_icon = Some(
        TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip(APP_NAME)
            .with_icon(icon)
            .build()?,
    );

    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();
    
    let event_loop_proxy = event_loop.create_proxy();
    std::thread::spawn(move || {
        loop {
            event_loop_proxy.send_event(()).ok();
            std::thread::sleep(Duration::from_millis(50));
        }
    });

    //打开窗口
    start_main_window();

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Ok(MenuEvent { id }) = menu_channel.try_recv() {
            if id.0 == "1001"{
                //打开
                start_main_window();
            }else{
                //退出
                *control_flow = ControlFlow::Exit;
            }
        }
        
        if let Ok(TrayIconEvent {click_type, id: _, x: _, y: _, icon_rect: _ }) = tray_channel.try_recv(){
            if let ClickType::Left = click_type{
                start_main_window();
            }
        }
    });
}

fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}