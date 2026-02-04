//! Генерация иконок и копирование логотипа: если в корне проекта есть logo.jpg,
//! копируем в public/ для сплеша и генерируем иконки в src-tauri/icons.

use std::path::Path;

fn main() {
    tauri_build::build();

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let project_root = match manifest_dir.parent() {
        Some(p) => p,
        None => return,
    };
    let logo_src = project_root.join("logo.jpg");
    if !logo_src.exists() {
        return;
    }

    let public_dir = project_root.join("public");
    let _ = std::fs::create_dir_all(&public_dir);
    let logo_dst = public_dir.join("logo.jpg");
    let _ = std::fs::copy(&logo_src, &logo_dst);

    let icons_dir = manifest_dir.join("icons");
    if let Ok(img) = image::open(&logo_src) {
        let rgba = img.to_rgba8();
        for (size, name) in [(32, "32x32.png"), (128, "128x128.png"), (256, "128x128@2x.png")] {
            let resized = image::imageops::resize(
                &rgba,
                size as u32,
                size as u32,
                image::imageops::FilterType::Triangle,
            );
            let path = icons_dir.join(name);
            let _ = resized.save(&path);
        }
    }
}
