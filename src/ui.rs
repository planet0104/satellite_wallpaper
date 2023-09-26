slint::slint!{
    import { TabWidget , VerticalBox, ComboBox, HorizontalBox, Button} from "std-widgets.slint";

    export component Main inherits Window {
        title: "卫星壁纸";
        width: 640px;
        height: 480px;
        icon: @image-url("res/favicon_64.ico");

        pure callback render-image() -> image;

        Rectangle {
            TabWidget {
                height: 100%;
                current-index: 1;
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
                                    source: render-image();
                                }
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
                                current-value: "卫星：风云4号";
                            }
                            ComboBox {
                                model: ["更新频率：10分钟", "更新频率：20分钟", "更新频率：30分钟", "更新频率：40分钟", "更新频率：50分钟", "更新频率：60分钟"];
                                current-value: "更新频率：10分钟";
                            }
                            ComboBox {
                                model: ["壁纸样式：整张", "壁纸样式：半张张"];
                                current-value: "壁纸样式：整张";
                            }
                            ComboBox {
                                model: ["开机启动：否", "开机启动：是"];
                                current-value: "开机启动：否";
                            }
                            Text {
                                text: "当前壁纸:";
                            }
                            Text {
                                text: "本地文件:";
                            }
                            Text {
                                text: "风云4号A星数据地址:";
                            }
                            Text {
                                text: "向日葵8号数据地址:";
                            }
                            Text {
                                text: "配置文件:";
                            }
                            Button {
                                text: "立即更新壁纸🔄";
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
                            }
                            Button {
                                text: "Gitee代码库🔗\n \nhttps://gitee.com/planet0104-osc/satellite_wallpaper";
                            }
                            Button {
                                text: "Github代码库🔗\n \nhttps://github.com/planet0104/satellite_wallpaper";
                            }
                        }
                    }
                }
            }
        }
    }
}