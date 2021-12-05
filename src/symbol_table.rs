/*!

  The symbol table digests strings and integers and produces a `u64` as a proxy ID. The ID can be
  used later to retrieve the original string or integer.

*/

use symbol_map::indexing::{HashIndexing, Indexing};
use std::fmt::{Display, Formatter};

/// A "Symbol" is a `usize`, which implements the `SymbolId` trait from the `symbol_map` crate.
pub type Symbol = usize;
pub type SymbolTable<'s> = HashIndexing<SymbolData<'s>, Symbol>;

/// The global symbol table. Fascilities for manipulating this table are provided as module-level
/// free functions.
pub static mut SYMBOLS: SymbolTable<'s> = HashIndexing::default();


/// This is not to be confused with `symbol_map::table::Symbol<D,
/// I>`. In fact, `symbol_map::indexing::Insertion` wraps an instance
/// of `symbol_map::table::Symbol<crate::symbol_table::SymbolData,
/// u64>`, which in turn wraps a `SymbolData` and a `SymbolId`.

// todo: Is this redundant given existence of `parameters::ParameterValue`?

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum SymbolData<'s> {
  Str(&'s str),
  I64(i64),
  Null
}

impl<'s> Display for SymbolData<'s> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {

      SymbolData::Str(s) => write!(f, "{}", *s),

      SymbolData::I64(n) => write!(f, "k!{}", n),

      SymbolData::Null => write!(f, "null"),

    }
  }
}


/// Returns a SymbolId from a `&str` either by returning the `SymbolId` associated
/// to the string of the table already contains the string, or inserting the string
/// into the global `SYMBOLS` symbol table as a new symbol, producing a new `SymbolId
pub fn from_str(text: &str) -> &Symbol {
  unsafe {
    SYMBOLS.get_or_insert(SymbolData::Str(text)).unwrap().SymbolId()
  }
}


#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
