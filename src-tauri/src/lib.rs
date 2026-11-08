#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            eprintln!("CatCut Desktop GUI initialized.");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running CatCut application");
}
