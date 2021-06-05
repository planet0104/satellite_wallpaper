fn main() {
    windows::build!(
        // Windows::Data::Xml::Dom::*,
        // Windows::Win32::System::Threading::{CreateEventW, SetEvent, WaitForSingleObject},
        // Windows::Win32::System::WindowsProgramming::CloseHandle,
        // Windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, GetDesktopWindow, MB_OK},
        Windows::Win32::UI::Shell::{SHGetSpecialFolderPathW, CSIDL_STARTUP,
            Shell_NotifyIconW, ShellExecuteW, NOTIFYICONDATAW, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIF_INFO, NIIF_NONE, NIF_ICON, NIF_MESSAGE, NIF_TIP},
        Windows::Win32::System::SystemServices::{
            GetModuleHandleW, HINSTANCE, LRESULT, PSTR, PWSTR, MAX_PATH, PWSTR,
        },
        Windows::Win32::Graphics::Gdi::{UpdateWindow, GetStockObject, WHITE_BRUSH, HBRUSH, HGDIOBJ },
        Windows::Win32::System::OleAutomation::{BSTR },
        Windows::Win32::System::Diagnostics::{
            Debug::GetLastError
        },
        Windows::Win32::UI::Controls::{ LR_DEFAULTCOLOR},
        Windows::Win32::UI::MenusAndResources::{HICON, HMENU },
        Windows::Win32::UI::DisplayDevices::{ POINT, RECT},
        Windows::System::UserProfile::*,
        Windows::Storage::*,
        Windows::Foundation::*,
        Windows::Storage::Streams::IRandomAccessStream,
        Windows::Win32::UI::WindowsAndMessaging::{
            RegisterWindowMessageW,
            RegisterClassW,
            CreateWindowExW,
            PostQuitMessage,
            SendMessageW,
            DefWindowProcW,
            LoadImageW,
            LoadCursorW,
            LoadIconW,
            WNDCLASSW,
            GetMessageW,
            PostMessageW,
            TranslateMessage,
            DispatchMessageW,
            GetWindowRect,
            CreatePopupMenu,
            AppendMenuW,
            GetCursorPos,
            SetForegroundWindow,
            TrackPopupMenu,
            TrackPopupMenuEx,

            ShowWindow, SHOW_WINDOW_CMD, WS_EX_TOOLWINDOW, SW_SHOWNORMAL,

            MSG,

            HWND, LPARAM, WPARAM, WM_CREATE, IMAGE_ICON, WM_LBUTTONDBLCLK, WM_RBUTTONDOWN, WM_LBUTTONDOWN,
            WM_DESTROY,
            WM_CLOSE,
            WM_KEYDOWN,
            WM_KEYUP,
            WM_PAINT,
            WM_QUIT,
            WM_USER,

            WS_POPUP, CW_USEDEFAULT, MF_POPUP, MF_STRING, TPM_RETURNCMD,

            IDC_ARROW, CS_HREDRAW, CS_VREDRAW, IDI_APPLICATION,

            GetDesktopWindow},
    );

    let mut res = winres::WindowsResource::new();
    // res.set_icon("favicon_128.ico");
    res.set_icon_with_id("res/favicon_64.ico", "IC_LOGO");
    res.compile().unwrap();
}
