use anyhow::Result;
use async_std::fs::create_dir;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{app::get_config_dir, def::{APP_NAME_E, DEFAULT_DOWNLOAD_URL_FY4B, DEFAULT_DOWNLOAD_URL_H8, DEFAULT_SERVER_PORT}};

#[derive(Clone, Serialize, Deserialize)]
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

    /// 当前壁纸略缩图
    pub current_wallpaper_thumbnail: Option<String>,
    /// 最后一次保存壁纸壁纸
    pub last_save_time: Option<String>,

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
            current_wallpaper_thumbnail: None,
            last_save_time: None
        }
    }
}

pub async fn load() -> Config {
    match read_config().await {
        Ok(cfg) => cfg,
        Err(err) => {
            error!("配置文件读取失败: {:?}", err);
            Config::default()   
        }
    }
}

pub async fn save(cfg: Config) {
    if let Err(err) = write_config(cfg).await {
        error!("配置保存失败: {:?}", err)
    }else{
        // info!("配置保存成功: {:?}", cfg);
        info!("配置保存成功");
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

async fn read_config() -> Result<Config> {
    let cfg_path = get_config_file_path().await;
    let mut config_file = async_std::fs::File::open(cfg_path).await?;
    let mut config_str = String::new();
    async_std::io::ReadExt::read_to_string(&mut config_file, &mut config_str).await?;
    Ok(toml::from_str(&config_str)?)
}

async fn write_config(mut cfg: Config) -> Result<()> {
    let cfg_path = get_config_file_path().await;
    cfg.config_path = cfg_path.clone();
    let cfg_str = toml::to_string(&cfg)?;
    let mut config_file = async_std::fs::File::create(cfg_path).await?;
    async_std::io::WriteExt::write_all(&mut config_file, cfg_str.as_bytes()).await?;
    Ok(())
}