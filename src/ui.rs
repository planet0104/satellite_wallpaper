slint::slint!{
    import { TabWidget , VerticalBox, ComboBox, HorizontalBox, Button} from "std-widgets.slint";

    export component Main inherits Window {
        title: "卫星壁纸";
        width: 640px;
        height: 480px;
        icon: @image-url("res/favicon_64.ico");

        pure callback render-image(int) -> image;
        in-out property <int> image-frame;

        in property <string> current_wallpaper: "";
        in-out property <string> wallpaper_file: "";
        in property <string> f4a_data_url: "";
        in property <string> h8_data_url: "";
        in property <string> config_file: "";

        in-out property <int> current-satellite-index: 0;
        in-out property <int> current-interval-index: 0;
        in-out property <int> current-size-index: 0;
        in-out property <bool> is-startup: false;

        callback change_satellite(int);
        callback change_interval(int);
        callback change_wallpaper_size(int);
        callback change_startup(bool);
        callback sync_now();
        callback open_home_page();
        callback open_gitee_page();
        callback open_github_page();
        callback open_image_file();

        Rectangle {
            TabWidget {
                height: 100%;
                current-index: 0;
                Tab {
                    title: "　　　　首页 🌏　　　　";
                    Rectangle {
                        background: #202020;
                        VerticalBox {
                            height: 100%;
                            alignment: center;
                            HorizontalBox { 
                                alignment: center;
                                Image {
                                    source: render-image(image-frame);
                                }
                            }
                        }
                        TouchArea {
                            clicked => { 
                                open-image-file()
                            }
                        }
                    }
                }
                Tab {
                    title: "　　　　设置 ⛭　　　　";
                    Rectangle {
                        background: #202020;
                        VerticalBox {
                            ComboBox {
                                model: ["卫星：风云4号", "卫星：向日葵8号"];
                                current-value: "卫星："+(current-satellite-index==0?"风云4号":"向日葵8号");
                                current-index: current-satellite-index;
                                selected => {
                                    change-satellite(self.current-index)
                                }
                            }
                            ComboBox {
                                model: ["更新频率：10分钟", "更新频率：20分钟", "更新频率：30分钟", "更新频率：40分钟", "更新频率：50分钟", "更新频率：60分钟"];
                                current-value: "更新频率："+((current-interval-index+1) * 10)+"分钟";
                                current-index: current-interval-index;
                                selected => {
                                    change-interval(self.current-index)
                                }
                            }
                            ComboBox {
                                model: ["壁纸样式：整张", "壁纸样式：半张"];
                                current-value: "壁纸样式："+(current-size-index==0?"整张":"半张");
                                current-index: current-size-index;
                                selected => {
                                    change-wallpaper-size(self.current-index)
                                }
                            }
                            ComboBox {
                                model: ["开机启动：否", "开机启动：是"];
                                current-value: "开机启动："+(is-startup?"是":"否");
                                current-index: is-startup? 1: 0;
                                selected => {
                                    change-startup(self.current-index==1)
                                }
                            }
                            TextInput {
                                text: "当前壁纸:"+current_wallpaper;
                            }
                            TextInput {
                                text: "本地文件:"+wallpaper_file;
                            }
                            TextInput {
                                text: "风云4号A星数据地址:"+f4a_data_url;
                            }
                            TextInput {
                                text: "向日葵8号数据地址:"+h8_data_url;
                            }
                            TextInput {
                                text: "配置文件:"+config_file;
                            }
                            Button {
                                text: "立即更新壁纸🔄";
                                clicked => {
                                    sync-now()
                                }
                            }
                        }
                    }
                }
                Tab {
                    title: "　　　　关于 ℹ️　　　　";
                    Rectangle {
                        background: #202020;
                        VerticalBox {
                            Button {
                                text: "项目主页🔗\n \nhttps://www.ccfish.run/satellite_wallpaper/index.html";
                                clicked => { 
                                    open-home-page()
                                }
                            }
                            Button {
                                text: "Gitee代码库🔗\n \nhttps://gitee.com/planet0104-osc/satellite_wallpaper";
                                clicked => {
                                    open-gitee-page()
                                }
                            }
                            Button {
                                text: "Github代码库🔗\n \nhttps://github.com/planet0104/satellite_wallpaper";
                                clicked => {
                                    open-github-page()
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}