use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

pub struct Events {
    rx: Receiver<KeyEvent>,
    // Need to be kept around to prevent disposing the sender side.
    _tx: Sender<KeyEvent>,
}

impl Events {
    pub fn new(tick_rate: Duration) -> Events {
        let (tx, rx) = channel();

        let event_tx = tx.clone(); // why clone??
        thread::spawn(move || loop {
            if crossterm::event::poll(tick_rate).unwrap() {
                if let event::Event::Key(key) = event::read().unwrap() {
                    event_tx.send(key).unwrap();
                }
            }
        });

        Events { rx, _tx: tx }
    }

    /// Attempts to read an event.
    /// This function block the current thread.
    pub fn next(&self) -> Result<KeyEvent, TryRecvError> {
        self.rx.try_recv()
    }
}
