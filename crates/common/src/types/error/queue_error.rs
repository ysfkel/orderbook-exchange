pub enum QueueError<T> {
    Full(T),
    Empty,
}
