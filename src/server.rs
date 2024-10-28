use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;
use async_std::task::sleep;
use async_std::task::spawn_blocking;
use log::error;
use log::info;
use once_cell::sync::Lazy;
use crate::downloader::current_time_str;
use crate::downloader::set_wallpaper_default;
use crate::config;

#[derive(Clone, Debug)]
pub struct DownloadStatus{
    pub downloading: bool,
    pub status: Option<String>
}

impl DownloadStatus{
    pub fn new(downloading: bool, status: &str) -> DownloadStatus{
        DownloadStatus{
            downloading,
            status: Some(format!("{} {status}", current_time_str()))
        }
    }

    pub fn status(&self) -> String{
        self.status.clone().unwrap_or(String::from("未启动"))
    }
}

pub static DOWNLOAD_STATUS: Lazy<Arc<Mutex<DownloadStatus>>> =  Lazy::new(||{
    Arc::new(Mutex::new(DownloadStatus{
        downloading: false,
        status: None,
    }))
});

pub async fn get_download_status() -> DownloadStatus{
    spawn_blocking(move ||{
        match DOWNLOAD_STATUS.lock(){
            Ok(status) => {
                info!("获取了download_status: {:?}", *status);
                status.clone()
            }
            Err(err) => {
                error!("get_download_status DOWNLOAD_STATUS锁定失败:{:?}", err);
                DownloadStatus::new(false, "状态查询失败")
            }
        }
    }).await
}

pub async fn set_download_status(status: DownloadStatus){
    spawn_blocking(move ||{
        match DOWNLOAD_STATUS.lock(){
            Ok(mut v) => {
                *v = status;
                info!("设置了download_status: {:?}", *v);
            }
            Err(err) => {
                error!("set_download_status DOWNLOAD_STATUS锁定失败:{:?}", err);
            }
        }
    }).await
}

/// 定时更新壁纸线程
pub async fn start_update_loop(){
    let mut instant:Option<Instant> = None;
    let sleep_dur = Duration::from_secs(5);
    loop{
        info!(">>>> update_loop.............");
        let cfg = config::load().await;
        if instant.is_none() || instant.as_ref().unwrap().elapsed() >  Duration::from_secs(cfg.update_interval as u64*60){
            info!("thread :时间到 开始下载壁纸...");
            let status = get_download_status().await;
            if !status.downloading{
                set_wallpaper_default().await;
                instant = Some(Instant::now());
            }else{
                info!("thread :is_downloading 不下载.")
            }
        }else{
            let last_update_time = match cfg.last_save_time{
                Some(i) => {
                    format!("上次更新: {i}")
                }
                None => "上次更新: 无".to_string(),
            };
            info!(">>>> update_loop.............last_update_time = {last_update_time}");
            set_download_status(DownloadStatus::new(false, &last_update_time)).await;
        }
        sleep(sleep_dur).await
    }
}