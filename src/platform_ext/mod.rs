#[cfg(target_os = "windows")]
pub mod win32;
#[cfg(target_os = "windows")]
/// Application / context menu handling. Currently Win32 only. Also has parsing functions
pub mod menu;