use std::io::Read;
use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    path::PathBuf,
};

use curl::easy::{Handler, ReadError, WriteError};

#[derive(Clone, Debug)]
pub enum Collector {
    File(PathBuf, usize),
    Ram(Vec<u8>),
}

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        match self {
            Collector::File(download_path, _size) => {
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(download_path)
                    .map_err(|e| {
                        eprintln!("{}", e);
                        WriteError::Pause
                    })?;

                file.write_all(data).map_err(|e| {
                    eprintln!("{}", e);
                    WriteError::Pause
                })?;
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
            Collector::File(path, size) => {
                let mut file = File::open(path).map_err(|e| {
                    eprintln!("{}", e);
                    ReadError::Abort
                })?;

                // Seek to the desired offset
                file.seek(SeekFrom::Start(*size as u64)).map_err(|e| {
                    eprintln!("{}", e);
                    ReadError::Abort
                })?;

                let read_size = file.read(data).map_err(|e| {
                    eprintln!("{}", e);
                    ReadError::Abort
                })?;

                // Update this so that we could seek succeding blocks of data from the file
                *size += read_size;

                Ok(read_size)
            }
            Collector::Ram(_) => Ok(0),
        }
    }
}

impl Collector {
    pub fn get_response_body(&self) -> Option<Vec<u8>> {
        match self {
            Collector::File(_, _) => None,
            Collector::Ram(container) => Some(container.clone()),
        }
    }
}
