use std::{
    io::stdin,
    sync::{mpsc, Mutex},
    thread::{sleep, spawn, JoinHandle},
    time::Duration,
};

use lazy_static::lazy_static;
use termion::{event::Key, input::TermRead};

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    tx: mpsc::Sender<Event<Key>>,
    tick_handle: JoinHandle<()>,
    input_handle: JoinHandle<()>,
    exploiter_handles: Vec<JoinHandle<()>>,
}

struct EventConfig {
    tick_rate: Duration,
}

lazy_static! {
    static ref EVENT_CONFIG: EventConfig = EventConfig {
        tick_rate: Duration::from_millis(250),
    };
    pub static ref EVENTS: Mutex<Events> = Mutex::new(Events::new());
}

impl Events {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Event<Key>>();

        let sender = tx.clone();
        let tick_handle = spawn(move || loop {
            std::thread::sleep(EVENT_CONFIG.tick_rate);
            if let Err(e) = sender.send(Event::Tick) {
                // TODO: log eror somewhere
                break;
            }
        });

        let sender = tx.clone();
        let input_handle = spawn(move || {
            let stdin = std::io::stdin();
            let stdin = stdin.lock();
            for key in stdin.keys() {
                if let Ok(key) = key {
                    if let Err(e) = sender.send(Event::Input(key)) {
                        // TODO: log error somewhere
                        break;
                    }
                }
            }
        });

        Self {
            rx,
            tx,
            input_handle,
            tick_handle,
            exploiter_handles: Vec::new(),
        }
    }
}

impl Iterator for Events {
    type Item = Event<Key>;

    fn next(&mut self) -> Option<Self::Item> {
        self.rx.recv().ok()
    }
}
