use std::sync::RwLock;

use anyhow::{anyhow, Result};
use image::{Rgba, RgbaImage};
use jni::{
    objects::{AsJArrayRaw, JObject, JString, JValue, JValueGen},
    sys::{JNIInvokeInterface_, _jobject, jint, jintArray},
    JavaVM,
};
use log::info;
use once_cell::sync::Lazy;
use slint::android::AndroidApp;

use super::open_main_window;

pub struct AppRef<'a>{
    app: std::sync::RwLockReadGuard<'a, Option<AndroidApp>>
}

impl <'a> AppRef<'a>{
    fn get(&'a self) -> &'a AndroidApp{
        self.app.as_ref().unwrap()
    }
}

pub static ANDROID_APP: Lazy<RwLock<Option<AndroidApp>>> = Lazy::new(|| RwLock::new(None));

pub fn get_app<'a>() -> Result<AppRef<'a>>{
    let app: std::sync::RwLockReadGuard<'_, Option<AndroidApp>> = ANDROID_APP.read().map_err(|err| anyhow!("{:?}", err))?;
    if app.is_none(){
        return Err(anyhow!("App未初始化!"));
    }
    Ok(AppRef{ app })
}

pub fn run() -> Result<()> {
    open_main_window();
    Ok(())
}

/// 同步设置锁屏壁纸
pub fn set_lock_screen_image(image: &str) -> Result<()>{
    info!("Android调用 set_lock_screen_image:{image}");
    Ok(())
}

// 设置壁纸
pub fn set_wallpaper_from_path(image: &str) -> Result<()>{
    info!("Android调用 set_wallpaper_from_path:{image}");
    android_set_wallpaper(&get_app()?.get(), image)?;
    info!("Android调用 set_wallpaper_from_path:成功");
    Ok(())
}

pub fn get_current_wallpaper() -> Result<String>{
    Err(anyhow!("获取失败"))
}

pub fn is_app_registered_for_startup(name:&str) -> Result<bool>{
    Ok(false)
}

pub fn register_app_for_startup(app_name:&str) -> Result<()>{
    Ok(())
}

pub fn open_file(path: &str){
    info!("打开文件:{path}");
}

pub fn remove_app_for_startup(app_name:&str) -> Result<()>{
    Ok(())
}

pub fn get_config_dir() -> String{
    let app = get_app().unwrap();
    get_files_dir(&app.get()).unwrap()
}

pub fn get_app_home_dir() -> String {
    let app = get_app().unwrap();
    get_files_dir(&app.get()).unwrap()
}

pub fn get_screen_size() -> (i32, i32){
    let app = get_app().unwrap();
    let size = get_window_size(&app.get());
    info!("Android屏幕大小:{:?}", size);
    size
}

pub fn get_window_size(app: &AndroidApp) -> (i32, i32){
    match app.native_window(){
        None => (1080, 1920),
        Some(w) => (w.width(), w.height())
    }
} 

pub fn get_files_dir(app: &AndroidApp) -> Result<String> {
    unsafe {
        let vm = JavaVM::from_raw(app.vm_as_ptr() as *mut *const JNIInvokeInterface_)?;
        let mut env = vm.attach_current_thread()?;
        let activity: JObject<'_> = JObject::from_raw(app.activity_as_ptr() as *mut _jobject);

        let file = env.call_method(activity, "getFilesDir", "()Ljava/io/File;", &[])?;

        if let JValueGen::Object(file) = file {
            let path = env.call_method(file, "getAbsolutePath", "()Ljava/lang/String;", &[])?;

            if let JValueGen::Object(path) = path {
                let path: JString = path.into();
                let str = env.get_string(&path)?;
                let str = std::ffi::CStr::from_ptr(str.get_raw());
                Ok(str.to_str()?.to_string())
            } else {
                Err(anyhow!("object is not a string"))
            }
        } else {
            Err(anyhow!("object is not a file"))
        }
    }
}

pub fn get_cache_dir(app: &AndroidApp) -> Result<String> {
    unsafe {
        let vm = JavaVM::from_raw(app.vm_as_ptr() as *mut *const JNIInvokeInterface_)?;
        let mut env = vm.attach_current_thread()?;
        let activity: JObject<'_> = JObject::from_raw(app.activity_as_ptr() as *mut _jobject);

        let file = env.call_method(activity, "getCacheDir", "()Ljava/io/File;", &[])?;

        if let JValueGen::Object(file) = file {
            let path = env.call_method(file, "getAbsolutePath", "()Ljava/lang/String;", &[])?;

            if let JValueGen::Object(path) = path {
                let path: JString = path.into();
                let str = env.get_string(&path)?;
                let str = std::ffi::CStr::from_ptr(str.get_raw());
                Ok(str.to_str()?.to_string())
            } else {
                Err(anyhow!("object is not a string"))
            }
        } else {
            Err(anyhow!("object is not a file"))
        }
    }
}

pub fn android_set_wallpaper(app: &slint::android::AndroidApp, path:&str) -> Result<()>{
    if !has_wallpaper_permission(app)?{
        request_wallpaper_permission(app)?;
        return Err(anyhow!("No Permission"));
    }

    let image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::open(path)?.to_rgba8();

    let colors = convert_image_to_vec(&image);

    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr() as *mut *const JNIInvokeInterface_) }?;
    let mut env = vm.attach_current_thread()?;
    let activity: JObject<'_> = unsafe { JObject::from_raw(app.activity_as_ptr() as *mut _jobject) };
    let activity_jvalue = JValueGen::try_from(&activity)?;
    let wallpaper_manager = env.call_static_method("android/app/WallpaperManager", "getInstance", "(Landroid/content/Context;)Landroid/app/WallpaperManager;", &[activity_jvalue])?;
    let wallpaper_manager = JObject::try_from(wallpaper_manager)?;

    let bitmap = create_java_bitmap_form_colors(vm.attach_current_thread()?, &colors, image.width() as i32, image.height() as i32)?;
    let bitmap_j = JValueGen::try_from(&bitmap)?;

    let _ = env.call_method(wallpaper_manager, "setBitmap", "(Landroid/graphics/Bitmap;)V", &[bitmap_j])?.v()?;
    Ok(())
}

pub fn android_set_lock_screen_wallpaper(){

}

pub fn check_self_permission(app: &slint::android::AndroidApp, permission: &str) -> Result<bool> {
    unsafe {
        let vm = JavaVM::from_raw(app.vm_as_ptr() as *mut *const JNIInvokeInterface_)?;
        let mut env = vm.attach_current_thread()?;
        let granted_int = env
            .get_static_field(
                "android/content/pm/PackageManager",
                "PERMISSION_GRANTED",
                "I",
            )?
            .i()?;
        // 创建Java字符串
        let permission_str = env.new_string(permission)?;
        let activity: JObject<'_> = JObject::from_raw(app.activity_as_ptr() as *mut _jobject);
        let result = env
            .call_method(
                activity,
                "checkSelfPermission",
                "(Ljava/lang/String;)I",
                &[JValueGen::Object(&JObject::from(permission_str))],
            )?
            .i()?;
        Ok(result == granted_int)
    }
}


pub fn request_permissions(
    app: &slint::android::AndroidApp,
    permissions: &[&str],
    request_code: i32,
) -> Result<()> {
    unsafe {
        let vm = JavaVM::from_raw(app.vm_as_ptr() as *mut *const JNIInvokeInterface_)?;
        let mut env = vm.attach_current_thread()?;
        let activity: JObject<'_> = JObject::from_raw(app.activity_as_ptr() as *mut _jobject);

        // 创建一个Java String数组
        let permission_count = permissions.len() as jint;
        let java_permission_array =
            env.new_object_array(permission_count, "java/lang/String", JObject::null())?;
        for (index, permission) in permissions.iter().enumerate() {
            let permission_str = env.new_string(*permission)?;
            env.set_object_array_element(&java_permission_array, index as jint, permission_str)?;
        }

        // 调用requestPermissions方法
        let _ = env.call_method(
            activity,
            "requestPermissions",
            "([Ljava/lang/String;I)V",
            &[
                JValueGen::Object(&JObject::from(java_permission_array)),
                request_code.into(),
            ],
        )?;
    }
    Ok(())
}

pub fn has_wallpaper_permission(app: &slint::android::AndroidApp) -> Result<bool> {
    let sdk_version = sdk_version(app)?;
    info!("sdk version:{sdk_version}");
    let permission = "android.permission.SET_WALLPAPER";
    if sdk_version > 23 {
        if !check_self_permission(app, permission)? {
            return Ok(false)
        }else{
            Ok(true)
        }
    }else{
        Ok(true)
    }
}

pub fn request_wallpaper_permission(app: &slint::android::AndroidApp) -> Result<()> {
    let sdk_version = sdk_version(app)?;
    info!("sdk version:{sdk_version}");
    let permission = "android.permission.SET_WALLPAPER";
    if sdk_version > 23 {
        if !check_self_permission(app, permission)? {
            request_permissions(app, &[permission], 100)?;
        }
    }
    Ok(())
}

pub fn sdk_version(app: &slint::android::AndroidApp) -> Result<i32> {
    unsafe {
        let vm = JavaVM::from_raw(app.vm_as_ptr() as *mut *const JNIInvokeInterface_)?;
        let mut env = vm.attach_current_thread()?;
        Ok(env
            .get_static_field("android/os/Build$VERSION", "SDK_INT", "I")?
            .i()?)
    }
}

/// 创建bitmap对象
pub fn create_java_bitmap_form_colors<'a>(mut env: jni::AttachGuard<'a>, colors:&[i32], width: i32, height:i32) -> Result<JObject<'a>>{
    info!("create_java_bitmap_form_colors step1 get Config class...");
	// 获取 Config 类
    let config = env.get_static_field("android/graphics/Bitmap$Config", "ARGB_8888", "Landroid/graphics/Bitmap$Config;")?;
    let config_ref = JValueGen::try_from(&config)?;
    info!("create_java_bitmap_form_colors step2 new_int_array...");
    let intarray = env.new_int_array(width*height)?;
    env.set_int_array_region(&intarray, 0, colors)?;
    let arr_jobj: JObject<'_> = unsafe { JObject::from_raw(**intarray) };
    let arr_jvalue = JValueGen::try_from(&arr_jobj)?;
    info!("create_java_bitmap_form_colors step3 createBitmap...");
	let bitmap = env.call_static_method("android/graphics/Bitmap", "createBitmap", "([IIILandroid/graphics/Bitmap$Config;)Landroid/graphics/Bitmap;",
		&[
			arr_jvalue,
			JValue::from(width),
			JValue::from(height),
			config_ref
		])?;
    info!("create_java_bitmap_form_colors step3 OK.");
	Ok(bitmap.l()?)
}

fn rgba_to_i32(rgba: &Rgba<u8>) -> i32 {
    let [r, g, b, a] = rgba.0;
    // 将每个通道的 u8 值组合成一个 i32 值
    // 注意：这里假设每个通道的值直接组合成一个 i32，不进行任何缩放或位移
    ((a as i32) << 24) | ((r as i32) << 16) | ((g as i32) << 8) | (b as i32)
}

fn convert_image_to_vec(image: &RgbaImage) -> Vec<i32> {
    image
        .pixels()
        .map(|pixel| rgba_to_i32(pixel))
        .collect()
}