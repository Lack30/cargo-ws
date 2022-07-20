use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{Ok, Result};
use clap::Parser;
use serde_derive::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[clap(
    name = "cargo dep",
    author = "Lack",
    version = "0.1.0",
    usage = "cargo dep [options]",
    about = "cargo plugin",
    long_about = "generate vscode workspace file"
)]
enum App {
    Dep(Dep),
}

#[derive(clap::Args, Debug)]
struct Dep {
    /// Name of the person to greet
    #[clap(short, long, value_parser, default_value = ".")]
    root: String,
}

#[derive(Debug, Deserialize)]
struct Cargo {
    package: Option<Package>,
}

#[derive(Debug, Deserialize)]
struct CargoLock {
    package: Option<Vec<Package>>,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Workspace {
    #[serde(skip)]
    name: String,
    folders: Option<Vec<WorkspaceFolder>>,
    settings: Option<WorkspaceSettings>,
}

impl Workspace {
    fn from(cargo: &Cargo, lock: &CargoLock) -> Result<Workspace> {
        let mut folders: Vec<WorkspaceFolder> = Vec::new();
        let mut deps = HashMap::new();
        let mut name = String::new();
        let home = std::env::var("HOME")?;
        let cargo_home = Path::new(&home).join(".cargo");
        if !cargo_home.exists() {
            anyhow::bail!("cargo not be installed");
        }
        let src = cargo_home.join("registry").join("src");
        let dir = fs::read_dir(src.as_path())?;
        let mut root = PathBuf::new();
        for file in dir {
            if file.is_ok() {
                root = file.unwrap().path();
                break;
            }
        }

        if root.exists() {
            name = match cargo.package.as_ref() {
                Some(v) => v.name.clone(),
                None => "dep".to_string(),
            };

            if let Some(packages) = lock.package.as_ref() {
                for pack in packages {
                    let pack_name = pack.name.clone() + "-" + pack.version.as_str();
                    deps.insert(pack_name, ());
                }
            }
        }

        let mut file_excludes = HashMap::new();
        let mut rust_exclude_dirs = Vec::new();

        let root_string = root.clone().into_os_string().into_string().unwrap();
        for p in fs::read_dir(&root)? {
            let entry = p.unwrap();
            let file_name = entry.file_name().into_string().unwrap();
            if !deps.contains_key(&file_name) {
                file_excludes.insert(file_name.clone(), true);
            }
        }

        rust_exclude_dirs.push(root_string.clone());
        folders.push(WorkspaceFolder {
            name: "".to_string(),
            path: ".".to_string(),
        });
        folders.push(WorkspaceFolder {
            name: "External Library".to_string(),
            path: root_string.clone(),
        });

        let settings = WorkspaceSettings {
            file_excludes: Some(file_excludes),
            rust_exclude_dirs: Some(rust_exclude_dirs),
        };
        let ws = Workspace {
            name,
            folders: Some(folders),
            settings: Some(settings),
        };

        Ok(ws)
    }

    fn apply(&self) -> Result<()> {
        let text = serde_json::to_string_pretty(&self)?;
        fs::write(self.name.to_string() + ".code-workspace", text)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceFolder {
    name: String,
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceSettings {
    #[serde(rename = "files.exclude")]
    file_excludes: Option<HashMap<String, bool>>,
    #[serde(rename = "rust-analyzer.files.excludeDirs")]
    rust_exclude_dirs: Option<Vec<String>>,
}

fn dep_handler(args: &Dep) {
    let cargo_lock = Path::new(&args.root).join("Cargo.lock");
    let cargo = Path::new(&args.root).join("Cargo.toml");
    let mut cargo_lock_fd = File::open(cargo_lock).expect("open Cargo.lock");
    let mut cargo_fd = File::open(cargo).expect("open Cargo.toml");

    let mut cargo_lock_buf = String::new();
    cargo_lock_fd
        .read_to_string(&mut cargo_lock_buf)
        .expect("read Cargo.lock");

    let mut cargo_buf = String::new();
    cargo_fd
        .read_to_string(&mut cargo_buf)
        .expect("read Cargo.toml");

    let cargo_lock_toml: CargoLock = toml::from_str(&cargo_lock_buf).expect("decode Cargo.lock");
    let cargo_toml: Cargo = toml::from_str(&cargo_buf).expect("decode Cargo.toml");

    let ws = Workspace::from(&cargo_toml, &cargo_lock_toml).expect("create workspace");

    ws.apply().expect("save workspace file");
}

fn main() {
    let app = App::parse();
    match app {
        App::Dep(args) => dep_handler(&args),
    }
}
