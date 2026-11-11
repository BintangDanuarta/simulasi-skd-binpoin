#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

fn start_backend(root: PathBuf) -> Result<(Child, String), String> {
    #[cfg(target_os = "windows")]
    let backend = root.join("backend/simulasi-skd-binpoin.exe");
    #[cfg(not(target_os = "windows"))]
    let backend = root.join("backend/simulasi-skd-binpoin");
    if !backend.is_file() {
        return Err(format!("Backend tidak ditemukan: {}", backend.display()));
    }

    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Tidak bisa memilih port lokal: {e}"))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    drop(listener);

    let child = Command::new(backend)
        .current_dir(&root)
        .env("BIND", format!("127.0.0.1:{port}"))
        .env("CPNS_DESKTOP_SHELL", "1")
        .env("CPNS_APP_DIR", &root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Gagal menjalankan backend lokal: {e}"))?;

    let url = format!("http://127.0.0.1:{port}/");
    for _ in 0..100 {
        if TcpListener::bind(("127.0.0.1", port)).is_err() {
            return Ok((child, url));
        }
        thread::sleep(Duration::from_millis(50));
    }

    Err("Backend lokal tidak mulai dalam waktu yang diharapkan".into())
}

fn main() {
    let app = tauri::Builder::default()
        .setup(move |app| {
            let root = app.path().resource_dir()?;
            let (backend, url) = start_backend(root.clone())
                .map_err(std::io::Error::other)?;
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url.parse().unwrap()))
                .title("Simulasi SKD by Binpoin")
                .inner_size(1180.0, 820.0)
                .min_inner_size(760.0, 600.0)
                .build()?;
            app.manage(std::sync::Mutex::new(Some(backend)));
            app.handle().plugin(tauri_plugin_updater::Builder::new().build())?;
            app.handle().plugin(tauri_plugin_process::init())?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Gagal membuat jendela desktop");

    app.run(|app, event| {
        if let tauri::RunEvent::Exit = event {
            if let Some(child) = app
                .try_state::<std::sync::Mutex<Option<Child>>>()
                .and_then(|state| state.lock().ok()?.take())
            {
                let mut child = child;
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    });
}