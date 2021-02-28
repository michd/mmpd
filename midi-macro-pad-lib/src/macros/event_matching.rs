pub mod midi;

pub trait MatchChecker<T> {
    fn matches(&self, val: T) -> bool;
}

pub trait EventType<T> : MatchChecker<T> {
    fn get_type(&self) -> &str;
}