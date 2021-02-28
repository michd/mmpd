mod adapters;

pub use adapters::get_adapter;

/// Container struct for window info
#[derive(Debug)]
pub struct FocusedWindow {
    pub window_class: Vec<String>,
    pub window_name: String,
}

impl FocusedWindow {
    /// Creates a blank instance
    fn blank() -> FocusedWindow {
        return FocusedWindow {
            window_class: vec![],
            window_name: String::from(""),
        }
    }
}
