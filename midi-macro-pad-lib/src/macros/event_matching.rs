mod midi;

trait MatchChecker<T> {
    fn matches(&self, val: T) -> bool;
}

trait EventType<T> : MatchChecker<T> {
    fn get_type(&self) -> &str;
}