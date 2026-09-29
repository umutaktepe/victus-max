use std::path::Path;

pub fn get_asset_path(filename: &str) -> String {
    let victus_path = format!("/usr/share/victus-max/assets/{}", filename);
    if Path::new(&victus_path).exists() {
        return victus_path;
    }

    let system_path = format!("/usr/share/omen-space/assets/{}", filename);
    if Path::new(&system_path).exists() {
        return system_path;
    }
    
    let local_gui = format!("src/victus-max-gui/assets/{}", filename);
    if Path::new(&local_gui).exists() {
        return local_gui;
    }

    // Fallback for local development
    format!("assets/{}", filename)
}
