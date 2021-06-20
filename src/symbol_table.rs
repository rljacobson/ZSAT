/*!
  
  The symbol table digests strings and integers and produces a `u64` as a proxy ID. The ID can be
  used later to retrieve the original string or integer.
  
*/

use symbol_map::indexing::HashIndexing;
use std::fmt::{Display, Formatter};

/// Most of the heavy lifting is done by `symbol_map`.
pub type SymbolTable<'s> = HashIndexing<Symbol<'s>, u64>;

pub static SYMBOLS: SymbolTable<'s> = HashIndexing::default();


#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Symbol<'s> {
  Str(&'s str),
  I64(i64),
  Null
}

impl<'s> Display for Symbol<'s> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {

      Symbol::Str(s) => write!(f, "{}", *s),

      Symbol::I64(n) => write!(f, "k!{}", n),

      Symbol::Null => write!(f, "null"),

    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
