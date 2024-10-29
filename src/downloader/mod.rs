use std::time::Instant;
use anyhow::{anyhow, Result};
use async_std::task::spawn_blocking;
use chrono::{Local, Timelike};
use image::{buffer::ConvertBuffer, imageops::resize, GenericImage, RgbImage, RgbaImage};
use log::{error, info, warn};
pub mod h8;
pub mod fy4x;

use crate::{app::{get_current_wallpaper, get_screen_size, get_wallpaper_file_path}, config::Config};

static DOWNLOADING: std::sync::RwLock<bool> = std::sync::RwLock::new(false);

pub fn is_downlading() -> bool{
    match DOWNLOADING.read() {
        Ok(v) =>*v,
        Err(_) => false,
    }
}

pub fn set_downlading(d: bool){
    match DOWNLOADING.write() {
        Ok(mut v) => *v = d,
        Err(_) => ()
    }
}

pub fn format_time_str(download_name:&str, d: u32, year:i32, month:u8, day:u8, hour: u8, minute:u8) -> String{
    format!("{}-D{}-UTC-{}年-{}月-{}日-{}时-{:02}分", download_name, d, year, month, day, hour, (minute/15)*15)
}

fn set_wallpaper<C:Fn(u32, u32) + 'static>(cfg:&mut Config, width: u32, height: u32, half: bool, callback: C) -> Result<()>{
    //保存原有壁纸路径
    if cfg.old_wallpaper.len() == 0{
        if let Ok(old) = get_current_wallpaper(){
            cfg.old_wallpaper = old;
        }
    }
    info!("set_wallpaper>>准备下载 {width}x{height}...");
    //创建一张黑色背景图片
    let mut paper = RgbImage::new(width, height);
    let d = if height > 1080||half { 4 }else{ 2};
    let image = 
        match cfg.satellite_name.as_str(){
            "h8" => h8::download_lastest(cfg, d, callback)?,
            _ => fy4x::download_lastest(cfg, d, callback)?,
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
        let mut image = fast_resize(&image, final_width as u32, final_height as u32);
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
        let image = fast_resize(&image, final_width as u32, final_height as u32);
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
    // 设置锁屏

    info!("开始调用set_lock_screen_image>>>>>>>>>>>>");

    let loc_res = super::app::set_lock_screen_image(&wallpaper_file_path);
    info!("锁屏设置结果: {:?}", loc_res);
    
    info!("开始调用set_wallpaper_from_path>>>>>>>>>>>>");
    let loc_res = super::app::set_wallpaper_from_path(&wallpaper_file_path);
    info!("壁纸设置结果: {:?}", loc_res);
    loc_res
    // warn!("未设置安卓壁纸，直接返回，看看是否会崩溃！");
    // Ok(())
}

pub async fn set_wallpaper_default(cfg: &mut Config){
    if is_downlading(){
        info!("壁纸正在下载中, 请稍后..");
        return;
    }
    // 获取屏幕宽高
    let (screen_width, screen_height) = get_screen_size();

    set_downlading(true);

    let display_type = cfg.display_type;
    let cfg_clone = cfg.clone();

    info!("调用 set_wallpaper >> step 001");
    let ret = spawn_blocking(move ||{
        let mut cfg = cfg_clone;
        info!("调用 set_wallpaper >> step 002");
        let ret = set_wallpaper(&mut cfg, screen_width as u32, screen_height as u32, display_type==2, |i,t|{
                info!("正在下载: {}/{}", i, t);
        });
        info!("调用 set_wallpaper >> step 003");
        (cfg, ret)
    }).await;
    info!("调用 set_wallpaper >> step 004");
    let (mut cfg, ret) = ret;
    //下载最新壁纸
    if let Err(err) = ret{
        error!("壁纸下载失败: {:?}", err);
    }else{
        cfg.last_download_timestamp = Some(Local::now().timestamp_millis());
        let _ = cfg.save_to_file().await;
    }
    info!("调用 set_wallpaper >> step 005");
    set_downlading(false);
    info!("下载结束....");
}

fn fast_resize(src:&RgbImage, dst_width: u32, dst_height: u32) -> RgbImage{
    let src = src.clone();
    fast_resize_block(&src, dst_width, dst_height)
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

//取一张图片
pub fn download_image(url: &str) -> Result<RgbaImage> {
    info!("download_image {}", url);
    let url = url.to_string();
    download_image_sync(&url)
}

fn download_image_sync(url: &str) -> Result<RgbaImage> {
    info!("download_image {}", url);

    
    //  GET /swapQuery/public/tileServer/getTile/fy-4b/full_disk/NatureColor_NoLit/20241029144500/jpg/2/2/3.png HTTP/1.1
    //     Host: rsapp.nsmc.org.cn
    //     User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:131.0) Gecko/20100101 Firefox/131.0
    //     Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/png,image/svg+xml,*/*;q=0.8
    //     Accept-Language: zh-CN,zh;q=0.8,zh-TW;q=0.7,zh-HK;q=0.5,en-US;q=0.3,en;q=0.2
    //     Accept-Encoding: gzip, deflate
    //     Connection: keep-alive
    //     Upgrade-Insecure-Requests: 1
    //     Priority: u=0, i 
     
     let req = minreq::get(url)
     .with_timeout(10);
    let response = 
    req.send()?;
    let image_data = response.as_bytes();
    info!("{url} \n 下载字节长度:{} headers:{:?}", image_data.len(), response.headers);
    if response.headers.contains_key("content-type"){
        if response.headers.get("content-type").unwrap() == "text/html"{
            info!("内容:{:?}", String::from_utf8(image_data.to_vec()));
        }
    }
    let img = image::load_from_memory(image_data)?.to_rgba8();
    info!("download_image {} OK:{}x{}", url, img.width(), img.height());
    Ok(img)
}