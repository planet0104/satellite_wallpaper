use anyhow::Result;
use async_std::fs::create_dir;
use chrono::{DateTime, Local};
use log::info;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{app::get_config_dir, def::{APP_NAME_E, DEFAULT_DOWNLOAD_URL_FY4B, DEFAULT_DOWNLOAD_URL_H8, DEFAULT_SERVER_PORT}};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// 壁纸更新时间间隔(分钟)
    pub update_interval: u32,

    /// 壁纸显示样式 1:整张 2:半张
    pub display_type: u32,

    /// 服务器端口号
    pub server_port: u32,

    /// h8卫星图片下载地址
    pub download_url_h8: String,
    /// 风云4号卫星图片下载地址
    pub download_url_fy4b: String,

    /// 下载的卫星名字
    pub satellite_name: String,

    /// 旧的桌面壁纸
    pub old_wallpaper: String,

    /// 当前壁纸日期
    pub current_wallpaper_date: String,

    /// 当前壁纸路径
    pub current_wallpaper_file: String,

    /// 最后一次保存壁纸壁纸
    pub last_download_timestamp: Option<i64>,

    /// 配置文件路径
    pub config_path: String,
}

impl Default for Config{
    fn default() -> Self {
        Self {
            update_interval: 10,
            display_type: 1,
            server_port: DEFAULT_SERVER_PORT,
            download_url_h8: DEFAULT_DOWNLOAD_URL_H8.to_string(),
            download_url_fy4b: DEFAULT_DOWNLOAD_URL_FY4B.to_string(),
            old_wallpaper: String::new(),
            current_wallpaper_date: String::new(),
            current_wallpaper_file: String::new(),
            config_path: String::new(),
            satellite_name: String::from("fy4b"),
            last_download_timestamp: None
        }
    }
}

impl Config{
    pub fn get_last_update_time_str(&self) -> String{
        if self.last_download_timestamp.is_none(){
            return "无".to_string();
        }
        if let Some(d) = DateTime::from_timestamp_millis(self.last_download_timestamp.unwrap()){
            let local_datetime: DateTime<Local> = DateTime::<Local>::from(d);
            local_datetime.format("%Y/%m/%d %H:%M:%S").to_string()
        }else{
            "无".to_string()
        }
    }
    
    pub async fn save_to_file(&mut self) -> Result<()> {
        let cfg_path = get_config_file_path().await;
        self.config_path = cfg_path.clone();
        let cfg_str: String = toml::to_string(&self)?;
        info!("写入文件:{cfg_path}");
        let mut config_file = async_std::fs::File::create(cfg_path).await?;
        async_std::io::WriteExt::write_all(&mut config_file, cfg_str.as_bytes()).await?;
        info!("配置文件保存成功 {cfg_str}");
        Ok(())
    }

    pub async fn load_from_file(&mut self) -> Result<()>{
        let cfg_path = get_config_file_path().await;
        info!("读取文件:{cfg_path}");
        let mut config_file = async_std::fs::File::open(cfg_path).await?;
        let mut config_str = String::new();
        async_std::io::ReadExt::read_to_string(&mut config_file, &mut config_str).await?;
        *self = toml::from_str(&config_str)?;
        info!("配置文件读取成功:{config_str}");
        Ok(())
    }

    pub async fn load_or_default() -> Config{
        let mut cfg = Config::default();
        let _ = cfg.load_from_file().await;
        cfg
    }
}

async fn get_config_file_path() -> String {
    let cfg_file_name = &format!("{}{}.toml", APP_NAME_E, env!("CARGO_PKG_VERSION"));
    let mut cfg_path_name = cfg_file_name.to_string();
    let cfg_dir = get_config_dir();
    #[cfg(windows)]
    let sp = "\\";
    #[cfg(not(windows))]
    let sp = "/";
    let my_cfg_dir = format!("{}{sp}{}", cfg_dir, APP_NAME_E);
    if Path::exists(Path::new(&my_cfg_dir)){
        cfg_path_name = format!("{}{sp}{}", my_cfg_dir, cfg_file_name);
    }else{
        if let Ok(()) = create_dir(&my_cfg_dir).await{
            cfg_path_name = format!("{}{sp}{}", my_cfg_dir, cfg_file_name);
        }
    }
    // info!("config {:?}", cfg_path_name);
    cfg_path_name
}