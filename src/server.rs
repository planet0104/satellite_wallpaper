use std::thread;
use std::time::Duration;
use std::time::Instant;
use log::info;
use crate::downloader::is_downloading;
use crate::downloader::set_wallpaper_default;
use crate::config;

pub fn start_async(){
    start_update_loop();
}

/// 定时更新壁纸线程
pub fn start_update_loop(){
    let _ = thread::spawn(||{
        let mut instant:Option<Instant> = None;
        let sleep_dur = Duration::from_secs(60);
        loop{
            let cfg = config::load();
            if instant.is_none() || instant.as_ref().unwrap().elapsed() >  Duration::from_secs(cfg.update_interval as u64*60){
                info!("thread :时间到");
                if !is_downloading(){
                    set_wallpaper_default();
                    instant = Some(Instant::now());
                }else{
                    info!("thread :is_downloading 不下载.")
                }
            }
            thread::sleep(sleep_dur);
        }
    });
}