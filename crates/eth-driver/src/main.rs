#![no_std]
#![no_main]
#![feature(never_type)]

#[main]
fn main() -> ThisHandler {
    todo!()
}

struct ThisHandler();

impl Handler for ThisHandler {
    type Error = !;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        todo!()
    }
}
