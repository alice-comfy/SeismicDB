use std::net::TcpStream;
use std::io::{Read, Write, Cursor};
use std::sync::mpsc::{Receiver, channel};
use byteorder::{BigEndian, ReadBytesExt};
use bufstream::BufStream;
use sdb_core::dtf::update::Update;
use crate::error::SeismicError;
use sdb_core::dtf::{update::UpdateVecConvert, file_format::decode_buffer};
use sdb_core::postprocessing::orderbook::Orderbook;

pub struct SeismicClient {
    pub stream: BufStream<TcpStream>,
    pub host: String,
    pub port: String,
}

impl SeismicClient {
    pub fn new(host: &str, port: &str) -> Result<SeismicClient, SeismicError> {
        let addr = format!("{}:{}", host, port);

        info!("Connecting to {}", addr);

        let stream = match TcpStream::connect(&addr) {
            Ok(stm) => stm,
            Err(_) => return Err(SeismicError::ConnectionError)
        };

        let reader_cap = 1024;
        let writer_cap = 1024;
        let stream = BufStream::with_capacities(reader_cap, writer_cap, stream);

        Ok(SeismicClient {
            stream,
            host: host.to_owned(),
            port: port.to_owned(),
        })
    }

    pub fn reconnect(&mut self) -> Result<(), SeismicError> {
        let addr = format!("{}:{}", self.host, self.port);
        info!("Reconnecting to {}", addr);
        self.stream = match TcpStream::connect(&addr) {
            Ok(stm) => BufStream::new(stm),
            Err(_) => return Err(SeismicError::ConnectionError)
        };
        Ok(())
    }

    pub fn cmd(&mut self, command: &str) -> Result<String, SeismicError> {
        self.stream.write(&(command.len() as u32).to_be_bytes())?;
        self.stream.write(command.as_bytes())?;
        self.stream.flush()?;

        let success = self.stream.read_u8()
            .map(|i| i == 0x1)
            .map_err(|_| SeismicError::ConnectionError)?;

        if command.starts_with("GET")
            && !command.contains("AS CSV")
            && !command.contains("AS JSON")
            && success
        {
            let size = self.stream.read_u64::<BigEndian>()?;
            let mut buf = vec![0_u8; size as usize];
            self.stream.read_exact(&mut buf)?;

            let mut buf = Cursor::new(buf.as_slice());
            let v = decode_buffer(&mut buf);
            Ok(format!("[{}]\n", v.as_json()))
        } else {
            let size = self.stream.read_u64::<BigEndian>()?;
            let mut buf = vec![0; size as usize];
            self.stream.read_exact(&mut buf)?;
            let res = std::str::from_utf8(&buf).unwrap().to_owned();
            if success {
                Ok(res)
            } else if res.starts_with("ERR: No db named") {
                let book_name = res.split(" ").nth(4).unwrap();
                Err(SeismicError::DBNotFoundError(book_name.to_owned()))
            } else  {
                Err(SeismicError::ServerError(res))
            }
        }
    }

    unsafe fn cmd_bytes_no_check(&mut self, command: &[u8], discard_result: bool) -> Result<bool, SeismicError> {
        self.stream.write(&(command.len() as u32).to_be_bytes())?;
        self.stream.write(command)?;
        self.stream.flush()?;
        if !discard_result {
            let _ret = self.stream.read_u8().map(|i| i == 0x1)?;
            let size = self.stream.read_u64::<BigEndian>()?;
            // ignore bytes
            std::io::copy(
                &mut self.stream.get_ref().take(size as u64),
                &mut std::io::sink()
            )?;
        }
        Ok(true)
    }

    pub fn create_db(&mut self, book_name: &str) -> Result<String, SeismicError> {
        info!("Creating db {}", book_name);
        self.cmd(&format!("CREATE {}\n", book_name))
    }

    pub fn use_db(&mut self, book_name: &str) -> Result<String, SeismicError> {
        self.cmd(&format!("USE {}\n", book_name))
    }

    pub fn orderbook_snapshot(&mut self, book_name: &str) -> Result<Orderbook, SeismicError> {
        let ob_json_str = self.cmd(&format!("OB {}\n", book_name))?;
        let ob = serde_json::from_str::<Orderbook>(&ob_json_str).map_err(|_e| SeismicError::JsonError)?;
        Ok(ob)
    }

    pub fn subscribe(mut self, book_name: &str) -> Result<Receiver<Update>, SeismicError> {
        self.cmd(&format!("SUBSCRIBE {}\n", book_name))?;

        let (tx, rx) = channel();

        std::thread::spawn(move || {
            loop {
                let success = self.stream.read_u8()
                    .map(|i| i == 0x1)
                    .map_err(|_| SeismicError::ConnectionError).unwrap();

                if !success { break }

                let size = self.stream.read_u64::<BigEndian>().unwrap();
                let mut buf = vec![0; size as usize];
                self.stream.read_exact(&mut buf).unwrap();
                let decoded = sdb_core::utils::decode_insert_into(&buf);
                match decoded {
                    Some((Some(up), Some(_book_name))) => tx.send(up).unwrap(),
                    e => {
                        println!("{:#?}", e);
                        ()
                    }
                }
            }
        });

        Ok(rx)
    }

    #[deprecated]
    pub fn insert_text(&mut self, book_name: String, update: &Update) -> Result<String, SeismicError> {
        let is_trade = if update.is_trade {"t"} else {"f"};
        let is_bid = if update.is_bid {"t"} else {"f"};
        let cmdstr = format!("ADD {}, {}, {}, {}, {}, {}; INTO {}\n",
                        update.ts, update.seq, is_trade, is_bid, update.price, update.size, book_name);
        self.cmd(&cmdstr)
    }

    /// you can achieve very high throughput by setting discard_result to true
    /// send an insert command without reading the output from tdb server
    pub fn insert(&mut self, book_name: Option<&str>, update: &Update, discard_result: bool) -> Result<bool, SeismicError> {
        let buf = sdb_core::utils::encode_insert_into(book_name, update)?;
        unsafe { self.cmd_bytes_no_check(&buf, discard_result) }
    }

    pub fn shutdown(self) {
        self.stream.into_inner().unwrap().shutdown(std::net::Shutdown::Both).unwrap()
    }
}
