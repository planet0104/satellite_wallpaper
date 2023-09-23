pub const APP_NAME: &str = "卫星壁纸";
pub const APP_NAME_E: &str = "SatelliteWallpaper";

pub const DEFAULT_SERVER_PORT: u32 = 50181;
pub const DEFAULT_DOWNLOAD_URL_H8: &str = "https://himawari8.nict.go.jp/img/";
// pub const DEFAULT_DOWNLOAD_URL_FY4A: &str = "http://rsapp.nsmc.org.cn/swapQuery/public/tileServer/getTile/fy-4a/reg_china/NatureColor/";
pub const DEFAULT_DOWNLOAD_URL_FY4A: &str = "http://rsapp.nsmc.org.cn/swapQuery/public/tileServer/getTile/fy-4a/full_disk/NatureColor_NoLit/";

pub const HTML: &str = include_str!("../res/index.html");
pub const FACICON: &[u8] = include_bytes!("../res/favicon_64.ico");
pub const ICON_FONT_TTF: &[u8] = include_bytes!("../res/iconfont.ttf");
pub const ICON_FONT_WOFF: &[u8] = include_bytes!("../res/iconfont.woff2");
pub const ICON_FONT_WOFF2: &[u8] = include_bytes!("../res/iconfont.woff");
