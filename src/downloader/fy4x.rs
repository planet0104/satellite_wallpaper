use std::{ops::Sub, time::{Duration, Instant}};
use anyhow::{anyhow, Result};
use image::{GenericImage, ImageBuffer, Rgba, RgbaImage};
use log::{error, info, warn};
use time::{OffsetDateTime, Date};

use crate::{config::Config, downloader::{download_image, download_image_sync, format_time_str}};

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
    let total = d*d;
    info!("开始下载图片 共{total}张...");
    let t = Instant::now();
    let (tx, rx) = std::sync::mpsc::channel();
    let mut count = 0;
    for y in 0..d{
        for x in 0..d{
            let url1 = url.to_string();
            let tx1 = tx.clone();
            std::thread::spawn(move ||{
                let ret = download_image(&format_url(&url1, year, month, day, hour, minute, d/2, x, y));
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
            error!("图片下载失败:{:?}", r.err());
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
    let t = Instant::now();
    let (width, height) = (images[0].width(), images[0].height());
    let mut big_img = RgbaImage::new(width*d, height*d);
    for x in 0..d{
        for y in 0..d{
            let img = images.remove(0);
            big_img.sub_image(x*width, y*height, img.width(), img.height()).copy_from(&img, 0, 0)?;
        }
    }
    info!("图片合并完成 {}x{}. 耗时:{}ms", big_img.width(), big_img.height(), t.elapsed().as_millis());
    Ok(big_img)
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
        let ret = download_image(&format_url(&cfg.download_url_fy4b, time.year(), time.month() as u8, time.day(), time.hour(), time.minute(), 1, 0, 0));
        if ret.is_err(){
            log::error!("download_image失败: {:?}", ret.err());
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
        warn!("壁纸无需重复下载");
        return Ok(None);
    }
    cfg.current_wallpaper_date = timestr;
    Ok(Some(download(&cfg.download_url_fy4b, d, time.year(), time.month() as u8, time.day(), time.hour(), time.minute(), callback)?))
}