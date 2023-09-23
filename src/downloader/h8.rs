use anyhow::Result;
use image::{GenericImage, RgbaImage};
use log::info;
use time::OffsetDateTime;

use crate::{config::{self, Config}, downloader::format_time_str};

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
    let mut images = vec![];
    let mut count = 0;
    let total = d*d;
    for y in 0..d{
        for x in 0..d{
            count += 1;
            callback(count, total);
            images.push(download_image(&format_url(url, year, month, day, hour, ten_minute / 10, d, x, y))?);
        }
    }
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

//取一张图片
fn download_image(url: &str) -> Result<RgbaImage> {
    info!("download_image {}", url);
    let response = minreq::get(url).send()?;
    let image_data = response.as_bytes();
    let img = image::load_from_memory(image_data)?.to_rgba8();
    Ok(img)
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
pub fn download_lastest<C:Fn(u32, u32) + 'static>(cfg: &mut Config, d:u32, callback:C ) -> Result<Option<RgbaImage>>{
    let mut timestamp = OffsetDateTime::now_utc().unix_timestamp();
    //减去20分钟
    timestamp -= 20 * 60 * 1000;
    let utc = OffsetDateTime::from_unix_timestamp(timestamp)?;
    let timestr = format_time_str(&cfg.satellite_name, d, utc.year(), utc.month() as u8, utc.day(), utc.hour(), utc.minute());
    info!("时间:{}", timestr);
    if cfg.current_wallpaper_date == timestr{
        info!("壁纸无需重复下载");
        return Ok(None);
    }
    cfg.current_wallpaper_date = timestr;
    config::save(cfg);
    Ok(Some(download(&cfg.download_url_h8, d, utc.year(), utc.month() as u8, utc.day(), utc.hour(), utc.minute(), callback)?))
}