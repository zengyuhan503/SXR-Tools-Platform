// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex as SyncMutex};
use std::{env, thread};
use tauri::Manager;
use tokio::sync::Mutex;

mod logger;
use logger::init_logging;

mod uese_exe_runner;
use uese_exe_runner::{ExeRunner, FileContentActions, InstallApps};

lazy_static! {
    static ref RUN_APP: Arc<Mutex<Vec<ExeRunner>>> = Arc::new(Mutex::new(Vec::new()));
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstallApp {
    path: String,
    exe_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Apps {
    name: String,
    version: String,
    path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Actions {
    name: String,
    item: Option<String>,
    orther: Option<String>,
}

/// 根据给定路径创建应用工具的哈希值
/// 
/// 该函数主要用于从文件路径中提取出最末尾的文件名或文件夹名，并以此为基础生成一个唯一的哈希值
/// 这在某些情况下可用于快速比较或查找特定的应用工具，而无需进行完整的路径比较
/// 
/// # 参数
/// - `path`: &str - 应用工具的文件或文件夹路径
/// 
/// # 返回值
/// - `u64` - 生成的哈希值
fn create_app_tool_hash(path: &str) -> u64 {
    // 提取路径中的最后部分，通常是文件名或文件夹名
    let tag = path.split("\\").last().unwrap();
    // 创建一个新的默认哈希器
    let mut hasher = DefaultHasher::new();
    // 将tag哈希到哈希器中
    tag.hash(&mut hasher);
    // 完成哈希计算并返回结果
    hasher.finish()
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_win = app.get_window("SXRTools").unwrap();
            let listen_win = app_win.clone();
            let apphandle = app.handle();
            listen_win.listen("run_actions", move |_e| {
                let emit_win = app_win.clone();
                let handles = apphandle.clone();
                // let apphandle = app.handle().clone();
                tokio::spawn(async move {
                    if let Some(data) = _e.clone().payload() {
                        let params: Actions = serde_json::from_str(data).expect("No json");
                        let actison = params.name.clone();
                        let install_apps = InstallApps::new(handles.clone());

                        match actison.as_str() {
                            "exit" => {
                                println!("exit");
                                let runners = RUN_APP.lock().await;
                                for runner in runners.iter() {
                                    runner.stop().await;
                                }
                                thread::sleep(std::time::Duration::from_secs(1));
                                handles.exit(0)
                            }
                            "init" => {
                                let _ = emit_win.emit("run_actions_res", install_apps.clone());
                            }
                            "install" => {
                                let exe_name = params.item.clone().unwrap();
                                let install_dir = params.orther.unwrap();
                                let exe_path = install_apps.apps_resources.join(&exe_name);
                                let res = install_apps
                                    .run_install_app(exe_name, exe_path, install_dir)
                                    .await;
                                let _ = emit_win.emit("run_actions_res", res);
                            }
                            "open_exe" => {
                                let path = params.item.unwrap();
                                let name = params.orther.unwrap();
                                let arc_win = Arc::new(SyncMutex::new(emit_win.clone()));
                                let mut runners = RUN_APP.lock().await;
                                let hashs = create_app_tool_hash(&path.clone());
                                let mut runer = ExeRunner::new(hashs);
                                let res = runer.start(&path, arc_win, name.clone()).await;
                                if let Ok(()) = res {
                                    runners.push(runer);
                                } else {
                                    let err = res.clone().err().unwrap();
                                    println!("{}", err);
                                    if err.contains("当前路径不存在，请重新安装") {
                                        let path_json = install_apps.install_json.clone();
                                        let res = FileContentActions::remove_content_form_file(
                                            name.clone(),
                                            path_json,
                                        );
                                        println!("res: {:?}", res);
                                    }
                                }

                                let _ = emit_win.emit("run_actions_res", res);
                            }
                            "stop_exe" => {
                                let path = params.item.unwrap();
                                let hashs = create_app_tool_hash(&path);
                                let mut runners = RUN_APP.lock().await;
                                for runner in runners.iter_mut() {
                                    if runner.tag == hashs {
                                        runner.stop().await;
                                    }
                                }
                            }
                            "uninstall" => {
                                let path = params.item.unwrap();
                                let name = params.orther.unwrap();
                                println!("uninstall: {}", path);
                                let res = install_apps.run_un_install(&path, name).await;
                                let _ = emit_win.emit("run_actions_res", res);
                            }
                            "set_logs" => {
                                let level = params.item.unwrap();
                                let message = params.orther.unwrap();

                                match level.as_str() {
                                    "info" => log::info!("{}", message),
                                    "error" => log::error!("{}", message),
                                    "warn" => log::warn!("{}", message),
                                    "debug" => log::debug!("{}", message),
                                    _ => log::trace!("{}", message),
                                }
                            }
                            _ => {
                                println!("Unknown action");
                            }
                        }
                    };
                });
            });
            init_logging();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
