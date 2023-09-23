use std::{ops::Sub, time::Duration};

use anyhow::Result;
use image::{GenericImage, RgbaImage};
use log::info;
use time::{OffsetDateTime, Date};

use crate::{config::{self, Config}, downloader::format_time_str};

//http://rsapp.nsmc.org.cn/geofy/


/// 下载4x4、2x2的图
pub fn download<C>(
    url: &str,
    d: u32,
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    callback: C,
) -> Result<RgbaImage>
where
    C: Fn(u32, u32) + 'static,
{
    let mut images = vec![];
    let mut count = 0;
    let total = d*d;
    for y in 0..d{
        for x in 0..d{
            count += 1;
            callback(count, total);
            images.push(download_image(&format_url(url, year, month, day, hour, minute, d/2, x, y))?);
        }
    }
    let (width, height) = (images[0].width(), images[0].height());
    let mut big_img = RgbaImage::new(width*d, height*d);
    for x in 0..d{
        for y in 0..d{
            let img = images.remove(0);
            big_img.sub_image(x*width, y*height, img.width(), img.height()).copy_from(&img, 0, 0)?;
        }
    }
    Ok(big_img)
}

//取一张图片
pub fn download_image(url: &str) -> Result<RgbaImage> {
    info!("download_image {}", url);
    let response = minreq::get(url).send()?;
    let image_data = response.as_bytes();
    let img = image::load_from_memory(image_data)?.to_rgba8();
    Ok(img)
}

// d 1代表4张图, 2代表16张图
pub fn format_url(
    url: &str,
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    d: u32,
    x: u32,
    y: u32,
) -> String {
    format!("{}{}{:02}{:02}{:02}{:02}00/jpg/{}/{}/{}.png", url, year, month, day, hour, minute, d, x, y)
    // 20210530054500/jpg/2/2/0.png
    // 20210530070000/jpg/1/1/0.png 5月30日15点00分
    // 20210530073000/jpg/1/1/0.png 5月30日15点15分
}

/// 下载最新图片, 20分钟之前
pub fn download_lastest<C:Fn(u32, u32) + 'static>(cfg: &mut Config, d:u32, callback:C ) -> Result<Option<RgbaImage>>{
    
    // 从当前时间以15分钟倒推，查询最后可下载的图片
    let now = OffsetDateTime::now_utc();
    let today = Date::from_calendar_date(now.year(), now.month(), now.day())?;
    let hour = today.with_hms(now.hour(), (now.minute()/15)*15, 0)?;

    let mut time = hour.sub(Duration::from_secs(60*15));
    // let mut time = hour.sub();

    let mut try_times = 0;
    while try_times < 4{
        //尝试下载最新一张图片, 递减15分钟
        if download_image(&format_url(
            &cfg.download_url_fy4a, time.year(), time.month() as u8, time.day(), time.hour(), time.minute(), 1, 0, 0)).is_err(){
                info!("卫星图片不存在，尝试下载更早的图片.");
                time = time.sub(Duration::from_secs(60*15));
                try_times += 1;
        }else{
            break;
        }
    }
    let timestr = format_time_str(&cfg.satellite_name, d, time.year(), time.month() as u8, time.day(), time.hour(), time.minute());
    info!("时间:{}", timestr);
    if cfg.current_wallpaper_date == timestr{
        info!("壁纸无需重复下载");
        return Ok(None);
    }
    cfg.current_wallpaper_date = timestr;
    config::save(cfg);
    Ok(Some(download(&cfg.download_url_fy4a, d, time.year(), time.month() as u8, time.day(), time.hour(), time.minute(), callback)?))
}