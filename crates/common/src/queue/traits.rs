pub trait QueueProducer<T> {
    type Error;
    fn push(&mut self, item: T) -> Result<(), Self::Error>;
}

pub trait  QueueConsumer<T> {
    type Error;
    fn pop(&mut self) -> Result<T, Self::Error>;
}