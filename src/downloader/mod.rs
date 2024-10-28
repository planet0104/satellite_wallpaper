use std::time::Instant;
use anyhow::Result;
use async_std::task::spawn_blocking;
use chrono::Timelike;
use image::{buffer::ConvertBuffer, imageops::resize, GenericImage, RgbImage};
use log::{info, error};
pub mod h8;
pub mod fy4x;

use crate::{app::{get_current_wallpaper, get_screen_size, get_wallpaper_file_path}, config, server::{set_download_status, DownloadStatus}};

pub fn format_time_str(download_name:&str, d: u32, year:i32, month:u8, day:u8, hour: u8, minute:u8) -> String{
    format!("{}-D{}-UTC-{}年-{}月-{}日-{}时-{:02}分", download_name, d, year, month, day, hour, (minute/15)*15)
}

async fn set_wallpaper<C:Fn(u32, u32) + 'static>(width: u32, height: u32, half: bool, callback: C) -> Result<()>{
    let mut cfg = config::load().await;
    //保存原有壁纸路径
    if cfg.old_wallpaper.len() == 0{
        if let Ok(old) = get_current_wallpaper(){
            cfg.old_wallpaper = old;
            config::save(cfg.clone()).await;
        }
    }
    info!("set_wallpaper>>准备下载 {width}x{height}...");
    //创建一张黑色背景图片
    let mut paper = RgbImage::new(width, height);
    let d = if height > 1080||half { 4 }else{ 2};
    let image = 
        match cfg.satellite_name.as_str(){
            "h8" => h8::download_lastest(&mut cfg, d, callback).await?,
            _ => fy4x::download_lastest(&mut cfg, d, callback).await?,
        };
    if image.is_none(){
        error!("set_wallpaper>>图片下载失败 {width}x{height} image.is_none()");
        return Ok(());
    }
    let mut image:RgbImage = image.unwrap().convert();

    //横屏模式
    let _image = if height < width || !half{
        // 图片稍微缩小一点
        let scale = if !half{
            (height as f32 * 0.9) / image.height() as f32
        }else{
            (width as f32 * 0.95) / image.width() as f32
        };
        let mut final_width = image.width() as f32 * scale;
        let mut final_height = image.height() as f32*scale;
        //不超过目标图片大小
        if final_width > paper.width() as f32{
            let s = paper.width() as f32 / final_width;
            final_width = final_width * s;
            final_height = final_height * s;
        }

        if final_height > paper.height() as f32{
            let s = paper.height() as f32 / final_height;
            final_width = final_width * s;
            final_height = final_height * s;
        }

        info!("set_wallpaper>>开始缩放 scale={scale} 目标大小:{final_width}x{final_height}...");
        let t = Instant::now();
        let mut image = fast_resize(&image, final_width as u32, final_height as u32).await;
        info!("set_wallpaper>>图片缩放成功 image:{}x{} paper:{}x{} half:{half} 耗时:{}ms", image.width(), image.height(), paper.width(), paper.height(), t.elapsed().as_millis());

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
        image
    }else{
        //竖屏: 地球直径取屏幕高度(%)，上午取地球右半部分，下午取地球左半部分
        
        use chrono::Local;
        let time = Local::now();
        let mut offset_x = 0;
        let mut left = true;
        if time.hour() <= 12 {
            //取地球右半部分
            let w = ((image.width() as f64 / 2.0)*1.06) as u32;
            let x = image.width() - w;
            image = image.sub_image(x, 0, w, image.height()).to_image();
        } else {
            //取地球左半部分
            let w = ((image.width() as f64 / 2.0)*1.06) as u32;
            image = image.sub_image(0, 0, w, image.height()).to_image();
            left = false;
        }

        //缩放，最大不超过屏幕高度
        let mut final_width = image.width() as f32 * 0.95;
        let mut final_height = image.height() as f32 * 0.95;
        //不超过目标图片大小
        if final_width > paper.width() as f32{
            let s = paper.width() as f32 / final_width;
            final_width = final_width * s;
            final_height = final_height * s;
        }

        if final_height > paper.height() as f32{
            let s = paper.height() as f32 / final_height;
            final_width = final_width * s;
            final_height = final_height * s;
        }

        info!("set_wallpaper>>开始缩放 目标大小:{final_width}x{final_height}...");
        let t = Instant::now();
        let image = fast_resize(&image, final_width as u32, final_height as u32).await;
        info!("set_wallpaper>>图片缩放成功 image:{}x{} paper:{}x{} half:{half} 耗时:{}ms", image.width(), image.height(), paper.width(), paper.height(), t.elapsed().as_millis());

        if !left{
            offset_x = (paper.width() - image.width()) as usize;
        }

        //拼接
        let offset_y = ((paper.height() - image.height()) / 2) as usize;
        let ew = image.width() as usize;
        for (y, buf) in image.chunks(ew * 3).enumerate() {
            if (y + offset_y) < paper.height() as usize {
                let offset = width as usize * 3 * (y + offset_y) + offset_x * 3;
                if let Some(s) = paper.get_mut(offset..offset + buf.len()) {
                    if s.len() == buf.len() {
                        s.copy_from_slice(buf);
                    }
                }
            } else {
                break;
            }
        }
        image
    };

    info!("set_wallpaper>>图片准备完成 paper:{}x{} half:{half}", paper.width(), paper.height());
    let wallpaper_file_path = get_wallpaper_file_path();
    info!("set_wallpaper>>wallpaper_file_path {wallpaper_file_path}");
    paper.save(&wallpaper_file_path)?;
    cfg.current_wallpaper_file = wallpaper_file_path.clone();
    info!("set_wallpaper>>保存配置文件...");
    config::save(cfg.clone()).await;

    // 设置锁屏
    let loc_res = super::app::set_lock_screen_image(&wallpaper_file_path);
    info!("锁屏设置结果: {:?}", loc_res);
    
    let loc_res = super::app::set_wallpaper_from_path(&wallpaper_file_path);
    info!("壁纸设置结果: {:?}", loc_res);
    loc_res
}

pub async fn set_wallpaper_default(){

    // 获取屏幕宽高
    let (screen_width, screen_height) = get_screen_size();

    set_download_status(DownloadStatus::new(true, &format!("开始下载壁纸 屏幕大小:{screen_width}x{screen_height}"))).await;

    let mut cfg = config::load().await;
    
    //下载最新壁纸
    if let Err(err) = set_wallpaper(screen_width as u32, screen_height as u32, cfg.display_type==2, |i,t|{
            info!("正在下载: {}/{}", i, t);
    }).await{
        error!("壁纸下载失败: {:?}", err);
        set_download_status(DownloadStatus::new(true, &format!("壁纸下载失败 {:?}", err))).await;
    }else{
        cfg.last_save_time = Some(current_time_str());
        config::save(cfg).await;
        set_download_status(DownloadStatus::new(true, "壁纸下载成功")).await;
    }
}

async fn fast_resize(src:&RgbImage, dst_width: u32, dst_height: u32) -> RgbImage{
    let src = src.clone();
    spawn_blocking(move ||{
        fast_resize_block(&src, dst_width, dst_height)
    }).await
}

fn fast_resize_block(src:&RgbImage, dst_width: u32, dst_height: u32) -> RgbImage{
    let mut dst_image = fast_image_resize::images::Image::new(
        dst_width,
        dst_height,
        fast_image_resize::PixelType::U8x3,
    );
    let mut src_image = fast_image_resize::images::Image::new(
        src.width(),
        src.height(),
        fast_image_resize::PixelType::U8x3,
    );
    src_image.buffer_mut().copy_from_slice(&src);
    let mut resizer = fast_image_resize::Resizer::new();
    let r = resizer.resize(&src_image, &mut dst_image, None);

    match r{
        Ok(_) => {
            if let Some(img) = RgbImage::from_raw(dst_image.width(), dst_image.height(), dst_image.buffer().to_vec()){
                return img;
            }
        }
        Err(err) => {
            error!("图片快速缩放失败:{:?}", err);
        }
    }
    resize(src, dst_width, dst_height, image::imageops::FilterType::Lanczos3)
}

pub fn current_time_str() -> String{
    chrono::Local::now().format("%Y/%m/%d %H:%M:%S").to_string()
}