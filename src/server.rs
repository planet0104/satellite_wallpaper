use std::io;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use base64::alphabet;
use base64::engine;
use base64::engine::GeneralPurposeConfig;
use image::ColorType;
use image::EncodableLayout;
use image::ImageEncoder;
use image::Rgba;
use image::codecs::png::PngEncoder;
use log::error;
use log::info;
use qrcode_rs::QrCode;
use tiny_http::Method;

use crate::app;
use crate::config::Config;
use crate::downloader;
use crate::downloader::is_downloading;
use crate::downloader::set_wallpaper_default;
use crate::downloader::set_wallpaper_default_async;
use crate::{config, def::*};

pub fn start_async(){
    start_update_loop();
    start_http_server();
}

/// 定时更新壁纸线程
pub fn start_update_loop(){
    let _ = thread::spawn(||{
        let mut instant:Option<Instant> = None;
        let sleep_dur = Duration::from_secs(60);
        loop{
            let cfg = config::load();
            if instant.is_none() || instant.as_ref().unwrap().elapsed() >  Duration::from_secs(cfg.update_interval as u64*60){
                info!("thread :时间到");
                if !is_downloading(){
                    set_wallpaper_default();
                    instant = Some(Instant::now());
                }else{
                    info!("thread :is_downloading 不下载.")
                }
            }
            thread::sleep(sleep_dur);
        }
    });
}

/// 本地HTTP服务
pub fn start_http_server(){
    let cfg = config::load();
    let server = tiny_http::Server::http(&format!("0.0.0.0:{}", cfg.server_port)).unwrap();
    info!("Now listening on port {}", cfg.server_port);

    //生成连接二维码保存至配置文件
    let local_ip = local_ipaddress::get().unwrap();
    let app_url = format!("http://{}:{}", local_ip, cfg.server_port);
    let code = QrCode::new(app_url.as_bytes()).unwrap();
    let image = code.render::<Rgba<u8>>().build();
    let mut data = vec![];
    let encoder = PngEncoder::new(&mut data);
    encoder.write_image(&image, image.width(), image.height(), ColorType::Rgba8).unwrap();

    let mut encode_buf = vec![];
    let mut reader: &[u8] = data.as_bytes();

    let engine = engine::GeneralPurpose::new(&alphabet::URL_SAFE, GeneralPurposeConfig::new());

    let mut encoder: base64::write::EncoderWriter<'_, engine::GeneralPurpose, &mut Vec<u8>> = base64::write::EncoderWriter::new(&mut encode_buf, &engine);
    io::copy(&mut reader, &mut encoder);
    encoder.finish();

    let qrcode_image_base64 = String::from_utf8(encode_buf).unwrap();

    thread::spawn(move || {
        for mut rq in server.incoming_requests() {
            let url = rq.url();
            // info!("url: {}", url);
            let response = match url {
                "/favicon.ico" => tiny_http::Response::from_data(FACICON.to_vec()).with_header(
                    "Content-Type: image/ico"
                        .parse::<tiny_http::Header>()
                        .unwrap(),
                ),
                "/iconfont.woff2" => tiny_http::Response::from_data(ICON_FONT_WOFF2.to_vec())
                    .with_header(
                        "Content-Type: application/woff2"
                            .parse::<tiny_http::Header>()
                            .unwrap(),
                    ),
                "/iconfont.woff" => tiny_http::Response::from_data(ICON_FONT_WOFF.to_vec())
                    .with_header(
                        "Content-Type: application/woff"
                            .parse::<tiny_http::Header>()
                            .unwrap(),
                    ),
                "/iconfont.ttf" => tiny_http::Response::from_data(ICON_FONT_TTF.to_vec())
                    .with_header(
                        "Content-Type: application/ttf"
                            .parse::<tiny_http::Header>()
                            .unwrap(),
                    ),
                "/is_downloading" => {
                    tiny_http::Response::from_string(format!("{}", downloader::is_downloading()))
                }
                "/download" => {
                    //立即更新壁纸
                    set_wallpaper_default_async();
                    tiny_http::Response::from_string("OK")
                }
                "/config" => {
                    let mut resp = tiny_http::Response::from_string("OK".to_string());
                    match rq.method(){
                        Method::Get => { 
                            //返回配置信息
                            resp = tiny_http::Response::from_string(serde_json::to_string(&config::load()).unwrap_or("{}".to_string()));
                        }
                        _ => {
                            //保存配置信息
                            let mut conf_data = String::new();
                            if let Ok(_len) = rq.as_reader().read_to_string(&mut conf_data){
                                match serde_json::from_str::<Config>(&conf_data){
                                    Ok(mut cfg) => config::save(&mut cfg),
                                    Err(err) => error!("配置格式错误: {} {:?}", conf_data, err)
                                }
                            }
                        }
                    }
                    resp
                }
                "/exit" => {
                    // app::exit();
                    tiny_http::Response::from_string("OK")
                }
                "/update_interval" => {
                    let mut resp = tiny_http::Response::from_string("OK".to_string());
                    let mut cfg = config::load();
                    match rq.method(){
                        Method::Get => {
                            //返回当前 update_interval
                            resp = tiny_http::Response::from_string(format!("{}", cfg.update_interval));
                        }
                        Method::Post => {
                            cfg.update_interval += 10;
                            if cfg.update_interval > 60{
                                cfg.update_interval = 10;
                            }
                            config::save(&mut cfg);
                            resp = tiny_http::Response::from_string(format!("{}", cfg.update_interval));
                        }
                        _ => ()
                    }
                    resp
                }
                "/display_type" => {
                    let mut resp = tiny_http::Response::from_string("OK".to_string());
                    let mut cfg = config::load();
                    match rq.method(){
                        Method::Get => {
                            //返回当前display_type
                            resp = tiny_http::Response::from_string(format!("{}", cfg.display_type));
                        }
                        Method::Post => {
                            if cfg.display_type == 1{
                                cfg.display_type = 2;
                            }else{
                                cfg.display_type = 1;
                            }
                            config::save(&mut cfg);
                            resp = tiny_http::Response::from_string(format!("{}", cfg.display_type));
                        }
                        _ => ()
                    }
                    resp
                }
                "/is_startup" => {
                    let is_registered = app::is_app_registered_for_startup(APP_NAME).unwrap_or(false);
                    tiny_http::Response::from_string(format!("{}", is_registered))
                }
                "/set_startup" => {
                    let mut is_registered = app::is_app_registered_for_startup(APP_NAME).unwrap_or(false);
                    if !is_registered{
                        is_registered = if app::register_app_for_startup(APP_NAME).is_ok(){
                            info!("开机启动注册成功");
                            true
                        }else{
                            info!("开机启动注册失败");
                            false
                        };
                    }else{
                        let _ = app::remove_app_for_startup(APP_NAME);
                        info!("取消开机启动");
                        is_registered = false;
                    }
                    tiny_http::Response::from_string(format!("{}", is_registered))
                }
                _ =>{
                    let mut html = HTML.to_string();
                    while html.contains("{qrcode_base64}"){
                        html = html.replace("{qrcode_base64}", &qrcode_image_base64)
                    }
                    tiny_http::Response::from_string(html).with_header(
                    "Content-Type: text/html; charset=utf-8"
                        .parse::<tiny_http::Header>()
                        .unwrap())
                }
            };
            let _ = rq.respond(response);
        }
    });
}