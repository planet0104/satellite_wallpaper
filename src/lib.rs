mod config;
mod downloader;
mod app;
mod def;
mod server;
mod ui;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: slint::android::AndroidApp) {
    use log::info;

    info!("android_main<<<<<<<<<<<<<<<< 0001");
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    info!("android_main<<<<<<<<<<<<<<<< 0002");

    *app::ANDROID_APP.write().unwrap() = Some(app.clone());
    info!("android_main<<<<<<<<<<<<<<<< 0003");
    slint::android::init(app.clone()).unwrap();
    info!("android_main<<<<<<<<<<<<<<<< 0004");
    crate::app::run().unwrap();
    info!("android_main<<<<<<<<<<<<<<<< 0005");
}