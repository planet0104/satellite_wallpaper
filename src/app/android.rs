use std::ffi::c_void;

use anyhow::{anyhow, Result};
use image::{Rgba, RgbaImage};
use jni::{
    objects::{JObject, JString, JValue, JValueGen},
    sys::{JNIInvokeInterface_, _jobject, jint},
    JavaVM,
};
use log::info;
use slint::android::AndroidApp;

use super::open_main_window;

pub static mut VM_PTR: usize = 0;
pub static mut ACTIVITY_PTR: usize = 0;
pub static mut WINDOW_SIZE: (i32, i32) = (0, 0);

pub fn get_vm() -> Result<JavaVM>{
    if unsafe { VM_PTR } == 0{
        return Err(anyhow!("vm指针为空!"));
    }
    let vm_ptr: *mut c_void = unsafe { VM_PTR } as *mut c_void;
    let vm = unsafe { JavaVM::from_raw(vm_ptr as *mut *const JNIInvokeInterface_) }?;
    Ok(vm)
}

pub fn set_vm_ptr(app: &AndroidApp){
    unsafe { VM_PTR = app.vm_as_ptr() as usize };
}

pub fn get_activity_ptr() -> Result<*mut c_void>{
    if unsafe { ACTIVITY_PTR } == 0{
        return Err(anyhow!("ACTIVITY_PTR指针为空!"));
    }
    let ptr: *mut c_void = unsafe { ACTIVITY_PTR } as *mut c_void;
    Ok(ptr)
}

pub fn set_activity_ptr(app: &AndroidApp){
    unsafe { ACTIVITY_PTR = app.activity_as_ptr() as usize };
}

pub fn get_native_window_size() -> Result<(i32, i32)>{
    if unsafe { WINDOW_SIZE.0 == 0 || WINDOW_SIZE.1 == 0}{
        return Err(anyhow!("WINDOW_SIZE为空!"));
    }
    Ok(unsafe { WINDOW_SIZE })
}

pub fn set_window_size(app: &AndroidApp){
    if let Some(w) = app.native_window(){
        unsafe { WINDOW_SIZE = (w.width(), w.height()) };
    }
}

pub fn run() -> Result<()> {
    open_main_window();
    Ok(())
}

pub fn get_wallpaper_file_path() -> String {
    let wallpaper_path_name = format!( "{}/wallpaper.png", get_app_home_dir());
    info!("wallpaper {:?}", wallpaper_path_name);
    wallpaper_path_name
}

/// 同步设置锁屏壁纸
pub fn set_lock_screen_image(image: &str) -> Result<()>{
    info!("Android调用 set_lock_screen_image:{image}");
    Ok(())
}

// 设置壁纸
pub fn set_wallpaper_from_path(image: &str) -> Result<()>{
    let image = image.to_string();
    info!("set_wallpaper_from_path 开始.....");
    std::thread::spawn(move ||{
        info!("Android调用 android_set_wallpaper:{image}");
        let ret = android_set_wallpaper(&image);
        info!("Android调用 android_set_wallpaper: {:?}", ret);
    });
    info!("set_wallpaper_from_path 调用android_set_wallpaper结束.....");
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
    get_files_dir().unwrap()
}

pub fn get_app_home_dir() -> String {
    get_files_dir().unwrap()
}

pub fn get_screen_size() -> (i32, i32){
    let size = get_window_size();
    info!("Android屏幕大小:{:?}", size);
    size
}

pub fn get_window_size() -> (i32, i32){
    match get_native_window_size(){
        Err(_) => (1080, 1920),
        Ok(w) => w
    }
} 

pub fn get_files_dir() -> Result<String> {
    unsafe {
        let vm = get_vm()?;
        let mut env = vm.attach_current_thread()?;
        let activity: JObject<'_> = JObject::from_raw(get_activity_ptr()? as *mut _jobject);

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

pub fn android_set_wallpaper(path:&str) -> Result<()>{
    if !has_wallpaper_permission()?{
        request_wallpaper_permission()?;
        return Err(anyhow!("No Permission"));
    }
    info!("读取文件:{path}");
    let image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::open(path)?.to_rgba8();

    let colors = convert_image_to_vec(&image);

    let vm = get_vm()?;
    let mut env = vm.attach_current_thread()?;
    let activity: JObject<'_> = unsafe { JObject::from_raw(get_activity_ptr()? as *mut _jobject) };
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

pub fn check_self_permission(permission: &str) -> Result<bool> {
    unsafe {
        let vm = get_vm()?;
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
        let activity: JObject<'_> = JObject::from_raw(get_activity_ptr()? as *mut _jobject);
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
    permissions: &[&str],
    request_code: i32,
) -> Result<()> {
    unsafe {
        let vm = get_vm()?;
        let mut env = vm.attach_current_thread()?;
        let activity: JObject<'_> = JObject::from_raw(get_activity_ptr()? as *mut _jobject);

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

pub fn has_wallpaper_permission() -> Result<bool> {
    let sdk_version = sdk_version()?;
    info!("sdk version:{sdk_version}");
    let permission = "android.permission.SET_WALLPAPER";
    if sdk_version > 23 {
        if !check_self_permission(permission)? {
            return Ok(false)
        }else{
            Ok(true)
        }
    }else{
        Ok(true)
    }
}

pub fn request_wallpaper_permission() -> Result<()> {
    let sdk_version = sdk_version()?;
    info!("sdk version:{sdk_version}");
    let permission = "android.permission.SET_WALLPAPER";
    if sdk_version > 23 {
        if !check_self_permission(permission)? {
            request_permissions(&[permission], 100)?;
        }
    }
    Ok(())
}

pub fn sdk_version() -> Result<i32> {
    unsafe {
        let vm = get_vm()?;
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