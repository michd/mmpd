mod adapters;

pub use adapters::get_adapter;

#[derive(Debug)]
pub struct FocusedWindow {
    pub window_class: Vec<String>,
    pub window_name: String,
}

impl FocusedWindow {
    fn blank() -> FocusedWindow {
        return FocusedWindow {
            window_class: vec![],
            window_name: String::from(""),
        }
    }
}
