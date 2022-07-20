use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::Path,
};

use anyhow::{Ok, Result};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Cargo {
    pub package: Option<Package>,
}

impl Cargo {
    /// 读取 Cargo.toml 解析成 Cargo
    ///
    /// # Example
    ///
    /// ```no_run
    /// mod config
    ///
    /// use std::path::Path;
    /// use anyhow::{Ok, Result}
    /// use config::Cargo
    ///
    /// fn main() -> Result<()> {
    ///     let path = Path::new("Cargo.toml");
    ///     let cargo = Cargo::from_path(path)?;    
    ///         
    ///     OK(())
    /// }
    /// ```
    pub fn from_path<P>(path: P) -> Result<Cargo>
    where
        P: AsRef<Path>,
    {
        let mut fd = File::open(path).expect("open Cargo.toml");

        let mut buf = String::new();
        fd.read_to_string(&mut buf)?;

        let cargo: Cargo = toml::from_str(&buf)?;

        Ok(cargo)
    }
}

#[derive(Debug, Deserialize)]
pub struct CargoLock {
    pub package: Option<Vec<Package>>,
}

impl CargoLock {
    /// 读取 Cargo.lock 解析成 CargoLock
    ///
    /// # Example
    ///
    /// ```no_run
    /// mod config
    ///
    /// use std::path::Path;
    /// use anyhow::{Ok, Result}
    /// use config::CargoLock
    ///
    /// fn main() -> Result<()> {
    ///     let path = Path::new("Cargo.lock");
    ///     let cargo_lock = CargoLock::from_path(path)?;    
    ///         
    ///     OK(())
    /// }
    /// ```
    pub fn from_path<P>(path: P) -> Result<CargoLock>
    where
        P: AsRef<Path>,
    {
        let mut fd = File::open(path)?;

        let mut buf = String::new();
        fd.read_to_string(&mut buf)?;

        let cargo_lock: CargoLock = toml::from_str(&buf)?;

        Ok(cargo_lock)
    }
}

#[derive(Debug, Deserialize)]
pub struct Package {
    // rust 第三方包名称
    pub name: String,
    // rust 第三方包版本
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Workspace {
    // 生成 code-workspace 中的 "folder" 配置
    pub folders: Option<Vec<WorkspaceFolder>>,
    // 生成 code-workspace 中的 "settings" 配置
    pub settings: Option<WorkspaceSettings>,
}

impl Workspace {
    /// # Example
    /// ```no_run
    /// mod config
    ///
    /// use std::path::{Path, PathBuf};
    /// use anyhow::{Ok, Result}
    /// use config::{CargoLock, Workspace}
    ///
    /// fn main() -> Result<()> {
    ///     let path = Path::new("Cargo.lock");
    ///     let cargo_lock = CargoLock::from_path(path)?;
    ///     
    ///     let rustup = PathBuf::from_str(rustup_path);
    ///     let registry = PathBuf::from_str(registry_path);    
    ///     
    ///     let ws = Workspace::from(rustup, registry, &cargo_lock)?;        
    ///
    ///     OK(())
    /// }
    /// ```
    pub fn from<P>(rustup: P, registry: P, lock: &CargoLock) -> Result<Workspace>
    where
        P: AsRef<Path>,
    {
        let mut folders: Vec<WorkspaceFolder> = Vec::new();
        let mut deps = HashMap::new();
        let mut file_excludes = HashMap::new();
        let mut rust_exclude_dirs = Vec::new();

        if registry.as_ref().exists() {
            if let Some(ref packages) = lock.package {
                for pack in packages {
                    let pack_name = pack.name.clone() + "-" + pack.version.as_str();
                    deps.insert(pack_name, ());
                }
            }

            let rustup_string = rustup
                .as_ref()
                .to_path_buf()
                .clone()
                .into_os_string()
                .into_string()
                .unwrap();

            let registry_string = registry
                .as_ref()
                .to_path_buf()
                .clone()
                .into_os_string()
                .into_string()
                .unwrap();
            for p in fs::read_dir(registry.as_ref())? {
                let entry = p.unwrap();
                let file_name = entry.file_name().into_string().unwrap();
                if !deps.contains_key(&file_name) {
                    file_excludes.insert(file_name.clone(), true);
                }
            }

            rust_exclude_dirs.push(registry_string.clone());
            rust_exclude_dirs.push(rustup_string.clone());
            folders.push(WorkspaceFolder {
                name: "".to_string(),
                path: ".".to_string(),
            });
            folders.push(WorkspaceFolder {
                name: "Stdlib".to_string(),
                path: rustup_string.clone(),
            });
            folders.push(WorkspaceFolder {
                name: "External Libraries".to_string(),
                path: registry_string.clone(),
            });
        }

        let settings = WorkspaceSettings {
            file_excludes: Some(file_excludes),
            rust_exclude_dirs: Some(rust_exclude_dirs),
        };
        let ws = Workspace {
            folders: Some(folders),
            settings: Some(settings),
        };

        Ok(ws)
    }

    /// # Example
    /// ```no_run
    /// mod config
    ///
    /// use std::path::{Path, PathBuf};
    /// use anyhow::{Ok, Result}
    /// use config::{CargoLock, Workspace}
    ///
    /// fn main() -> Result<()> {
    ///     let path = Path::new("Cargo.lock");
    ///     let cargo_lock = CargoLock::from_path(path)?;
    ///     
    ///     let rustup = PathBuf::from_str(rustup_path);
    ///     let registry = PathBuf::from_str(registry_path);    
    ///     
    ///     let ws = Workspace::from(rustup, registry, &cargo_lock)?;
    ///     let target = "simple.code-workspace";
    ///     ws.apply(target.to_string())?;      
    ///
    ///     OK(())
    /// }
    /// ```
    pub fn apply(&self, path: String) -> Result<()> {
        let text = serde_json::to_string_pretty(&self)?;
        fs::write(path, text)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceFolder {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    // 因为 vscode workspace 配置文件中不支持多级目录，
    // 如果要实现和 Clion 相同的功能，需要改变思路，先
    // 在 folders 中加载本地所有包，再使用 "files.exclude"
    // 忽略非本项目的其他包。
    #[serde(rename = "files.exclude")]
    file_excludes: Option<HashMap<String, bool>>,

    // 生成 rust-analyzer.files.excludeDirs 配置
    // 因为工作区中加载所有 .cargo 目录下的第三方包，
    // 会导致 rust-analyzer 在项目启动加载所有包，
    // 使用该配置告诉 rust-analyzer 忽略加载
    #[serde(rename = "rust-analyzer.files.excludeDirs")]
    rust_exclude_dirs: Option<Vec<String>>,
}

mod test {
    #[allow(unused)]
    use crate::config::{Cargo, CargoLock, Workspace};
    #[allow(unused)]
    use std::path::Path;

    #[test]
    fn test_from_cargo() {
        let path = Path::new("Cargo.toml");
        let cargo = Cargo::from_path(path).unwrap();

        assert!(cargo.package.is_some());
    }

    #[test]
    fn test_from_cargo_lock() {
        let path = Path::new("Cargo.lock");
        let cargo = CargoLock::from_path(path).unwrap();

        assert!(cargo.package.is_some());
    }

    #[test]
    fn test_from_workspace() {
        let rustup = Path::new("rustup").to_path_buf();
        let registry = Path::new("registry").to_path_buf();

        let path = Path::new("Cargo.lock");
        let cargo = CargoLock::from_path(path).unwrap();

        let ws = Workspace::from(rustup, registry, &cargo).unwrap();

        assert!(ws.folders.is_some());
        assert!(ws.settings.is_some());
    }
}
