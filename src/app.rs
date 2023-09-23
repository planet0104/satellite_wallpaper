use std::{mem, path::Path, ffi::OsStr, os::windows::prelude::OsStrExt};
use anyhow::{anyhow, Result};
use windows::{Win32::{UI::{Shell::{NOTIFYICONDATAW, Shell_NotifyIconW, NIF_INFO, NIIF_NONE, NIM_MODIFY, ShellExecuteW, SHGetSpecialFolderPathW, CSIDL_STARTUP}, WindowsAndMessaging::{SW_SHOWNORMAL, GetDesktopWindow, GetWindowRect}}, Foundation::{HWND, RECT, MAX_PATH}}, core::{PCWSTR, HSTRING}, Storage::StorageFile, System::UserProfile::LockScreen};

static TEMPLATE:&str = r"[InternetShortcut]
URL=--
IconIndex=0
IconFile=--
";

use crate::config;

use super::def::*;
use once_cell::sync::Lazy;
use std::{borrow::BorrowMut, sync::Mutex};

static NOTIFY_ICON_DATA: Lazy<Mutex<NOTIFYICONDATAW>> =
    Lazy::new(|| Mutex::new(unsafe { std::mem::zeroed() }));

/// 显示气泡提示
pub fn show_bubble(info: &str) {
    if let Ok(nid) = NOTIFY_ICON_DATA.lock().borrow_mut() {
        let nid: &mut NOTIFYICONDATAW = nid;
        let htitle = HSTRING::from(APP_NAME);
        let title = htitle.as_wide();
        nid.szInfoTitle[0..title.len()].copy_from_slice(title);
        let hinfo = HSTRING::from(info);
        let info = hinfo.as_wide();
        nid.szInfo[0..info.len()].copy_from_slice(info);
        nid.uFlags = NIF_INFO;
        nid.dwInfoFlags = NIIF_NONE;
        unsafe { Shell_NotifyIconW(NIM_MODIFY, nid) };
    }
}

pub fn open_in_browser(){
    unsafe{
        let cfg = config::load();
        let url = format!("http://localhost:{}", cfg.server_port);
        ShellExecuteW(None, PCWSTR(create_pcwstr("open").as_ptr()), PCWSTR(create_pcwstr(&format!("{}\0", url)).as_ptr()), None, None, SW_SHOWNORMAL);
    }
}

fn create_pcwstr(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub fn get_screen_size() -> (i32, i32){
    let hwnd = unsafe{ GetDesktopWindow() };
    let mut rect:RECT = unsafe{ mem::zeroed() };
    if unsafe{ GetWindowRect(hwnd, &mut rect).is_ok() }{
        (rect.right-rect.left, rect.bottom-rect.top)
    }else{
        (1920, 1080)
    }
}

/// 同步设置锁屏壁纸
pub fn set_lock_screen_image(image: &str) -> Result<()>{
    let file = StorageFile::GetFileFromPathAsync(&HSTRING::from(image))?.get()?;
    LockScreen::SetImageFileAsync(&file)?.get()?;
    Ok(())
}

pub fn remove_app_for_startup(app_name:&str) -> Result<()>{
    let hwnd = unsafe{ GetDesktopWindow() };
    let mut path:[u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    unsafe{ SHGetSpecialFolderPathW(hwnd, &mut path, CSIDL_STARTUP as i32, false) };
    let path = String::from_utf16(&path)?.replace("\u{0}", "");
    std::fs::remove_file(format!("{}\\{}.url", path, app_name))?;
    Ok(())
}

pub fn register_app_for_startup(app_name:&str) -> Result<()>{
    let hwnd: HWND = unsafe{ GetDesktopWindow() };
    let mut path:[u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    unsafe{ SHGetSpecialFolderPathW(hwnd, &mut path, CSIDL_STARTUP as i32, false) };
    let path = String::from_utf16(&path)?.replace("\u{0}", "");
    let url_file = format!("{}\\{}.url", path, app_name);
    //写入url文件
    use std::io::Write;
    let mut file = std::fs::File::create(url_file)?;
    let exe_path = ::std::env::current_exe()?;
    if let Some(exe_path) = exe_path.to_str(){
        file.write_all(TEMPLATE.replace("--", exe_path).as_bytes())?;
        Ok(())
    }else{
        Err(anyhow!("exe路径读取失败!"))
    }
}

pub fn is_app_registered_for_startup(app_name:&str) -> Result<bool>{
    let hwnd = unsafe{ GetDesktopWindow() };
    let mut path:[u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    unsafe{ SHGetSpecialFolderPathW(hwnd, &mut path, CSIDL_STARTUP as i32, false) };
    let path = String::from_utf16(&path)?.replace("\u{0}", "");
    Ok(Path::new(&format!("{}\\{}.url", path, app_name)).exists())
}