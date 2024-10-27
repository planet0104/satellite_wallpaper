use std::{ffi::OsStr, mem, os::windows::prelude::OsStrExt, path::{Path, PathBuf}, process::Command, time::Duration};
use anyhow::{anyhow, Result};
use async_std::task::block_on;
use log::{info, LevelFilter};
use windows::{core::PCWSTR, Win32::{Foundation::{HWND, MAX_PATH, RECT}, UI::{Shell::{SHGetSpecialFolderPathW, ShellExecuteW, CSIDL_STARTUP}, WindowsAndMessaging::{GetDesktopWindow, GetWindowRect, SW_SHOWNORMAL}}}};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem}, MouseButtonState, TrayIconBuilder, TrayIconEvent
};

static TEMPLATE:&str = r"[InternetShortcut]
URL=--
IconIndex=0
IconFile=--
";

use crate::def::{APP_NAME, APP_NAME_E};

pub fn open_file(path: &str){
    unsafe{
        ShellExecuteW(None, PCWSTR(create_pcwstr("open").as_ptr()), PCWSTR(create_pcwstr(&format!("{}\0", path)).as_ptr()), None, None, SW_SHOWNORMAL);
    }
}

pub fn start_main_window(){
    // 获取当前可执行文件的路径
    let current_exe = std::env::current_exe().unwrap();

    // 构建命令行参数
    let command_args = vec!["/c".to_string()];

    // 启动新进程并传递命令行参数
    Command::new(current_exe)
        .args(&command_args)
        .spawn().unwrap();
}

fn create_pcwstr(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub fn get_screen_size() -> (i32, i32){
    let hwnd = unsafe{ GetDesktopWindow() };
    let mut rect:RECT = unsafe{ mem::zeroed() };
    if unsafe{ GetWindowRect(hwnd, &mut rect).is_ok() }{
        (rect.right-rect.left, rect.bottom-rect.top)
    }else{
        (1920, 1080)
    }
}

pub fn remove_app_for_startup(app_name:&str) -> Result<()>{
    let hwnd = unsafe{ GetDesktopWindow() };
    let mut path:[u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let _ = unsafe{ SHGetSpecialFolderPathW(hwnd, &mut path, CSIDL_STARTUP as i32, false) };
    let path = String::from_utf16(&path)?.replace("\u{0}", "");
    std::fs::remove_file(format!("{}\\{}.url", path, app_name))?;
    Ok(())
}

pub fn register_app_for_startup(app_name:&str) -> Result<()>{
    let hwnd: HWND = unsafe{ GetDesktopWindow() };
    let mut path:[u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let _ = unsafe{ SHGetSpecialFolderPathW(hwnd, &mut path, CSIDL_STARTUP as i32, false) };
    let path = String::from_utf16(&path)?.replace("\u{0}", "");
    let url_file = format!("{}\\{}.url", path, app_name);
    //写入url文件
    use std::io::Write;
    let mut file = std::fs::File::create(url_file)?;
    let exe_path = ::std::env::current_exe()?;
    if let Some(exe_path) = exe_path.to_str(){
        file.write_all(TEMPLATE.replace("--", exe_path).as_bytes())?;
        Ok(())
    }else{
        Err(anyhow!("exe路径读取失败!"))
    }
}

pub fn is_app_registered_for_startup(app_name:&str) -> Result<bool>{
    let hwnd = unsafe{ GetDesktopWindow() };
    let mut path:[u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let _ = unsafe{ SHGetSpecialFolderPathW(hwnd, &mut path, CSIDL_STARTUP as i32, false) };
    let path = String::from_utf16(&path)?.replace("\u{0}", "");
    Ok(Path::new(&format!("{}\\{}.url", path, app_name)).exists())
}

pub fn run() -> Result<()> {
    env_logger::Builder::new().filter_level(LevelFilter::Info).init();

    let args: Vec<String> = std::env::args().collect();
    for arg in &args[1..] {
        let arg = arg.to_lowercase();
        if arg.starts_with("/c") {
            //打开设置页面
            info!("收到 /c参数，打开窗口");
            super::open_main_window();
            return Ok(());
        }
    }

    std::thread::spawn(move ||{
        block_on(crate::server::start_update_loop());
    });

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
        
        if let Ok( TrayIconEvent::Click{id:_, position:_, rect:_, button, button_state }) = tray_channel.try_recv(){
            if let (tray_icon::MouseButton::Left, MouseButtonState::Down) = (button, button_state){
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

pub fn get_config_dir() -> String{
    dirs::config_dir().unwrap_or(PathBuf::default()).to_str().unwrap_or("").to_string()
}

pub fn get_app_home_dir() -> String {
    let mut app_home_dir = String::from(".");
    if let Some(home_dir) = dirs::home_dir(){
        if let Some(home_dir) = home_dir.to_str(){
            let app_home_dir_tmp = format!("{}\\{}", home_dir, APP_NAME_E);
            if Path::exists(Path::new(&app_home_dir_tmp)){
                app_home_dir = app_home_dir_tmp;
            }else{
                if let Ok(()) = std::fs::create_dir(&app_home_dir_tmp){
                    app_home_dir = app_home_dir_tmp;
                }
            }
        }
    }
    info!("app_home_dir {}", app_home_dir);
    app_home_dir
}

use windows::{core::{HSTRING, Interface}, Storage::{IStorageFile, StorageFile}, System::UserProfile::LockScreen};

/// 同步设置锁屏壁纸
pub fn set_lock_screen_image(image: &str) -> Result<()>{
    let file = StorageFile::GetFileFromPathAsync(&HSTRING::from(image))?.get()?;
    let file: IStorageFile = file.cast()?;
    LockScreen::SetImageFileAsync(&file)?.get()?;
    Ok(())
}

// 设置壁纸
pub fn set_wallpaper_from_path(image: &str) -> Result<()>{
    match wallpaper::set_from_path(image){
        Ok(()) => Ok(()),
        Err(err) => {
            Err(anyhow!("{:?}", err))
        }
    }
}

pub fn get_current_wallpaper() -> Result<String>{
    Ok(wallpaper::get().map_err(|err| anyhow!("{:?}", err))?)
}