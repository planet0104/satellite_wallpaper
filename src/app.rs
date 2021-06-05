mod bindings {
    windows::include_bindings!();
}
use std::{mem::{self, size_of}, path::Path};
use log::info;
use anyhow::{anyhow, Result};

use windows::HSTRING;
use bindings::{
    Windows::Win32::Graphics::Gdi::{
        GetStockObject, UpdateWindow, HBRUSH, WHITE_BRUSH,
    },
    Windows::Win32::UI::DisplayDevices::{POINT, RECT},
    Windows::Win32::System::SystemServices::*,
    Windows::Win32::UI::Controls::LR_DEFAULTCOLOR,
    Windows::Win32::UI::MenusAndResources::*,
    Windows::Win32::UI::Shell::*,
    Windows::Win32::UI::WindowsAndMessaging::*,
    Windows::System::UserProfile::*,
    Windows::Storage::*
};

const IDR_EXIT: usize = 10;
const IDR_OPEN: usize = 11;

static TEMPLATE:&str = r"[InternetShortcut]
URL=--
IconIndex=0
IconFile=--
";

use crate::config;

use super::def::*;
use once_cell::sync::Lazy;
use std::{borrow::BorrowMut, ptr::null_mut, sync::Mutex};

static NOTIFY_ICON_DATA: Lazy<Mutex<NOTIFYICONDATAW>> =
    Lazy::new(|| Mutex::new(unsafe { std::mem::zeroed() }));
static WM_TASKBAR_CREATED: Lazy<u32> =
    Lazy::new(|| unsafe { RegisterWindowMessageW("TaskbarCreated") });
static MENU_HANDLE: Lazy<Mutex<HMENU>> =
    Lazy::new(|| Mutex::new(unsafe { std::mem::zeroed() }));

//窗口消息函数
extern "system" fn wndproc(
    h_wnd: HWND,
    u_msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    // info!("wndproc msg: {}", u_msg);
    match u_msg {
        WM_CREATE => {
            // info!("WM_CREATE");
            to_tray(h_wnd);
            //弹出气泡
            show_bubble("已启动");
            //在浏览器打开
            open_in_browser();
        }
        WM_USER => {
            match l_param.0 as u32 {
                // WM_LBUTTONDBLCLK => {
                //     open_in_browser();
                // }
                WM_RBUTTONDOWN | WM_LBUTTONDOWN => {
                    unsafe{
                        let mut pt: POINT = POINT { x: 0, y: 0 };
                        GetCursorPos(&mut pt); //取鼠标坐标
                        SetForegroundWindow(h_wnd); //解决在菜单外单击左键菜单不消失的问题
                                                    // EnableMenuItem(hmenu,IDR_PAUSE,MF_GRAYED);//让菜单中的某一项变灰
                        let h_menu = *MENU_HANDLE.lock().unwrap();
                        match TrackPopupMenu(
                            h_menu,
                            TPM_RETURNCMD,
                            pt.x,
                            pt.y,
                            0,
                            h_wnd,
                            null_mut(),
                        ).0 as usize
                        {
                            //显示菜单并获取选项ID
                            IDR_EXIT => {
                                //PostMessage将消息放入消息队列后马上返回，而SendMessage直到窗口过程处理完消息后才返回
                                delete_tray();
                                PostMessageW(h_wnd, WM_QUIT, w_param, l_param);
                            }
                            IDR_OPEN => {
                                open_in_browser();
                            }
                            0 => {
                                PostMessageW(h_wnd, WM_LBUTTONDOWN, None, None);
                            }
                            _ => {}
                        }
                    }
                }
                _ => (),
            }
        }
        WM_DESTROY => {
            info!("程序结束 WM_DESTROY");
            delete_tray();
            unsafe{ PostQuitMessage(0) };
        }
        _ => {
            /*
             * 防止当Explorer.exe 崩溃以后，程序在系统系统托盘中的图标就消失
             *
             * 原理：Explorer.exe 重新载入后会重建系统任务栏。当系统任务栏建立的时候会向系统内所有
             * 注册接收TaskbarCreated 消息的顶级窗口发送一条消息，我们只需要捕捉这个消息，并重建系
             * 统托盘的图标即可。
             */
            if u_msg == *WM_TASKBAR_CREATED {
                unsafe { SendMessageW(h_wnd, WM_CREATE, w_param, l_param) };
            }
        }
    }
    unsafe { DefWindowProcW(h_wnd, u_msg, w_param, l_param) }
}

pub fn start_app(i_cmd_show: u32) -> i32{
    crate::server::start_async();

    let instance = unsafe { GetModuleHandleW(None) };
    // debug_assert!(instance.0 != 0);

    let wc = WNDCLASSW {
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW) },
        hIcon: unsafe { LoadIconW(None, IDI_APPLICATION) },
        hInstance: instance,
        cbClsExtra: 0,
        cbWndExtra: 0,
        hbrBackground: unsafe { HBRUSH(GetStockObject(WHITE_BRUSH).0) },
        lpszClassName: PWSTR(format!("{}\0", APP_NAME).as_ptr() as _),
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        ..Default::default()
    };

    let _atom = unsafe { RegisterClassW(&wc) };
    // info!("RegisterClassW = {}", atom);

    let handle = unsafe {
        CreateWindowExW(
            WS_EX_TOOLWINDOW,
            wc.lpszClassName,
            APP_NAME,
            WS_POPUP,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            instance,
            std::ptr::null_mut(),
        )
    };

    // info!("GetLastError()={}", unsafe{ GetLastError().0 });

    let mut message = MSG::default();

    unsafe {
        ShowWindow(handle, SHOW_WINDOW_CMD(i_cmd_show));
        // ShowWindow(handle, SHOW_WINDOW_CMD(1));

        UpdateWindow(handle);

        while GetMessageW(&mut message, handle, 0, 0).0 != 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    message.wParam.0 as i32
}

/// 创建托盘
fn to_tray(hwnd: HWND) {
    if let Ok(nid) = NOTIFY_ICON_DATA.lock().borrow_mut() {
        let nid: &mut NOTIFYICONDATAW = nid;
        nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 0;
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        nid.uCallbackMessage = WM_USER; //自定义的消息 处理托盘图标事件
                                        //加载资源文件中的光标
        let instance = unsafe { GetModuleHandleW(None) };
        nid.hIcon = HICON(unsafe {
            LoadImageW(instance, "IC_LOGO", IMAGE_ICON, 0, 0, LR_DEFAULTCOLOR).0
        });
        let htip = HSTRING::from(APP_NAME);
        let tip = htip.as_wide();
        nid.szTip[0..tip.len()].copy_from_slice(tip);
        unsafe { Shell_NotifyIconW(NIM_ADD, nid) }; //在托盘区添加图标
    }

    //一级菜单
    unsafe{
        let h_menu = CreatePopupMenu();
        AppendMenuW(h_menu, MF_STRING, IDR_OPEN, "打开");
        AppendMenuW(h_menu, MF_STRING, IDR_EXIT, "退出");
        *MENU_HANDLE.lock().unwrap() = h_menu;
    }
}

/// 删除托盘
fn delete_tray() {
    if let Ok(nid) = NOTIFY_ICON_DATA.lock().borrow_mut() {
        let nid: &mut NOTIFYICONDATAW = nid;
        unsafe { Shell_NotifyIconW(NIM_DELETE, nid) };
    }
}

/// 显示气泡提示
fn show_bubble(info: &str) {
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
        ShellExecuteW(None, "open", url, None, None, SW_SHOWNORMAL.0 as i32);
    }
}

pub fn get_screen_size() -> (i32, i32){
    let hwnd = unsafe{ GetDesktopWindow() };
    let mut rect:RECT = unsafe{ mem::zeroed() };
    if unsafe{ GetWindowRect(hwnd, &mut rect).0 != 0 }{
        (rect.right-rect.left, rect.bottom-rect.top)
    }else{
        (1920, 1080)
    }
}

/// 同步设置锁屏壁纸
pub fn set_lock_screen_image(image: &str) -> Result<()>{
    let file = StorageFile::GetFileFromPathAsync(image)?.get()?;
    LockScreen::SetImageFileAsync(file)?.get()?;
    Ok(())
}

pub fn remove_app_for_startup(app_name:&str) -> Result<()>{
    let hwnd = unsafe{ GetDesktopWindow() };
    let mut path:[u16; MAX_PATH as usize+1] = [0; MAX_PATH as usize+1];
    unsafe{ SHGetSpecialFolderPathW(hwnd, PWSTR(path.as_mut_ptr()), CSIDL_STARTUP as i32, false) };
    let path = String::from_utf16(&path)?.replace("\u{0}", "");
    std::fs::remove_file(format!("{}\\{}.url", path, app_name))?;
    Ok(())
}

pub fn register_app_for_startup(app_name:&str) -> Result<()>{
    let hwnd = unsafe{ GetDesktopWindow() };
    let mut path:[u16; MAX_PATH as usize+1] = [0; MAX_PATH as usize+1];
    unsafe{ SHGetSpecialFolderPathW(hwnd, PWSTR(path.as_mut_ptr()), CSIDL_STARTUP as i32, false) };
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
    let mut path:[u16; MAX_PATH as usize+1] = [0; MAX_PATH as usize+1];
    unsafe{ SHGetSpecialFolderPathW(hwnd, PWSTR(path.as_mut_ptr()), CSIDL_STARTUP as i32, false) };
    let path = String::from_utf16(&path)?.replace("\u{0}", "");
    Ok(Path::new(&format!("{}\\{}.url", path, app_name)).exists())
}