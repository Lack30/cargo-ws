/// 生成 vscode *.code-workspace，让 vscode 实现 Clion 类似的功能，直接在项目中浏览相关的依赖包代码。
/// 
/// 思路如下:
///  1. vscode 可以通过设置工作区支持打开多个目录，这样就可以将标准库和第三方库添加到工作区中。
///  2. rust 默认标准库保存在 $HOME/.rustup 目录中，通过 `rustup default` 确认默认的 toolchains，因
///     此标准库路径为，$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/
///  3. rust 第三方库保存在 $HOME/.cargo 目录中，ll $HOME/.cargo/registry/src/github.com-xx 中
///  4. $HOME/.cargo 中保存本机所有项目依赖包的缓存，所以还需要忽略无关的包，读取项目 Cargo.lock 文件，确
///     认当前项目的依赖包，将其他包记录到 "settings" > "files.exclude"
///  5. rust-analyzer 启动时会加载工作区所有的包，导致打开缓慢，设置 "settings" > "rust-analyzer.files.excludeDirs"
///     屏蔽非本项目的包。
/// 
/// code-workspace 格式:
/// {
///  "folders": [
///    {
///      "name": "",
///      "path": "."
///    },
///    {
///      "name": "Stdlib",
///      "path": "$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library"
///    },
///    {
///      "name": "External Libraries",
///      "path": "$HOME/.cargo/registry/src/github.com-1ecc6299db9ec823"
///    }
///  ],
///  "settings": {
///    "files.exclude": {
///         "clap-3.2.0",
///         ...
///    },
///    "rust-analyzer.files.excludeDirs": [
///      "$HOME/.cargo/registry/src/github.com-1ecc6299db9ec823",
///      "$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library"
///    ]
///  }
/// }
mod config;

use clap::Parser;
use config::{Cargo, CargoLock, Workspace};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command as OsCommand,
};

#[derive(Parser)]
#[clap(name = "cargo", bin_name = "cargo")]
enum App {
    Ws(Ws),
}

#[derive(clap::Args, Debug)]
#[clap(
    author,
    version,
    override_usage = "cargo ws [options]",
    about = "generate vscode workspace file"
)]
struct Ws {
    /// Name of the person to greet
    #[clap(short, long, value_parser, default_value = ".")]
    root: String,
}

fn generate(args: &Ws) {
    let cargo_path = Path::new(&args.root).join("Cargo.toml");
    let cargo_lock_path = Path::new(&args.root).join("Cargo.lock");

    // 读取项目 Cargo.toml 和 Cargo.lock 文件 获取项目依赖第三方包信息
    let cargo = Cargo::from_path(cargo_path).expect("Failed to parse Cargo.toml");
    let cargo_lock = CargoLock::from_path(cargo_lock_path).expect("Failed to parse Cargo.lock");

    let home = dirs::home_dir().expect("Failed to get current user home directory");

    let rustup_home = Path::new(&home).join(".rustup");
    if !rustup_home.exists() {
        println!("rustc rust be installed");
        return;
    }

    let output = OsCommand::new("rustup")
        .arg("default")
        .output()
        .expect("Failed to execute rustup");
    let result = String::from_utf8_lossy(output.stdout.as_slice()).to_string();
    let toolchain = result.split(" ").take(1).next().expect("Failed to parse rustup toolchain");
    let rustup = rustup_home
        .join("toolchains")
        .join(toolchain)
        .join("lib")
        .join("rustlib")
        .join("src")
        .join("rust")
        .join("library");

    // 确定 rust .cargo 路径
    let cargo_home = Path::new(&home).join(".cargo");
    if !cargo_home.exists() {
        println!("cargo not be installed");
        return;
    }
    let registry_src = fs::read_dir(cargo_home.join("registry").join("src").as_path())
        .expect("Failed to walk $HOME/.cargo");
    let mut registry = PathBuf::new();
    let registry_entry = registry_src.take(1).next();
    if let Some(result) = registry_entry {
        if let Ok(entry) = result {
            registry = entry.path();
        }
    }

    let ws = Workspace::from(rustup, registry, &cargo_lock).expect("Failed to create workspace");

    let name = match cargo.package {
        Some(ref pack) => pack.name.clone(),
        None => "cargo-ws".to_string(),
    };

    let path = name + ".code-workspace";
    ws.apply(path).expect("Failed to save workspace file");
}

fn main() {
    let app = App::parse();
    match app {
        App::Ws(args) => generate(&args),
    }
}
