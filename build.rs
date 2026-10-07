#[cfg(windows)]
fn main() {
    use image::{imageops, ImageFormat, RgbaImage};
    use std::{env, path::PathBuf};

    println!("cargo:rerun-if-changed=assets/doomsday-radio.png");
    println!("cargo:rerun-if-changed=Cargo.toml");

    let logo = image::open("assets/doomsday-radio.png")
        .expect("Station logo must be a valid PNG")
        .into_rgba8();
    let scaled = imageops::thumbnail(&logo, 256, 256);
    let mut icon = RgbaImage::new(256, 256);
    imageops::overlay(
        &mut icon,
        &scaled,
        i64::from((256 - scaled.width()) / 2),
        i64::from((256 - scaled.height()) / 2),
    );
    let icon_path = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory missing"))
        .join("doomsday-radio.ico");
    icon.save_with_format(&icon_path, ImageFormat::Ico)
        .expect("Windows icon could not be generated");

    let mut resource = winresource::WindowsResource::new();
    resource
        .set_icon(icon_path.to_str().expect("Icon path must be valid UTF-8"))
        .set("ProductName", "Doomsday Radio")
        .set("FileDescription", "Doomsday Radio Player")
        .set("OriginalFilename", "doomsday-radio.exe")
        .compile()
        .expect("Windows icon and version resources could not be compiled");
}

#[cfg(not(windows))]
fn main() {}
