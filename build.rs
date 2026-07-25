// PezMax-One build script
// Compiles Windows resource files (.rc) to embed the app icon into the executable

fn main() {
    // Only compile resources on Windows
    #[cfg(target_os = "windows")]
    {
        let _ = embed_resource::compile("build/windows/icon.rc", embed_resource::NONE);
    }
}