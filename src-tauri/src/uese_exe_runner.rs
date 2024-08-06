use crate::NAME_MAP;
use dirs::document_dir;

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::PathBuf;
use std::process::Command as SysCommand;
use std::sync::{Arc, Mutex as SyncMutex};
use tauri::{AppHandle, Window};
use tokio::process::Command;
use tokio::sync::{Mutex, Notify};

extern crate winreg;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppInfo {
    name: String,
    version: String,
    path: String,
    is_start: bool,
    install_dir: String,
}

pub struct FileContentActions {}
impl FileContentActions {
    pub async fn write_to_file(info: AppInfo, json_path: PathBuf) -> Result<(), String> {
        let path = json_path.to_string_lossy().to_string();
        let file_content = fs::read_to_string(path.clone()).unwrap();
        let mut apps: Vec<AppInfo> = vec![];
        if !file_content.is_empty() {
            apps = serde_json::from_str(&file_content).unwrap();
        }
        apps.push(info);
        let json_data = serde_json::to_string_pretty(&apps);
        match json_data {
            Ok(data) => {
                let s = fs::write(path.clone(), data);
                return s.map_err(|err| format!("{}", err));
            }
            Err(err) => {
                return Err(format!("err:{}", err));
            }
        }
    }
    pub fn remove_content_form_file(app_name: String, path: PathBuf) -> Result<(), String> {
        let path = path.to_string_lossy().to_string();
        let file_content = fs::read_to_string(path.clone()).unwrap();
        let mut apps: Vec<AppInfo> = serde_json::from_str(&file_content).unwrap();
        apps.retain(|app| app.name != app_name);
        let json_data = serde_json::to_string_pretty(&apps);
        match json_data {
            Ok(data) => {
                let s = fs::write(path.clone(), data);
                return s.map_err(|err| format!("{}", err));
            }
            Err(err) => {
                return Err(format!("err:{}", err));
            }
        }
    }
}

#[derive(Debug)]
pub struct ExeRunner {
    pub child: Arc<Mutex<Option<tokio::process::Child>>>,
    pub notifier: Arc<Notify>,
    pub tag: u64,
}

impl ExeRunner {
    pub fn new(hashs: u64) -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
            notifier: Arc::new(Notify::new()),
            tag: hashs,
        }
    }
    pub fn get_run_path(exe_path: &str, is_run: bool) -> Result<String, String> {
        let entries = fs::read_dir(exe_path);
        let mut run_path = String::new();
        match entries {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let file_name = entry.file_name().to_string_lossy().into_owned();
                        if file_name.ends_with(".exe") {
                            if is_run {
                                if file_name != "uninstall.exe" {
                                    run_path = format!("{}\\{}", exe_path, file_name);
                                }
                            } else {
                                if file_name == "uninstall.exe" {
                                    run_path = format!("{}\\{}", exe_path, file_name);
                                }
                            }
                        }
                    }
                }
            }
            Err(err) => {
                return Err(format!("Error: {:?}", err));
            }
        }
        Ok(run_path)
    }
    pub async fn start(
        &mut self,
        exe_path: &str,
        arc_win: Arc<SyncMutex<Window>>,
        name: String,
    ) -> Result<(), String> {
        println!("exe_path:{:?}", exe_path);
        let run_exe = ExeRunner::get_run_path(exe_path, true)?;

        if run_exe.is_empty() {
            let err = format!("Error: {}", "当前路径不存在，请重新安装");
            return Err(err);
        }
        let child = Command::new(run_exe)
            .spawn()
            .expect("failed to start the exe file");

        {
            let mut locked_child = self.child.lock().await;
            *locked_child = Some(child);
        }

        let child_clone = self.child.clone();
        let notifier_clone = self.notifier.clone();

        tokio::spawn(async move {
            let run_name = name.clone();
            let mut locked_child = child_clone.lock().await;
            if let Some(child) = locked_child.as_mut() {
                tokio::select! {
                    result = child.wait() => {
                        match result {
                            Ok(_status) =>{
                                let _res =arc_win.lock().unwrap();
                                let _= _res.emit("run_close", run_name.clone());
                            },
                            Err(e) => println!("Failed to wait on child: {}", e),
                        }
                    }
                    _ = notifier_clone.notified() => {
                        let _=child.kill().await;
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) {
        self.notifier.notify_one();
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct InstallApps {
    pub apps: Vec<String>,
    pub contents: String,
    pub apps_resources: PathBuf,
    pub install_json: PathBuf,
}
impl InstallApps {
    pub fn new(app: AppHandle) -> Self {
        let resources: PathBuf = app
            .path_resolver()
            .resource_dir()
            .unwrap()
            .join("resources");
        let document_path = document_dir().unwrap();
        let mut doc_path = document_path.join("sxr_apps");
        let _ = fs::create_dir_all(doc_path.clone());
        let _file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(doc_path.clone().join("app.json"))
            .unwrap();
        doc_path = doc_path.join("app.json");
        let apps_resources: PathBuf = resources.join("apps");
        let install_apps: PathBuf = doc_path;
        let exe_resources: PathBuf = apps_resources.clone();
        let install_json = install_apps.clone();
        let apps = InstallApps::reade_apps(apps_resources);
        let contents = InstallApps::reade_file_contents(install_apps);
        Self {
            apps,
            contents,
            apps_resources: exe_resources,
            install_json,
        }
    }
    pub fn reade_apps(path: PathBuf) -> Vec<String> {
        let entries = fs::read_dir(path);
        let mut apps = Vec::new();
        match entries {
            Ok(entrie) => {
                for entry in entrie {
                    if let Ok(entry) = entry {
                        let file_name = entry.file_name().to_string_lossy().into_owned();
                        apps.push(file_name);
                    }
                }
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }
        };
        apps
    }
    pub fn reade_file_contents(path: PathBuf) -> String {
        let file_path = path.display().to_string();
        let mut contents = String::new();
        let mut content_file = File::open(file_path).unwrap();
        let _ = content_file.read_to_string(&mut contents);
        contents
    }
    pub async fn run_install_app(
        &self,
        exe_name: String,
        path: PathBuf,
        install_dir: String,
    ) -> Result<String, String> {
        let file_name = get_app_file_name(exe_name.clone());
        let dir_path = format!(r"{}\{}", install_dir.clone(), file_name.clone());
        let mut dir = format!("/D={}", dir_path);
        dir = dir.replace("\"", "");
        let _is_check = check_unstall_file_exists(install_dir.clone())?;

        println!("dir_path:{},path:{}", dir_path, install_dir);

        let exes: Vec<String> = exe_name.split("_").map(|s| s.to_string()).collect();
        let output = SysCommand::new(path).arg("/S").arg(&dir).output();
        match output {
            Ok(output) if output.status.success() => {
                let info = AppInfo {
                    name: exes[0].clone(),
                    version: exes[1].clone(),
                    path: dir_path.clone(),
                    is_start: false,
                    install_dir: exe_name.clone(),
                };
                let res = FileContentActions::write_to_file(info, self.install_json.clone()).await;
                if res.is_ok() {
                    return Ok(dir_path.clone());
                } else {
                    return Err(res.unwrap_err());
                }
            }
            Ok(output) => Err(String::from_utf8_lossy(&output.stderr).into_owned()),
            Err(e) => Err(e.to_string()),
        }
    }
    pub async fn run_un_install(&self, path: &str, name: String) -> Result<(), String> {
        let un_path = ExeRunner::get_run_path(path, false)?;
        if un_path.is_empty() {
            let _ = FileContentActions::remove_content_form_file(name, self.install_json.clone());
            return Err(String::from(
                "当前应用不存在，已删除记录，如需使用请重新安装",
            ));
        }
        let output = SysCommand::new(un_path).arg("/S").output();
        match output {
            Ok(output) if output.status.success() => {
                return FileContentActions::remove_content_form_file(
                    name,
                    self.install_json.clone(),
                );
            }
            Ok(output) => Err(String::from_utf8_lossy(&output.stderr).into_owned()),
            Err(e) => Err(e.to_string()),
        }
    }
}

fn get_app_file_name(exe_name: String) -> String {
    let parts: Vec<&str> = exe_name.split('_').collect();
    let name = parts[0];
    if NAME_MAP.contains_key(name) {
        let name = NAME_MAP.get(name).unwrap().to_string();
        return name;
    } else {
        let mut hasher = DefaultHasher::new();
        let inputs = name;
        inputs.hash(&mut hasher);
        let hash_value = hasher.finish();
        return format!("SXR_VQ920_Apps{}", hash_value);
    }
}
fn check_unstall_file_exists(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(path);
    let check_path = path_buf.join("uninstall.exe");
    if check_path.exists() {
        return Err("该目录下已经安装过应用，请安装到其他地址".to_string());
    } else {
        return Ok(());
    }
}
