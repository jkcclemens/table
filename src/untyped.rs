//! A table with a potentially different type for each entry.

use error::{Error, Result};
use iter::TableIterator;

use serde::de::DeserializeOwned;
use serde::ser::Serialize;

use serde_msgpack;

use std::cell::RefCell;
use std::fs::{OpenOptions, File};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::Path;

/// A table.
///
/// Tables function similarly to [`Vec`]s.
///
/// [`Vec`]: ::std::vec::Vec
#[derive(Debug)]
pub struct Table {
  // header[x] = length of element at data[x]
  // sum(header[..x])..sum(header[..x]) + header[x] = data[x]
  pub(crate) header: Vec<u64>,
  header_file: RefCell<File>,
  data_file: RefCell<File>,
  len: u64,
}

impl Table {
  /// Open a table with a given name, creating it on the disk if it doesn't exist.
  pub fn open(name: &str) -> Result<Table> {
    let mut oo = OpenOptions::new();
    oo
      .read(true)
      .write(true)
      .create(true);

    let idx = format!("{}.idx", name);
    let p = Path::new(&idx);

    let existed = p.exists();

    let header_file = RefCell::new(oo.open(p).map_err(Error::Io)?);
    let data_file = RefCell::new(oo.open(format!("{}.dat", name)).map_err(Error::Io)?);

    let header = if existed {
      serde_msgpack::from_read(&*header_file.borrow()).map_err(Error::MsgPackDec)?
    } else {
      Vec::new()
    };
    let len = header.iter().sum();
    Ok(Table {
      header,
      header_file,
      data_file,
      len,
    })
  }

  /// Gets the length of this table.
  pub fn len(&self) -> usize {
    self.header.len()
  }

  /// Checks if the table is empty.
  ///
  /// Equivalent to `table.len() == 0`.
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Get an item at position `pos`.
  ///
  /// This attempts to deserialize whatever bytes are found at position `pos`. This will return
  /// `Ok(None)` if `pos` exceeds the table's bounds.
  pub fn get<V>(&self, pos: usize) -> Result<Option<V>>
    where V: DeserializeOwned,
  {
    // check if the header has an entry for the position
    if pos >= self.header.len() {
      return Ok(None);
    }

    // sum all header entries before the position
    let start: u64 = self.header[..pos].iter().sum();
    // get the header entry at the position
    let len = self.header[pos];
    // seek to the start of the data
    self.data_file.borrow_mut().seek(SeekFrom::Start(start)).map_err(Error::Io)?;
    // create a vector to store the data
    let mut data = vec![0; len as usize];
    // read the data
    self.data_file.borrow_mut().read_exact(&mut data).map_err(Error::Io)?;
    // attempt to deserialize the data
    serde_msgpack::from_slice(&data).map_err(Error::MsgPackDec).map(Some)
  }

  pub fn get_at<V>(&self, offset: u64, len: u64) -> Result<V>
    where V: DeserializeOwned
  {
    // seek to the offset
    self.data_file.borrow_mut().seek(SeekFrom::Start(offset)).map_err(Error::Io)?;
    // create a vector with enough room for the data
    let mut data = vec![0; len as usize];
    // read in the data
    self.data_file.borrow_mut().read_exact(&mut data).map_err(Error::Io)?;
    // attempt to deserialize
    serde_msgpack::from_slice(&data).map_err(Error::MsgPackDec)
  }

  /// Add an item to the end of the table.
  // FIXME: if writing fails, the header is invalid
  pub fn push<V>(&mut self, value: &V) -> Result<()>
    where V: Serialize,
  {
    // serialize the data
    let serialized = serde_msgpack::to_vec(value).map_err(Error::MsgPackEnc)?;
    // get the length of the serialized data
    let len = serialized.len();
    // add the length to the header
    self.header.push(len as u64);
    // add length to master length
    self.len += len as u64;
    // get the size of the data file according to the updated header
    // let size: u64 = self.header.iter().sum();
    let mut data_file = self.data_file.borrow_mut();
    // set the data file's length
    data_file.set_len(self.len).map_err(Error::Io)?;
    // seek to the start of the new data
    data_file.seek(SeekFrom::End(-(len as i64))).map_err(Error::Io)?;
    // write the new data
    data_file.write_all(&serialized).map_err(Error::Io)?;

    Ok(())
  }

  /// Remove an item from the end of the table, if there is an item to remove.
  ///
  /// If a deserialization error occurs, the data that was popped will be lost.
  ///
  /// This will return `Ok(None)` if the table is empty.
  pub fn pop<V>(&mut self) -> Result<Option<V>>
    where V: DeserializeOwned,
  {
    // make sure there's data to pop
    if self.header.is_empty() {
      return Ok(None);
    }

    // pop the length from the header
    // we just checked to make sure the header's not empty
    let len = self.header.pop().unwrap();

    // calculate start point (note that in order to truncate, we have to do this, so we can't just
    // relative seek from the end of the file)
    // let start: u64 = self.header[..self.header.len()].iter().sum();
    let start = self.len - self.header[self.header.len() - 1];
    let mut data_file = self.data_file.borrow_mut();
    // seek to the start
    data_file.seek(SeekFrom::Start(start)).map_err(Error::Io)?;

    // allocate a vec to store the data
    let mut data = vec![0; len as usize];
    // read the data
    data_file.read_exact(&mut data).map_err(Error::Io)?;

    // truncate the file
    data_file.set_len(start).map_err(Error::Io)?;

    // attempt to deserialize
    serde_msgpack::from_slice(&data).map_err(Error::MsgPackDec).map(Some)
  }

  /// Write the header file to the disk.
  ///
  /// Unlike the data file, the header file is not written to until this is called or the table is
  /// dropped. Calling this will force the header file to be written to the disk.
  pub fn write_header(&mut self) -> Result<()> {
    // serialize the header
    let header = serde_msgpack::to_vec(&self.header).map_err(Error::MsgPackEnc)?;
    // set the length of the header file
    self.header_file.borrow_mut().set_len(header.len() as u64).map_err(Error::Io)?;
    // seek to the beginning
    self.header_file.borrow_mut().seek(SeekFrom::Start(0)).map_err(Error::Io)?;
    // write the header
    self.header_file.borrow_mut().write_all(&header).map_err(Error::Io)?;

    Ok(())
  }

  /// Create an iterator over the values in the table.
  pub fn iter<V>(&self) -> TableIterator<V> {
    TableIterator {
      table: self,
      pos: 0,
      offset: 0,
      _phantom: Default::default(),
    }
  }
}

impl Drop for Table {
  fn drop(&mut self) {
    self.write_header().ok();
  }
}
