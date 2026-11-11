use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    #[cfg(feature = "desktop")]
    tauri_build::build();

    println!("cargo:rerun-if-changed=app.ico");
    println!("cargo:rerun-if-changed=build.rs");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let icon_path = manifest_dir.join("app.ico");
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

        if icon_path.exists() {
            let rc_path = out_dir.join("app.rc");
            let res_path = out_dir.join("app.res");

            // Format path with forward slashes for RC file
            let icon_str = icon_path.to_string_lossy().replace('\\', "/");
            let rc_content = format!("1 ICON \"{icon_str}\"\n");
            if fs::write(&rc_path, rc_content).is_ok() {
                let windres = env::var("WINDRES").unwrap_or_else(|_| {
                    let target = env::var("TARGET").unwrap_or_default();
                    if target.contains("x86_64") && target.contains("gnu") {
                        if which_cmd("x86_64-w64-mingw32-windres") {
                            "x86_64-w64-mingw32-windres".to_string()
                        } else {
                            "windres".to_string()
                        }
                    } else {
                        "windres".to_string()
                    }
                });

                let status = Command::new(&windres)
                    .args(["-i"])
                    .arg(&rc_path)
                    .args(["-O", "coff", "-o"])
                    .arg(&res_path)
                    .status();

                match status {
                    Ok(s) if s.success() => {
                        println!("cargo:rustc-link-arg={}", res_path.display());
                    }
                    Ok(s) => {
                        eprintln!("cargo:warning=windres failed with exit code: {s}");
                    }
                    Err(e) => {
                        eprintln!("cargo:warning=windres executable not found ({windres}): {e}");
                    }
                }
            }
        }
    }
}

fn which_cmd(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
