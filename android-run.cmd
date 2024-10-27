@REM 运行时请解除 Cargo.toml中的注释
@REM crate-type = cdylib

@REM build.rs中把编译资源的代码注释掉

@REM 编译时，或者切换到android环境，rust-analyzer会卡住，因为编译 skia_bindings库速度太慢。设置 SKIA_BINARIES_URL环境变量，值为http开头的地址，可解决。

@REM 请设置系统环境变量，否则无法正常编译 例如：ANDROID_NDK = "D:\android-ndk-r21e"

cargo apk run --target aarch64-linux-android --lib