[![build badge](https://github.com/lack-io/cargo-ws/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/lack-io/cargo-ws/actions/workflows/rust.yml/badge.svg)

# 简介
[cargo-ws](https://github.com/lack-io/cargo-ws) 是一个轻量级的 cargo 插件。你可以使用它来生成 vscode 工作区配置文件，这样就可以在工作区内浏览第三方库的代码，就像 JetBrains Idea。

# 安装
```bash
cargo install cargo-ws
```

# 生成配置文件
```bash
cargo new foo
cd foo
cargo check
cargo ws
```

# 最终效果

![image](https://raw.githubusercontent.com/lack-io/cargo-ws/main/images/image.png)