mod config;
mod downloader;
mod app;
mod def;
mod server;
mod ui;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: slint::android::AndroidApp) {
    use app::{set_activity_ptr, set_vm_ptr, set_window_size};

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    set_vm_ptr(&app);
    set_activity_ptr(&app);
    set_window_size(&app);
    slint::android::init(app.clone()).unwrap();
    crate::app::run().unwrap();
}