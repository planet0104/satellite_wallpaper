
use std::time::Instant;
use std::path::Path;
use async_std::task::spawn_blocking;
use def::APP_NAME;
use log::warn;
use log::{error, info};
use slint::Rgb8Pixel;
use slint::{Image, SharedPixelBuffer, Timer, TimerMode, Weak};
use async_std::sync::{Arc, Mutex};
use crate::config::Config;
use crate::downloader;
use crate::def;
use crate::downloader::is_downlading;
use crate::server;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
pub use android::*;

static DEFAULT_IMAGE:&[u8] = include_bytes!("../../res/icon_loading.png");

pub async fn open_wall_paper_image(url: &str) -> anyhow::Result<SharedPixelBuffer<Rgb8Pixel>>{
    let url = url.to_string();
    spawn_blocking(move ||{
        let t = Instant::now();
        info!("开始读取图片文件....................");
        let file = image::open(Path::new(&url))?;
        info!("读取图片文件....................{}ms", t.elapsed().as_millis());
        let t = Instant::now();
        let img = file.to_rgb8();
        info!("读取图片文件....................{}x{} {}ms", img.width(), img.height(), t.elapsed().as_millis());
        let t = Instant::now();
        let buf = SharedPixelBuffer::clone_from_slice(
            &img,
            img.width(),
            img.height(),
        );
        info!("读取图片文件.................... buf={}x{} {}ms", buf.width(), buf.height(), t.elapsed().as_millis());
        Ok(buf)
    }).await
}

fn update_config_ui(app: Weak<crate::ui::Main>, cfg:&Config){
    let app = match app.upgrade() {
        Some(app) => app,
        None => return
    };
    app.set_wallpaper_file(cfg.current_wallpaper_file.as_str().into());
    app.set_h8_data_url(cfg.download_url_h8.as_str().into());
    app.set_f4a_data_url(cfg.download_url_fy4b.as_str().into());
    app.set_config_file(cfg.config_path.as_str().into());
    app.set_current_interval_index(cfg.update_interval as i32/10 - 1);
    app.set_current_size_index(cfg.display_type as i32-1);
    app.set_current_satellite_index(if cfg.current_wallpaper_date.contains("fy4b"){ 0 }else{ 1 });
}

pub fn open_main_window(){
    use slint::ComponentHandle;
    info!("启动窗口...");
    let app = crate::ui::Main::new().unwrap();

    #[cfg(target_os = "android")]
    {
        let _ = slint::spawn_local(server::start_update_loop(std::sync::Arc::new(std::sync::Mutex::new(false))));   
    }

    let config = Arc::new(Mutex::new(Config::default()));
    
    app.set_is_startup(is_app_registered_for_startup(APP_NAME).unwrap_or(false));
    let app_clone = app.as_weak();
    app.on_open_image_file(move || {
        open_file(&app_clone.unwrap().get_wallpaper_file());
    });

    app.on_sync_now(move || {
        let _ = slint::spawn_local(async move {
            if !is_downlading(){
                let mut cfg = Config::load_or_default().await;
                downloader::set_wallpaper_default(&mut cfg).await;
            }else{
                info!("按钮点击 正在下载中...");
            }
        });
    });

    let config_clone = config.clone();
    app.on_change_satellite(move |select_index| {
        let config_clone = config_clone.clone();
        let _ = slint::spawn_local(async move {
            let mut cfg = config_clone.lock().await;
            if select_index == 0{
                cfg.satellite_name = "fy4b".to_string();
            }else{
                cfg.satellite_name = "h8".to_string();
            }
            let _ = cfg.save_to_file().await;

            let mut cfg_clone = cfg.clone();
            
            let _ = slint::spawn_local(async move {
                //立即更新
                downloader::set_wallpaper_default(&mut cfg_clone).await;
                info!("on_change_satellite 壁纸更新完成...");
            });
        });
    });

    let config_clone = config.clone();
    app.on_change_interval(move |select_index| {
        let config_clone = config_clone.clone();
        let _ = slint::spawn_local(async move {
            let mut cfg = config_clone.lock().await;
            let intervals = [10, 20, 30, 40, 50, 60];
            cfg.update_interval = intervals[select_index as usize];
            let _ = cfg.save_to_file().await;
        });
    });

    let config_clone = config.clone();
    app.on_change_wallpaper_size(move |select_index| {
        let config_clone = config_clone.clone();
        let _ = slint::spawn_local(async move {
            let mut cfg = config_clone.lock().await;
            cfg.display_type = select_index as u32 + 1;
            let _ = cfg.save_to_file().await;
        });
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
    let timer = Timer::default();
    timer.start(TimerMode::Repeated, std::time::Duration::from_millis(3000), move || {
        let app_clone = app_clone.clone();
        let _ = slint::spawn_local(async move {
            info!("timer 开始....");
            let app = match app_clone.upgrade(){
                Some(v) => v,
                None => {
                    error!("app_clone.upgrade() 失败 timer中.");
                    return
                }
            };
            
            let cfg = Config::load_or_default().await;
            update_config_ui(app_clone.clone(), &cfg);
            let current_wallpaper_date = app.get_current_wallpaper();
            let t = Instant::now();
            let is_downloading = is_downlading();
            warn!("锁定 is_downloading 耗时:{}ms", t.elapsed().as_millis());
            if is_downloading{
                //显示更新状态文字
                app.set_download_status("正在下载壁纸...".into());
            }else{
                warn!("Timer 未下载 开始设置状态...");
                app.set_download_status(format!("上次更新: {}", cfg.get_last_update_time_str()).into());
                warn!("Timer 未下载 检查是否要更新图片...");
                if current_wallpaper_date != cfg.current_wallpaper_date{
                    warn!("Timer 未下载 需要更新图片...");
                    app.set_current_wallpaper(cfg.current_wallpaper_date.as_str().into());
                    warn!("Timer 未下载 打开图片文件...");
                    let url = cfg.current_wallpaper_file.to_string();
                    let t = Instant::now();
                    warn!("Timer 未下载 打开图片文件 01...");
                    let buf = open_wall_paper_image(&url).await;
                    warn!("Timer 未下载 打开图片文件 02...");
                    if buf.is_err(){
                        error!("图片读取失败:{:?}", buf.err());
                        return;
                    }
                    let buf = buf.unwrap();
                    info!("加载图片耗时:{}ms", t.elapsed().as_millis());
                    let t = Instant::now();
                    let new_image = Image::from_rgb8(buf);
                    app.set_source_image(new_image);
                    info!("渲染图片耗时:{}ms", t.elapsed().as_millis());
                }
            }
            // info!("timer 结束...");
        });
    });

    let default_image = image::load_from_memory_with_format(DEFAULT_IMAGE, image::ImageFormat::Png).unwrap().to_rgba8();
    let image = Image::from_rgba8(SharedPixelBuffer::clone_from_slice(
        &default_image,
        default_image.width(),
        default_image.height(),
    ));
    app.set_source_image(image);
    
    let config_clone = config.clone();
    let app_clone = app.as_weak();
    let _ = slint::spawn_local(async move{
        update_config_ui(app_clone, &*config_clone.lock().await);
    });

    app.run().unwrap();
    info!("窗口关闭");
}