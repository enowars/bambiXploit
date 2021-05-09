use crate::{config::Config, stats::BambiStats};

use std::{
    io::{self, BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
    sync::{mpsc, Arc},
};

struct Submitter {
    connection: Option<Connection>,
}

struct Connection {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl Submitter {
    fn try_submit(&mut self, flag: &[u8], config: &Config) -> io::Result<()> {
        if self.connection.is_none() {
            self.reconnect(&config.flagbot_address)?
        }
        let connection = self.connection.as_mut().unwrap();

        connection.writer.write_all(&flag)?;
        connection.writer.write_all(&vec![0x0A])?;
        connection.writer.flush()?;

        let mut response = String::new();
        connection.reader.read_line(&mut response)?;

        Ok(())
    }

    fn reconnect(&mut self, address: &str) -> io::Result<()> {
        let connection = TcpStream::connect(address)?;

        self.connection = Some(Connection {
            reader: BufReader::new(connection.try_clone()?),
            writer: BufWriter::new(connection),
        });

        Ok(())
    }
}

pub fn submit_thread(
    receiver: mpsc::Receiver<Vec<u8>>,
    config: Arc<Config>,
    stats: Arc<BambiStats>,
) {
    let mut submitter = Submitter { connection: None };

    loop {
        let flag = receiver.recv().unwrap();
        loop {
            if submitter.try_submit(&flag, &config).is_ok() {
                stats.add_ok(1);
                break;
            } else {
                let _ = submitter.reconnect(&config.flagbot_address);
            }
        }
    }
}
