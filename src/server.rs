use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;
use async_std::task::sleep;
use log::info;
use crate::config::Config;
use crate::downloader::is_downlading;
use crate::downloader::set_wallpaper_default;

/// 定时更新壁纸线程
pub async fn start_update_loop(is_exit: Arc<Mutex<bool>>){
    let mut instant:Option<Instant> = None;
    let sleep_dur = Duration::from_secs(30);
    loop{
        if *is_exit.lock().unwrap(){
            break;
        }
        info!(">>>> update_loop.............");
        let mut cfg = Config::load_or_default().await;
        // let last_update_time = match cfg.last_save_time{
        //     Some(i) => {
        //         format!("上次更新: {i}")
        //     }
        //     None => "上次更新: 无".to_string(),
        // };
        if instant.is_none() || instant.as_ref().unwrap().elapsed() >  Duration::from_secs(cfg.update_interval as u64*60){
            info!("thread :时间到 开始下载壁纸...");
            if !is_downlading(){
                set_wallpaper_default(&mut cfg).await;
                instant = Some(Instant::now());
            }else{
                info!("thread :is_downloading 不下载.")
            }
        }
        sleep(sleep_dur).await
    }
    info!("程序结束，退出任务...");
}