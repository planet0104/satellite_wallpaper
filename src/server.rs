use std::time::Duration;
use std::time::Instant;
use async_std::task::sleep;
use log::info;
use crate::downloader::is_downloading;
use crate::downloader::set_wallpaper_default;
use crate::config;

/// 定时更新壁纸线程
pub async fn start_update_loop(){
    let mut instant:Option<Instant> = None;
    let sleep_dur = Duration::from_secs(60);
    loop{
        info!("更新任务进行中...");
        let cfg = config::load().await;
        if instant.is_none() || instant.as_ref().unwrap().elapsed() >  Duration::from_secs(cfg.update_interval as u64*60){
            info!("thread :时间到");
            if !is_downloading().await{
                set_wallpaper_default().await;
                instant = Some(Instant::now());
            }else{
                info!("thread :is_downloading 不下载.")
            }
        }
        sleep(sleep_dur).await
    }
}