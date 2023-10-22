use std::io::Read;
use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    path::PathBuf,
};

use curl::easy::{Handler, ReadError, WriteError};

#[derive(Clone, Debug)]
pub struct FileInfo {
    pub path: PathBuf,
    bytes_transferred: usize,
}

impl FileInfo {
    pub fn path(path: PathBuf) -> Self {
        Self {
            path,
            bytes_transferred: 0,
        }
    }

    fn update_bytes_transferred(&mut self, transferred: usize) {
        self.bytes_transferred += transferred;
    }

    fn bytes_transferred(&self) -> usize {
        self.bytes_transferred
    }
}

#[derive(Clone, Debug)]
pub enum Collector {
    File(FileInfo),
    Ram(Vec<u8>),
}

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        match self {
            Collector::File(info) => {
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(info.path.clone())
                    .map_err(|e| {
                        eprintln!("{}", e);
                        WriteError::Pause
                    })?;

                file.write_all(data).map_err(|e| {
                    eprintln!("{}", e);
                    WriteError::Pause
                })?;

                info.update_bytes_transferred(data.len());
                Ok(data.len())
            }
            Collector::Ram(container) => {
                container.extend_from_slice(data);
                Ok(data.len())
            }
        }
    }

    fn read(&mut self, data: &mut [u8]) -> Result<usize, ReadError> {
        match self {
            Collector::File(info) => {
                let mut file = File::open(info.path.clone()).map_err(|e| {
                    eprintln!("{}", e);
                    ReadError::Abort
                })?;

                file.seek(SeekFrom::Start(info.bytes_transferred() as u64))
                    .map_err(|e| {
                        eprintln!("{}", e);
                        ReadError::Abort
                    })?;

                let read_size = file.read(data).map_err(|e| {
                    eprintln!("{}", e);
                    ReadError::Abort
                })?;

                info.update_bytes_transferred(read_size);

                Ok(read_size)
            }
            Collector::Ram(_) => Ok(0),
        }
    }
}

impl Collector {
    pub fn get_response_body(&self) -> Option<Vec<u8>> {
        match self {
            Collector::File(_) => None,
            Collector::Ram(container) => Some(container.clone()),
        }
    }
}
