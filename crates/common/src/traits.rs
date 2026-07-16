use crate::thread_handle::ThreadHandle;

pub trait ThreadHandler {
    fn start(self) -> ThreadHandle;
}
