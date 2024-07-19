pub trait SharedChannel {
    type Message;

    fn write(&self, message: Self::Message) -> bool;
    fn read(&self) -> Option<Self::Message>;
}

impl SharedChannel for () {
    type Message = ();

    fn write(&self, message: Self::Message) -> bool { false }
    fn read(&self) -> Option<Self::Message> { None }
}
