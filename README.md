# 微趣鸭 Rust Cli 客户端

使用 Rust 写出的神秘鸭客户端，多平台支持，以命令行方式运行。

## 特点

 - 运行快：Rust 语言作为编译型语言，运行速度要高于解释型语言。
 - 省资源：运行内存占用约 3MB，适合在一些小型设备上运行。（但设备可能也没那么小型）
 - 少依赖：二进制文件直接运行，不需要环境前置，可以便携运行。
 - 可自定义运行器：更自然和方便的命令运行方式，并异步等待命令结束，返回命令结果。

## 运行

你可以在终端（ cmd 、bash 等）中运行这个程序。
对于 Windows 用户来说，可以用类似于下面的 bat 运行程序。

```bat
@echo off

rem 假设你将文件放在 D 盘根目录
cd /d D:\

rem 假设你的文件名叫 wqy-client-rs_win64.exe
rem 假设你的设备 id 是 12345678，你的设备密码是 123456
rem 在实际使用时，请将该文本替换成实际的 ID 和密码

wqy-client-rs_win64.exe -i 12345678 -p 123456 -l info
```

## 运行帮助

```
wqy-client-rs 0.1.2
Command line version of weduck client implemented by Rust

USAGE:
    wqy-client-rs.exe [OPTIONS] --id <device-id> --password <device-password>

FLAGS:
    -h, --help
            Prints help information

    -V, --version
            Prints version information


OPTIONS:
    -i, --id <device-id>
            设备 id, 比如 12345678

    -p, --password <device-password>
            设备密码, 比如 123456

    -l, --log-level <log-level>
            env_logger 日志级别，举例：[Error|Info|Debug]
            将设置你的环境变量 "RUST_LOG" 为该值。
            当然，你也可以自行设置环境变量。
            
    -r, --max-retry <max-retry-times>
            最大连接失败重试次数 [默认: 5]

    -e, --shell-executor <shell-executor>
            用户自定义命令运行器

            比如，对 Windows 系统来说，默认的运行器等同于输入 -e "cmd /c".
            对 Unix 系统来说，等同于 -e "bash -c"。
            举例：如果你想用 Powershell 7 运行，可以输入 -e "pwsh -Command"

            但对于更复杂的指令，更推荐直接以文件方式运行脚本。
            比如，在后台设置命令为 "pwsh -File C:\example.ps1"，而不设置 -e 参数。
            
            输入的 -e "文本" 将以空白符分割，第一项为可执行文件，其余项以及下发的命令都将直接作为参数输入。

```

