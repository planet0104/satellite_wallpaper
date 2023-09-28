use std::{mem, path::Path, ffi::OsStr, os::windows::prelude::OsStrExt, process::Command, cell::RefCell, rc::Rc};
use anyhow::{anyhow, Result};
use log::info;
use slint::{Image, Timer, TimerMode, SharedPixelBuffer};
use windows::{Win32::{UI::{Shell::{ShellExecuteW, SHGetSpecialFolderPathW, CSIDL_STARTUP}, WindowsAndMessaging::{SW_SHOWNORMAL, GetDesktopWindow, GetWindowRect}}, Foundation::{HWND, RECT, MAX_PATH}}, core::{PCWSTR, HSTRING}, Storage::StorageFile, System::UserProfile::LockScreen};

static TEMPLATE:&str = r"[InternetShortcut]
URL=--
IconIndex=0
IconFile=--
";

use crate::{config, downloader::{set_wallpaper_default_async, self}, def::APP_NAME};

static DEFAULT_IMAGE:&[u8] = include_bytes!("../res/icon_loading.png");

pub fn open_file(path: &str){
    unsafe{
        ShellExecuteW(None, PCWSTR(create_pcwstr("open").as_ptr()), PCWSTR(create_pcwstr(&format!("{}\0", path)).as_ptr()), None, None, SW_SHOWNORMAL);
    }
}

pub fn open_main_window(){
    use slint::ComponentHandle;
    info!("启动窗口...");
    let app = crate::ui::Main::new().unwrap();

    let cfg = Rc::new(RefCell::new(config::load()));

    app.set_wallpaper_file(cfg.borrow().current_wallpaper_file.as_str().into());
    app.set_h8_data_url(cfg.borrow().download_url_h8.as_str().into());
    app.set_f4a_data_url(cfg.borrow().download_url_fy4a.as_str().into());
    app.set_config_file(cfg.borrow().config_path.as_str().into());

    app.set_is_startup(is_app_registered_for_startup(APP_NAME).unwrap_or(false));
    app.set_current_interval_index(cfg.borrow().update_interval as i32/10 - 1);
    app.set_current_size_index(cfg.borrow().display_type as i32-1);
    app.set_current_satellite_index(if cfg.borrow().current_wallpaper_date.contains("fy4a"){ 0 }else{ 1 });
    
    let app_clone = app.as_weak();
    app.on_open_image_file(move || {
        open_file(&app_clone.unwrap().get_wallpaper_file());
    });

    app.on_sync_now(move || {
        if !downloader::is_downloading(){
            set_wallpaper_default_async();
        }
    });

    let cfg_clone = cfg.clone();
    app.on_change_satellite(move |select_index| {
        let mut cfg = cfg_clone.borrow_mut();
        if select_index == 0{
            cfg.satellite_name = "fy4a".to_string();
        }else{
            cfg.satellite_name = "h8".to_string();
        }
        config::save(&mut cfg);
        //立即更新
        set_wallpaper_default_async();
    });

    let cfg_clone = cfg.clone();
    app.on_change_interval(move |select_index| {
        let mut cfg = cfg_clone.borrow_mut();
        let intervals = [10, 20, 30, 40, 50, 60];
        cfg.update_interval = intervals[select_index as usize];
        config::save(&mut cfg);
    });

    let cfg_clone = cfg.clone();
    app.on_change_wallpaper_size(move |select_index| {
        let mut cfg = cfg_clone.borrow_mut();
        cfg.display_type = select_index as u32 + 1;
        config::save(&mut cfg);
    });

    let app_clone = app.as_weak();
    app.on_change_startup(move |startup| {
        let is_registered = 
        if startup{
            let is_registered = is_app_registered_for_startup(APP_NAME).unwrap_or(false);
            if !is_registered{
                register_app_for_startup(APP_NAME).is_ok()
            }else{
                true
            }
        }
        else{
            let _ = remove_app_for_startup(APP_NAME);
            false
        };
        app_clone.unwrap().set_is_startup(is_registered);
    });
    
    app.on_open_home_page(move || {
        open_file("https://www.ccfish.run/satellite_wallpaper/index.html");
    });

    app.on_open_gitee_page(move || {
        open_file("https://gitee.com/planet0104-osc/satellite_wallpaper");
    });

    app.on_open_github_page(move || {
        open_file("https://github.com/planet0104/satellite_wallpaper");
    });

    //定时刷新图片
    let app_clone = app.as_weak();
    let cfg_clone = cfg.clone();
    let timer = Timer::default();
    timer.start(TimerMode::Repeated, std::time::Duration::from_millis(2000), move || {
        let app = app_clone.unwrap();
        let mut cfg = cfg_clone.borrow_mut();
        *cfg = config::load();
        let current_wallpaper_date = app.get_current_wallpaper();
        if current_wallpaper_date != cfg.current_wallpaper_date && !downloader::is_downloading(){
            info!("刷新了图片...");
            app.set_current_wallpaper(cfg.current_wallpaper_date.as_str().into());
            // app.set_wallpaper_file(cfg.current_wallpaper_file.as_str().into());
            app.set_image_frame(app.get_image_frame()+1);
        }
    });

    let cfg_clone = cfg.clone();
    app.on_render_image(move |_frame| {
        let cfg = cfg_clone.borrow();
        //渲染到Image
        let default_image =
        if cfg.current_wallpaper_file.len() > 0{
            image::open(Path::new(&cfg.current_wallpaper_file)).unwrap().to_rgba8()
        }else{
            image::load_from_memory_with_format(DEFAULT_IMAGE, image::ImageFormat::Png).unwrap().to_rgba8()
        };
        Image::from_rgba8(SharedPixelBuffer::clone_from_slice(
            &default_image,
            default_image.width(),
            default_image.height(),
        ))
    });

    app.run().unwrap();
    info!("窗口关闭");
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

/// 同步设置锁屏壁纸
pub fn set_lock_screen_image(image: &str) -> Result<()>{
    let file = StorageFile::GetFileFromPathAsync(&HSTRING::from(image))?.get()?;
    LockScreen::SetImageFileAsync(&file)?.get()?;
    Ok(())
}

pub fn remove_app_for_startup(app_name:&str) -> Result<()>{
    let hwnd = unsafe{ GetDesktopWindow() };
    let mut path:[u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    unsafe{ SHGetSpecialFolderPathW(hwnd, &mut path, CSIDL_STARTUP as i32, false) };
    let path = String::from_utf16(&path)?.replace("\u{0}", "");
    std::fs::remove_file(format!("{}\\{}.url", path, app_name))?;
    Ok(())
}

pub fn register_app_for_startup(app_name:&str) -> Result<()>{
    let hwnd: HWND = unsafe{ GetDesktopWindow() };
    let mut path:[u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    unsafe{ SHGetSpecialFolderPathW(hwnd, &mut path, CSIDL_STARTUP as i32, false) };
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
    unsafe{ SHGetSpecialFolderPathW(hwnd, &mut path, CSIDL_STARTUP as i32, false) };
    let path = String::from_utf16(&path)?.replace("\u{0}", "");
    Ok(Path::new(&format!("{}\\{}.url", path, app_name)).exists())
}