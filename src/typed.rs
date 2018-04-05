//! A table with one type for all entries.
//!
//! Really, this is just a wrapper around a [`Table`] that forces the compiler to use one type for
//! the generic calls.

use error::Result;
use iter::TableIterator;
use untyped::Table;

use serde::de::DeserializeOwned;
use serde::ser::Serialize;

use std::marker::PhantomData;

pub struct TypedTable<V> {
  table: Table,
  _phantom: PhantomData<V>,
}

impl<V> TypedTable<V>
  where V: Serialize + DeserializeOwned,
{
  pub fn open(name: &str) -> Result<TypedTable<V>> {
    Ok(Table::open(name)?.into())
  }

  pub fn len(&self) -> usize {
    self.table.len()
  }

  pub fn is_empty(&self) -> bool {
    self.table.is_empty()
  }

  pub fn get(&self, pos: usize) -> Result<Option<V>> {
    self.table.get(pos)
  }

  pub fn push(&mut self, value: &V) -> Result<()> {
    self.table.push(value)
  }

  pub fn pop(&mut self) -> Result<Option<V>> {
    self.table.pop()
  }

  pub fn write_header(&mut self) -> Result<()> {
    self.table.write_header()
  }

  pub fn iter(&self) -> TableIterator<V> {
    self.table.iter()
  }
}

impl<V> From<Table> for TypedTable<V> {
  fn from(table: Table) -> TypedTable<V> {
    TypedTable {
      table,
      _phantom: Default::default(),
    }
  }
}
