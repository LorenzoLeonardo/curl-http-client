use std::fmt::Debug;
use std::io::Read;
use std::time::Instant;
use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    path::PathBuf,
};

use curl::easy::{Handler, ReadError, WriteError};
use http::{HeaderMap, HeaderName, HeaderValue};
use log::trace;
use tokio::sync::mpsc::Sender;

/// This is an information about the transfer(Download/Upload) speed that will be sent across tasks.
/// It is useful to get the transfer speed and displayed it according to
/// user's application.
#[derive(Clone, Debug)]
pub struct TransferSpeed(f64);

impl TransferSpeed {
    pub fn as_bytes_per_sec(&self) -> u64 {
        self.0 as u64
    }
}

impl From<u64> for TransferSpeed {
    fn from(value: u64) -> Self {
        Self(value as f64)
    }
}

impl From<usize> for TransferSpeed {
    fn from(value: usize) -> Self {
        Self(value as f64)
    }
}

impl From<i32> for TransferSpeed {
    fn from(value: i32) -> Self {
        Self(value as f64)
    }
}

impl From<i64> for TransferSpeed {
    fn from(value: i64) -> Self {
        Self(value as f64)
    }
}

impl From<f64> for TransferSpeed {
    fn from(value: f64) -> Self {
        Self(value)
    }
}
/// Stores the path for the downloaded file or the uploaded file.
/// Internally it will also monitor the bytes transferred and the Download/Upload speed.
#[derive(Clone, Debug)]
pub struct FileInfo {
    /// File path to download or file path of the source file to be uploaded.
    pub path: PathBuf,
    /// Sends the transfer speed information via channel to another task.
    /// This is an optional parameter depends on the user application.
    send_speed_info: Option<Sender<TransferSpeed>>,
    bytes_transferred: usize,
    transfer_started: Instant,
    transfer_speed: TransferSpeed,
}

impl FileInfo {
    /// Sets the destination file path to download or file path of the source file to be uploaded.
    pub fn path(path: PathBuf) -> Self {
        Self {
            path,
            send_speed_info: None,
            bytes_transferred: 0,
            transfer_started: Instant::now(),
            transfer_speed: TransferSpeed::from(0),
        }
    }

    /// Sets the FileInfo struct with a message passing channel to send transfer speed information across user applications.
    /// It uses a tokio bounded channel to send the information across tasks.
    pub fn with_transfer_speed_sender(mut self, send_speed_info: Sender<TransferSpeed>) -> Self {
        self.send_speed_info = Some(send_speed_info);
        self
    }

    fn update_bytes_transferred(&mut self, transferred: usize) {
        self.bytes_transferred += transferred;

        let now = Instant::now();
        let difference = now.duration_since(self.transfer_started);

        self.transfer_speed =
            TransferSpeed::from((self.bytes_transferred) as f64 / difference.as_secs_f64());
    }

    fn bytes_transferred(&self) -> usize {
        self.bytes_transferred
    }

    fn transfer_speed(&self) -> TransferSpeed {
        self.transfer_speed.clone()
    }
}

fn send_transfer_info(info: &FileInfo) {
    if let Some(tx) = info.send_speed_info.clone() {
        let transfer_speed = info.transfer_speed();
        tokio::spawn(async move {
            tx.send(transfer_speed).await.map_err(|e| {
                trace!("{:?}", e);
            })
        });
    }
}

/// This is an extended trait for the curl::easy::Handler trait.
pub trait ExtendedHandler: Handler {
    // Return the response body if the Collector if available.
    fn get_response_body(&self) -> Option<Vec<u8>> {
        None
    }
    // Return the response body if the Collector if available with complete headers.
    fn get_response_body_and_headers(&self) -> (Option<Vec<u8>>, Option<HeaderMap>) {
        (None, None)
    }
}

/// Collector::File(FileInfo) is used to be able to download and upload files.
/// Collector::Ram(`Vec<u8>`) is used to store response body into Memory.
/// Collector::RamWithHeaders(`Vec<u8>`, `Vec<u8>`) is used to store response body into Memory and with complete headers.
/// Collector::FileAndHeaders(`FileInfo`, `Vec<u8>`) is used to be able to download and upload files and with complete headers.
#[derive(Clone, Debug)]
pub enum Collector {
    /// Collector::File(`FileInfo`) is used to be able to download and upload files.
    File(FileInfo),
    /// Collector::Ram(`Vec<u8>`) is used to store response body into Memory.
    Ram(Vec<u8>),
    /// Collector::RamWithHeaders(`Vec<u8>`, `Vec<u8>`) is used to store response body into Memory and with complete headers.
    RamAndHeaders(Vec<u8>, Vec<u8>),
    /// Collector::FileAndHeaders(`FileInfo`, `Vec<u8>`) is used to be able to download and upload files and with complete headers.
    FileAndHeaders(FileInfo, Vec<u8>),
}

impl Handler for Collector {
    /// This will store the response from the server
    /// to the data vector or into a file depends on the
    /// Collector being used.
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        match self {
            Collector::File(info) => {
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(info.path.clone())
                    .map_err(|e| {
                        trace!("{}", e);
                        WriteError::Pause
                    })?;

                file.write_all(data).map_err(|e| {
                    trace!("{}", e);
                    WriteError::Pause
                })?;

                info.update_bytes_transferred(data.len());

                send_transfer_info(info);
                Ok(data.len())
            }
            Collector::Ram(container) => {
                container.extend_from_slice(data);
                Ok(data.len())
            }
            Collector::RamAndHeaders(container, _) => {
                container.extend_from_slice(data);
                Ok(data.len())
            }
            Collector::FileAndHeaders(info, _) => {
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(info.path.clone())
                    .map_err(|e| {
                        trace!("{}", e);
                        WriteError::Pause
                    })?;

                file.write_all(data).map_err(|e| {
                    trace!("{}", e);
                    WriteError::Pause
                })?;

                info.update_bytes_transferred(data.len());

                send_transfer_info(info);
                Ok(data.len())
            }
        }
    }
    /// This will read the chunks of data from a file that will be uploaded
    /// to the server. This will be use if the Collector is Collector::File(FileInfo).
    fn read(&mut self, data: &mut [u8]) -> Result<usize, ReadError> {
        match self {
            Collector::File(info) => {
                let mut file = File::open(info.path.clone()).map_err(|e| {
                    trace!("{}", e);
                    ReadError::Abort
                })?;

                file.seek(SeekFrom::Start(info.bytes_transferred() as u64))
                    .map_err(|e| {
                        trace!("{}", e);
                        ReadError::Abort
                    })?;

                let read_size = file.read(data).map_err(|e| {
                    trace!("{}", e);
                    ReadError::Abort
                })?;

                info.update_bytes_transferred(read_size);

                send_transfer_info(info);
                Ok(read_size)
            }
            Collector::Ram(_) => Ok(0),
            Collector::RamAndHeaders(_, _) => Ok(0),
            Collector::FileAndHeaders(info, _) => {
                let mut file = File::open(info.path.clone()).map_err(|e| {
                    trace!("{}", e);
                    ReadError::Abort
                })?;

                file.seek(SeekFrom::Start(info.bytes_transferred() as u64))
                    .map_err(|e| {
                        trace!("{}", e);
                        ReadError::Abort
                    })?;

                let read_size = file.read(data).map_err(|e| {
                    trace!("{}", e);
                    ReadError::Abort
                })?;

                info.update_bytes_transferred(read_size);

                send_transfer_info(info);
                Ok(read_size)
            }
        }
    }

    fn header(&mut self, data: &[u8]) -> bool {
        match self {
            Collector::File(_) => {}
            Collector::Ram(_) => {}
            Collector::RamAndHeaders(_, headers) => {
                headers.extend_from_slice(data);
            }
            Collector::FileAndHeaders(_, headers) => {
                headers.extend_from_slice(data);
            }
        }
        true
    }
}

impl ExtendedHandler for Collector {
    /// If Collector::File(FileInfo) is set, there will be no response body since the response
    /// will be stored into a file.
    ///
    /// If Collector::Ram(`Vec<u8>`) is set, the response body can be obtain here.
    fn get_response_body(&self) -> Option<Vec<u8>> {
        match self {
            Collector::File(_) => None,
            Collector::Ram(container) => Some(container.clone()),
            Collector::RamAndHeaders(container, _) => Some(container.clone()),
            Collector::FileAndHeaders(_, _) => None,
        }
    }

    /// If Collector::File(`FileInfo`) is set, there will be no response body since the response will be stored into a file.
    /// If Collector::Ram(`Vec<u8>`) is set, the response body can be obtain here.
    /// If Collector::RamAndHeaders(`Vec<u8>`, `Vec<u8>`) is set, the response body and the complete headers are generated.
    /// If Collector::FileAndHeaders(`FileInfo`, `Vec<u8>`) is set, there will be no response body since the response will be stored into a file but a complete headers are generated.
    fn get_response_body_and_headers(&self) -> (Option<Vec<u8>>, Option<HeaderMap>) {
        match self {
            Collector::File(_) => (None, None),
            Collector::Ram(container) => (Some(container.clone()), None),
            Collector::RamAndHeaders(container, headers) => {
                let header_str = std::str::from_utf8(headers).unwrap();
                let mut header_map = HeaderMap::new();

                for line in header_str.lines() {
                    // Split each line into key-value pairs
                    if let Some((key, value)) = line.split_once(": ").to_owned() {
                        if let Ok(header_name) = HeaderName::from_bytes(key.as_bytes()) {
                            if let Ok(header_value) = HeaderValue::from_str(value) {
                                // Insert the key-value pair into the HeaderMap
                                header_map.insert(header_name, header_value);
                            }
                        }
                    }
                }
                (Some(container.clone()), Some(header_map))
            }
            Collector::FileAndHeaders(_, headers) => {
                let header_str = std::str::from_utf8(headers).unwrap();
                let mut header_map = HeaderMap::new();

                for line in header_str.lines() {
                    // Split each line into key-value pairs
                    if let Some((key, value)) = line.split_once(": ").to_owned() {
                        if let Ok(header_name) = HeaderName::from_bytes(key.as_bytes()) {
                            if let Ok(header_value) = HeaderValue::from_str(value) {
                                // Insert the key-value pair into the HeaderMap
                                header_map.insert(header_name, header_value);
                            }
                        }
                    }
                }
                (None, Some(header_map))
            }
        }
    }
}
