[中文文档](https://github.com/lack-io/cargo-ws/blob/main/README_CN.md)

# summary
[cargo-ws](https://github.com/lack-io/cargo-ws) is light-weight cargo plugin. It can generate vscode `code-workspace` file.  Then you can view file that be opened, like Jetbrains Idea.

# install
```bash
cargo install cargo-ws
```

# generate vscode workspace file
```bash
cargo new foo
cd foo
cargo check
cargo ws
```

# show

![image](https://raw.githubusercontent.com/lack-io/cargo-ws/main/images/image.png)