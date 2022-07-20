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
    usage = "cargo ws [options]",
    about = "generate vscode workspace file"
)]
struct Ws {
    /// Name of the person to greet
    #[clap(short, long, value_parser, default_value = ".")]
    root: String,
}

fn dep_handler(args: &Ws) {
    let cargo = Path::new(&args.root).join("Cargo.toml");
    let cargo_lock = Path::new(&args.root).join("Cargo.lock");

    // 读取项目 Cargo.toml 和 Cargo.lock 文件 获取项目依赖第三方包信息
    let cargo = Cargo::from_path(cargo).expect("Failed to parse Cargo.toml");
    let cargo_lock = CargoLock::from_path(cargo_lock).expect("Failed to parse Cargo.lock");

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
    let toolchain = result.split(" ").take(1).next();
    if toolchain.is_none() {
        println!("Failed to parse rustup toolchain");
        return;
    }
    let rustup = rustup_home
        .join("toolchains")
        .join(toolchain.unwrap().to_string())
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
        App::Ws(args) => dep_handler(&args),
    }
}
