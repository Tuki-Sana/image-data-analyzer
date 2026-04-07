mod analyze;
mod color_theory;
mod harmony;
mod meta;
mod palette_match;
mod shape_analysis;
mod theory;

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PixelSample {
    r: u8,
    g: u8,
    b: u8,
    hex: String,
}

#[tauri::command]
fn analyze_image(path: String) -> Result<analyze::Analysis, String> {
    analyze::analyze_path(&path)
}

#[tauri::command]
fn sample_pixel(path: String, x: u32, y: u32) -> Result<Option<PixelSample>, String> {
    Ok(analyze::sample_pixel(&path, x, y)?.map(|(r, g, b)| PixelSample {
        r,
        g,
        b,
        hex: format!("#{:02X}{:02X}{:02X}", r, g, b),
    }))
}

#[tauri::command]
fn save_text_file(path: String, contents: String) -> Result<(), String> {
    std::fs::write(path, contents).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_binary_file(path: String, contents: Vec<u8>) -> Result<(), String> {
    std::fs::write(path, contents).map_err(|e| e.to_string())
}

#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

#[tauri::command]
fn analyze_shape(path: String) -> Result<shape_analysis::ShapeAnalysisDto, String> {
    shape_analysis::analyze_shape_path(&path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            analyze_image,
            sample_pixel,
            save_text_file,
            save_binary_file,
            read_text_file,
            analyze_shape,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
