#[cfg(target_os = "linux")]
mod x11;

#[cfg(target_os = "linux")]
pub use self::x11::get_adapter;

/// Container struct for window info
#[derive(Debug)]
pub struct FocusedWindow {
    pub window_class: Vec<String>,
    pub window_name: String,
}

/// Adapters implementing this trait can be asked to provided data on the currently focused window.
pub trait FocusAdapter {
    /// Returns an instance of FocusedWindow with relevant focused window info (class, name) if
    /// available, None otherwise.
    fn get_focused_window(&self) -> Option<FocusedWindow>;
}