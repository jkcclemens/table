use error::Result;
use untyped::Table;

use serde::de::DeserializeOwned;

use std::marker::PhantomData;

/// An iterator over a [`Table`].
pub struct TableIterator<'a, V> {
  pub(crate) table: &'a Table,
  pub(crate) pos: usize,
  pub(crate) offset: u64,
  pub(crate) _phantom: PhantomData<V>,
}

impl<'a, V> Iterator for TableIterator<'a, V>
  where V: DeserializeOwned
{
  type Item = Result<V>;

  fn next(&mut self) -> Option<Self::Item> {
    // get the length of the item
    let len = self.table.header.get(self.pos)?;

    // get the item without doing additional math
    let ret = self.table.get_at(self.offset, *len);

    // increment the position and offset
    self.offset += len;
    self.pos += 1;

    Some(ret)
  }
}
