
use std::time::Instant;
use std::path::Path;
use def::APP_NAME;
use log::{error, info};
use slint::{Image, SharedPixelBuffer, Timer, TimerMode, Weak};
use async_std::sync::{Arc, Mutex, RwLock};
use crate::config::{self, Config};
use crate::downloader;
use crate::def;
use crate::server::get_download_status;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
pub use android::*;

static DEFAULT_IMAGE:&[u8] = include_bytes!("../../res/icon_loading.png");

async fn update_config_ui(app: Weak<crate::ui::Main>){
    let app = match app.upgrade() {
        Some(app) => app,
        None => return
    };
    let cfg = config::load().await;
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
    
    app.set_is_startup(is_app_registered_for_startup(APP_NAME).unwrap_or(false));
    let app_clone = app.as_weak();
    app.on_open_image_file(move || {
        open_file(&app_clone.unwrap().get_wallpaper_file());
    });

    app.on_sync_now(move || {
        let _ = slint::spawn_local(async move {
            if !get_download_status().await.downloading{
                downloader::set_wallpaper_default().await;
            }
        });
    });

    app.on_change_satellite(move |select_index| {
        let _ = slint::spawn_local(async move {
            let mut cfg = config::load().await;
            if select_index == 0{
                cfg.satellite_name = "fy4b".to_string();
            }else{
                cfg.satellite_name = "h8".to_string();
            }
            config::save(cfg).await;
            
            let _ = slint::spawn_local(async move {
                //立即更新
                downloader::set_wallpaper_default().await;
                info!("on_change_satellite 壁纸更新完成...");
            });
        });
    });

    app.on_change_interval(move |select_index| {
        let _ = slint::spawn_local(async move {
            let mut cfg = config::load().await;
            let intervals = [10, 20, 30, 40, 50, 60];
            cfg.update_interval = intervals[select_index as usize];
            config::save(cfg).await;
        });
    });

    app.on_change_wallpaper_size(move |select_index| {
        let _ = slint::spawn_local(async move {
            let mut cfg = config::load().await;
            cfg.display_type = select_index as u32 + 1;
            config::save(cfg).await;
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
            // info!("timer 开始....");
            let app = match app_clone.upgrade(){
                Some(v) => v,
                None => {
                    error!("app_clone.upgrade() 失败 timer中.");
                    return
                }
            };
            update_config_ui(app_clone.clone()).await;
            let cfg = config::load().await;
            let current_wallpaper_date = app.get_current_wallpaper();
            info!("开始获取download_status...");
            let status = get_download_status().await;
            info!("获取到download_status... {:?}", status);
            //显示更新状态文字
            info!("更新下载状态文字:{}", status.status());
            app.set_download_status(status.status().into());
            
            if current_wallpaper_date != cfg.current_wallpaper_date && !status.downloading{
                info!("刷新图片...");
                app.set_current_wallpaper(cfg.current_wallpaper_date.as_str().into());

                let url = cfg.current_wallpaper_file.to_string();
                let app = app_clone.clone();
                std::thread::spawn(move ||{
                    info!("显示当前壁纸:{url}");
                    match image::open(Path::new(&url)){
                        Ok(file) => {
                            let img = file.to_rgba8();
                            let _ = app.upgrade_in_event_loop(move |app|{
                                let t = Instant::now();
                                let new_image = Image::from_rgba8(SharedPixelBuffer::clone_from_slice(
                                    &img,
                                    img.width(),
                                    img.height(),
                                ));
                                app.set_source_image(new_image);
                                info!("渲染图片耗时:{}ms", t.elapsed().as_millis());
                            });
                        }
                        Err(err) => {
                            info!("显示当前壁纸 读取失败:{:?}", err);
                        }
                    }
                });
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
    
    let _ = slint::spawn_local(update_config_ui(app.as_weak()));

    app.run().unwrap();
    info!("窗口关闭");
}