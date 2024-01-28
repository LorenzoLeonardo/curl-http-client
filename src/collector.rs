use std::io::Read;
use std::time::Instant;
use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    path::PathBuf,
};

use curl::easy::{Handler, ReadError, WriteError};
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

/// The Collector will handle two types in order to store data, via File or via RAM.
/// Collector::File(FileInfo) is useful to be able to download and upload files.
/// Collector::Ram(`Vec<u8>`) is used to store response body into Memory.
#[derive(Clone, Debug)]
pub enum Collector {
    /// Collector::File(FileInfo) is useful to be able to download and upload files.
    File(FileInfo),
    /// Collector::Ram(`Vec<u8>`) is used to store response body into Memory.
    Ram(Vec<u8>),
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
        }
    }
}

impl Collector {
    /// If Collector::File(FileInfo) is set, there will be no response body since the response
    /// will be stored into a file.
    ///
    /// If Collector::Ram(`Vec<u8>`) is set, the response body can be obtain here.
    pub fn get_response_body(&self) -> Option<Vec<u8>> {
        match self {
            Collector::File(_) => None,
            Collector::Ram(container) => Some(container.clone()),
        }
    }
}
