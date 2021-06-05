use std::{fs::{File, create_dir}, io::Read, path::Path, sync::Mutex, thread};
use anyhow::{anyhow, Result};
use image::{GenericImage, Rgba, RgbaImage, imageops::resize};
use log::{info, error};
use once_cell::sync::Lazy;
pub mod h8;
pub mod fy4a;

use crate::{app::{self, get_screen_size}, config, def::APP_NAME_E};

static IS_DOWNLOADING: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

pub fn format_time_str(download_name:&str, d: u32, year:i32, month:u32, day:u32, hour: u32, minute:u32) -> String{
    format!("{}-D{}-UTC-{}年-{}月-{}日-{}时-{:02}分", download_name, d, year, month, day, hour, (minute/15)*15)
}

fn set_wallpaper<C:Fn(u32, u32) + 'static>(width: u32, height: u32, half: bool, callback: C) -> Result<()>{
    let mut cfg = config::load();
    //保存原有壁纸路径
    if cfg.old_wallpaper.len() == 0{
        if let Ok(old) = wallpaper::get(){
            cfg.old_wallpaper = old;
            config::save(&mut cfg);
        }
    }
    //创建一张黑色背景图片
    let mut paper = RgbaImage::new(width, height);
    paper.pixels_mut().for_each(|p| *p = Rgba([0, 0, 0, 255]));
    let d = if height > 1080||half { 4 }else{ 2};
    let image = 
        match cfg.satellite_name.as_str(){
            "h8" => h8::download_lastest(&mut cfg, d, callback)?,
            _ => fy4a::download_lastest(&mut cfg, d, callback)?,
        };
    if image.is_none(){
        return Ok(());
    }
    let image = image.unwrap();
    // image.save("test.png").unwrap();
    // 图片稍微缩小一点
    let scale = if !half{
        (height as f32 * 0.9) / image.height() as f32
    }else{
        (width as f32 * 0.95) / image.width() as f32
    };
    let mut image = resize(&image, (image.width() as f32 * scale) as u32, (image.height() as f32*scale) as u32, image::imageops::FilterType::Lanczos3);
    
    // 复制到桌面背景中
    if half{
        //复制上半块
        let x = (paper.width()-image.width())/2;
        let y = (paper.height() as f32 * 0.05) as u32;
        //要复制的高度
        let ch = paper.height()-y;
        //要复制的图像
        image = image.sub_image(0, 0, image.width(), ch).to_image();
        paper.sub_image(x, y, image.width(), image.height()).copy_from(&image, 0, 0)?;
    }else{
        let x = (paper.width()-image.width())/2;
        let y = (paper.height()-image.height())/2;
        paper.sub_image(x, y, image.width(), image.height()).copy_from(&image, 0, 0)?;
    }

    let wallpaper_file_path = get_wallpaper_file_path();
    paper.save(&wallpaper_file_path)?;
    cfg.current_wallpaper_file = wallpaper_file_path.clone();

    // 生成略缩图
    let thumb_scale = 500. / image.width() as f32;
    let (thumb_width, thumb_height) = (image.width() as f32 * thumb_scale, image.height() as f32 * thumb_scale);
    let thumb = resize(&image, thumb_width as u32, thumb_height as u32, image::imageops::FilterType::Triangle);
    let thumb_path = format!("{}\\thumbnail.png", get_app_home_dir());
    if let Ok(_) = thumb.save(&thumb_path){
        if let Ok(mut f) = File::open(thumb_path){
            let mut png_data = vec![];
            let _ = f.read_to_end(&mut png_data);
            if png_data.len() > 0{
                cfg.current_wallpaper_thumbnail = Some(base64::encode(&png_data));
            }
        }
    }

    config::save(&mut cfg);

    // 设置锁屏
    let loc_res = app::set_lock_screen_image(&wallpaper_file_path);
    info!("锁屏设置结果: {:?}", loc_res);
    
    // 设置壁纸
    match wallpaper::set_from_path(&wallpaper_file_path){
        Ok(()) => Ok(()),
        Err(err) => {
            Err(anyhow!("{:?}", err))
        }
    }
}

pub fn set_wallpaper_default(){
    {
        if let Ok(mut d) = IS_DOWNLOADING.lock(){
            *d = true;
        }
    }
    let cfg = config::load();
    // 获取屏幕宽高
    let (screen_width, screen_height) = get_screen_size();
    //下载最新壁纸
    if let Err(err) = set_wallpaper(screen_width as u32, screen_height as u32, cfg.display_type==2, |i,t|{
            info!("正在下载: {}/{}", i, t);
    }){
        error!("壁纸下载失败: {:?}", err);
    }
    {
        if let Ok(mut d) = IS_DOWNLOADING.lock(){
            *d = false;
        }
    }
}

pub fn is_downloading() -> bool{
    if let Ok(d) = IS_DOWNLOADING.lock(){
        *d
    }else{
        false
    }
}

pub fn set_wallpaper_default_async(){
    thread::spawn(move ||{
        set_wallpaper_default();
    });
}

fn get_wallpaper_file_path() -> String {
    let wallpaper_path_name = format!( "{}\\wallpaper.png", get_app_home_dir());
    info!("wallpaper {:?}", wallpaper_path_name);
    wallpaper_path_name
}

fn get_app_home_dir() -> String {
    let mut app_home_dir = String::from(".");
    if let Some(home_dir) = dirs::home_dir(){
        if let Some(home_dir) = home_dir.to_str(){
            let app_home_dir_tmp = format!("{}\\{}", home_dir, APP_NAME_E);
            if Path::exists(Path::new(&app_home_dir_tmp)){
                app_home_dir = app_home_dir_tmp;
            }else{
                if let Ok(()) = create_dir(&app_home_dir_tmp){
                    app_home_dir = app_home_dir_tmp;
                }
            }
        }
    }
    info!("app_home_dir {}", app_home_dir);
    app_home_dir
}