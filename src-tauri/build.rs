use std::path::PathBuf;

fn main() {
    configure_existing_sidecars();
    sync_dev_binaries();
    tauri_build::build()
}

fn configure_existing_sidecars() {
    let Ok(target) = std::env::var("TARGET") else {
        return;
    };

    let manifest_dir = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()),
    );
    let extension = if cfg!(windows) { ".exe" } else { "" };
    let mut external_bin = Vec::new();
    let mut resources = Vec::new();

    for name in ["adb", "scrcpy"] {
        let binary = manifest_dir
            .join("binaries")
            .join(format!("{name}-{target}{extension}"));
        if binary.is_file() {
            external_bin.push(format!("\"binaries/{name}\""));
            println!("cargo:rerun-if-changed={}", binary.display());
        }
    }

    if let Ok(entries) = std::fs::read_dir(manifest_dir.join("binaries")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if name.eq_ignore_ascii_case("README.md") {
                continue;
            }
            resources.push(format!("\"binaries/{name}\""));
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    if external_bin.is_empty() && resources.is_empty() {
        println!("cargo:rerun-if-changed={}", manifest_dir.join("binaries").display());
        return;
    }

    let mut fields = Vec::new();
    if !external_bin.is_empty() {
        fields.push(format!("\"externalBin\":[{}]", external_bin.join(",")));
    }
    if !resources.is_empty() {
        fields.push(format!("\"resources\":[{}]", resources.join(",")));
    }

    std::env::set_var(
        "TAURI_CONFIG",
        format!("{{\"bundle\":{{{}}}}}", fields.join(",")),
    );
}

fn sync_dev_binaries() {
    let manifest_dir = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()),
    );
    let source_dir = manifest_dir.join("binaries");
    if !source_dir.is_dir() {
        return;
    }

    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("target"));
    let output_dir = target_dir.join(profile);

    if let Err(error) = std::fs::create_dir_all(&output_dir) {
        println!("cargo:warning=failed to create binary output dir: {error}");
        return;
    }

    let Ok(entries) = std::fs::read_dir(&source_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let source = entry.path();
        if !source.is_file() {
            continue;
        }
        let Some(name) = source.file_name() else {
            continue;
        };
        if name.to_string_lossy().eq_ignore_ascii_case("README.md") {
            continue;
        }

        let dest = output_dir.join(name);
        if let Err(error) = std::fs::copy(&source, &dest) {
            println!(
                "cargo:warning=failed to copy {} to {}: {error}",
                source.display(),
                dest.display()
            );
        }
    }
}
