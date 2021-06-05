use anyhow::Result;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{fs::{File, create_dir}, io::{Read, Write}, path::Path};

use crate::def::{APP_NAME_E, DEFAULT_DOWNLOAD_URL_FY4A, DEFAULT_DOWNLOAD_URL_H8, DEFAULT_SERVER_PORT};

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
    pub download_url_fy4a: String,

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

    /// 配置文件路径
    pub config_path: String,
}

pub fn load() -> Config {
    match read_config() {
        Ok(cfg) => cfg,
        Err(err) => {
            error!("配置文件读取失败: {:?}", err);
            Config {
                update_interval: 10,
                display_type: 1,
                server_port: DEFAULT_SERVER_PORT,
                download_url_h8: DEFAULT_DOWNLOAD_URL_H8.to_string(),
                download_url_fy4a: DEFAULT_DOWNLOAD_URL_FY4A.to_string(),
                old_wallpaper: String::new(),
                current_wallpaper_date: String::new(),
                current_wallpaper_file: String::new(),
                config_path: String::new(),
                satellite_name: String::from("h8"),
                current_wallpaper_thumbnail: None,
            }
        }
    }
}

pub fn save(cfg: &mut Config) {
    if let Err(err) = write_config(cfg) {
        error!("配置保存失败: {:?}", err)
    }else{
        // info!("配置保存成功: {:?}", cfg);
        info!("配置保存成功");
    }
}

fn get_config_file_path() -> String {
    let cfg_file_name = &format!("{}.toml", APP_NAME_E);
    let mut cfg_path_name = cfg_file_name.to_string();
    if let Some(cfg_dir) = dirs::config_dir(){
        if let Some(cfg_dir) = cfg_dir.to_str(){
            let my_cfg_dir = format!("{}\\{}", cfg_dir, APP_NAME_E);
            if Path::exists(Path::new(&my_cfg_dir)){
                cfg_path_name = format!("{}\\{}", my_cfg_dir, cfg_file_name);
            }else{
                if let Ok(()) = create_dir(&my_cfg_dir){
                    cfg_path_name = format!("{}\\{}", my_cfg_dir, cfg_file_name);
                }
            }
        }
    }
    // info!("config {:?}", cfg_path_name);
    cfg_path_name
}

fn read_config() -> Result<Config> {
    let cfg_path = get_config_file_path();
    let mut config_file = File::open(cfg_path)?;
    let mut config_str = String::new();
    config_file.read_to_string(&mut config_str)?;
    Ok(toml::from_str(&config_str)?)
}

fn write_config(cfg: &mut Config) -> Result<()> {
    let cfg_path = get_config_file_path();
    cfg.config_path = cfg_path.clone();
    let mut config_file = File::create(cfg_path)?;
    let cfg_str = toml::to_string(cfg)?;
    config_file.write_all(cfg_str.as_bytes())?;
    Ok(())
}
