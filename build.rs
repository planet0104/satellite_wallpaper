fn main() {
    let mut res = winres::WindowsResource::new();
    // res.set_icon("favicon_128.ico");
    res.set_icon_with_id("res/favicon_64.ico", "IC_LOGO");
    res.compile().unwrap();
}
