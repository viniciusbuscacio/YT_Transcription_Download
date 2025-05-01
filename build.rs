#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set("FileDescription", "YouTube Transcription Downloader")
       .set("ProductName", "YT_Transcription_Downloader")
       .set("CompanyName", "https://github.com/viniciusbuscacio/yt_transcription_downloader")
       .set("LegalCopyright", "Copyright © 2025 Vinicius Buscacio")
       .set("OriginalFilename", "yt_transcription_downloader.exe")
       .set("ProductVersion", "1.0.0")
       .set("FileVersion", "1.0.0")
       .set("LegalTrademarks", "Licensed under MIT")
       .set_icon("app_icon.ico");

    res.compile().expect("Failed to compile Windows resources");
}

#[cfg(not(windows))]
fn main() {
    // Nothing to do for non-Windows platforms
}