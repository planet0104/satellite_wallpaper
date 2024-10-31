use std::time::Instant;
use anyhow::{anyhow, Result};
use image::{GenericImage, ImageBuffer, Rgba, RgbaImage};
use log::{error, info, warn};
use time::OffsetDateTime;

use crate::{config::Config, downloader::{download_image, format_time_str}};

/// 下载4x4、2x2的图，最终大小: 1100x1100 、2200x2200
pub fn download<C>(
    url: &str,
    d: u32,
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    ten_minute: u8,
    callback: C,
) -> Result<RgbaImage>
where
    C: Fn(u32, u32) + 'static,
{
    /*
    4d的图排列如下:
    00,10,20,30
    01,11,21,31
    02,12,22,32
    03,13,23,33
    2d的图排列如下:
    00,10
    01,11
     */
    let mut count = 0;
    let total = d*d;
    let t = Instant::now();
    info!("开始下载图片 共{total}张...");
    let (tx, rx) = std::sync::mpsc::channel();

    for y in 0..d{
        for x in 0..d{
            let url1 = url.to_string();
            let tx1 = tx.clone();
            std::thread::spawn(move ||{
                let ret = download_image(&format_url(&url1, year, month, day, hour, ten_minute / 10, d, x, y));
                let _ = tx1.send((count, ret));
            });
            count += 1;
        }
    }

    let mut images = vec![None; total as usize];

    let mut count = 0;
    for _ in 0..total{
        let r = rx.recv();
        if r.is_err(){
            error!("图片下载超时:{:?}", r.err());
            break;
        }
        let (i, img) = r.unwrap();
        if img.is_err(){
            error!("图片下载失败:{:?}", img.err());
            break;
        }
        images[i as usize] = Some(img?);
        count += 1;
        callback(count, total);
    }
    
    for (i, img) in images.iter().enumerate(){
        if img.is_none(){
            return Err(anyhow!("{i}号图片下载失败!"));
        }
    }
    
    let mut images:Vec<ImageBuffer<Rgba<u8>, Vec<u8>>> = images.into_iter().map(|v| v.unwrap()).collect();

    info!("图片下载完成 共{}张. 耗时:{}ms", images.len(), t.elapsed().as_millis());

    let (width, height) = (images[0].width(), images[0].height());
    let mut big_img = RgbaImage::new(width*d, height*d);
    for y in 0..d{
        for x in 0..d{
            let img = images.remove(0);
            big_img.sub_image(x*width, y*height, img.width(), img.height()).copy_from(&img, 0, 0)?;
        }
    }
    Ok(big_img)
}

fn format_url(
    url: &str,
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    ten_minute: u8,
    d: u32,
    x: u32,
    y: u32,
) -> String {
    format!("{}D531106/{}d/550/{}/{:02}/{:02}/{:02}{}000_{}_{}.png", url, d, year, month, day, hour, ten_minute/10, x, y)
}

/// 下载最新图片, 20分钟之前
pub fn download_lastest<C:Fn(u32, u32) + 'static>(cfg: &Config, d:u32, callback:C ) -> Result<Option<(String, RgbaImage)>>{
    let mut timestamp = OffsetDateTime::now_utc().unix_timestamp();
    //减去20分钟
    timestamp -= 20 * 60 * 1000;
    let utc = OffsetDateTime::from_unix_timestamp(timestamp)?;
    let timestr = format_time_str(&cfg.satellite_name, d, utc.year(), utc.month() as u8, utc.day(), utc.hour(), utc.minute());
    info!("时间:{}", timestr);
    if cfg.current_wallpaper_date == timestr{
        warn!("壁纸无需重复下载");
        return Ok(None);
    }
    let img = download(&cfg.download_url_h8, d, utc.year(), utc.month() as u8, utc.day(), utc.hour(), utc.minute(), callback)?;
    Ok(Some((timestr, img)))
}