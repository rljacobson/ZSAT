/*!
A `Clause` is a data structure that efficiently represents a set of OR'ed literals.

A clause is a set of distinct literals OR'ed together. For example
$x_1 \lor \overline{x}_2 \lor \overline{x}_3 \lor x_4$.

 */

use std::ops::Index;

use crate::{
  BoolVariable,
  Literal,
  LiteralVector,
  VariableApproximateSet,
  data_structures::ApproximateSet
};

pub type ClauseOffset = usize;
pub type ClauseVector = Vec<Clause>;
pub type ClauseWrapperVector = Vec<ClauseWrapper>;


/// The primary clause representation. `Clause`'s are garbage collected.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Clause {
  literals    : LiteralVector,
  approx      : VariableApproximateSet,

  id          : u32,
  size        : u32,
  capacity    : u32,

  inact_rounds: u32,
  glue        : u32,
  psm         : u32, // Transient field used during gc

  is_strengthened: bool,
  is_removed     : bool,
  is_learned     : bool,
  is_used        : bool,
  is_frozen      : bool,
  reinit_stack   : bool,
}

impl Clause {
  // region Getters and Setters
  pub fn literals(&self) -> &LiteralVector          { &self.literals }
  pub fn approx(&self)   -> &VariableApproximateSet { &self.approx   }

  pub fn size(&self)            -> u32  { self.size            }
  pub fn capacity(&self)        -> u32  { self.capacity        }
  pub fn id(&self)              -> u32  { self.id              }
  pub fn inact_rounds(&self)    -> u32  { self.inact_rounds    }
  pub fn glue(&self)            -> u32  { self.glue            }
  pub fn psm(&self)             -> u32  { self.psm             }
  pub fn is_strengthened(&self) -> bool { self.is_strengthened }
  pub fn is_removed(&self)      -> bool { self.is_removed      }
  pub fn is_learned(&self)      -> bool { self.is_learned      }
  pub fn is_used(&self)         -> bool { self.is_used         }
  pub fn is_frozen(&self)       -> bool { self.is_frozen       }
  pub fn reinit_stack(&self)    -> bool { self.reinit_stack    }

  // pub fn set_literals(&mut self     , literals     : LiteralVector          )  { self.literals = literals;}
  // pub fn set_approx(&mut self       , approx       : VariableApproximateSet )  { self.approx   = approx;  }
      fn set_size(&mut self         , size         :  u32  ) { self.size         = size;         }
      fn set_capacity(&mut self     , capacity     :  u32  ) { self.capacity     = capacity;     }
  pub fn set_removed(&mut self      , is_removed   :  bool ) { self.is_removed   = is_removed;   }
  pub fn set_used(&mut self         , is_used      :  bool ) { self.is_used      = is_used;      }
  pub fn set_reinit_stack(&mut self , reinit_stack :  bool ) { self.reinit_stack = reinit_stack; }


  // Setters needing special treatment
  pub fn shrink(&mut self, literal_count: u32) {
    sassert!(literal_count <= self.size);

    if literal_count < self.size {
      self.set_size(literal_count);
      self.set_strengthened(true);
    }
  }

  pub fn restore(&mut self, literal_count: u32) {
    sassert!(num_lits <= self.capacity);

    self.set_size(literal_count);
  }

  pub fn set_glue(&mut self, glue: u32) {
    self.glue = u32::min(glue, 255);
  }

  pub fn set_psm(&mut self, psm: u32) {
    self.psm = u32::min(psm, 255);
  }

  pub fn set_frozen(&mut self, is_frozen: bool) {
    sassert!(self.is_learned);
    sassert!(self.is_frozen != is_frozen);
    self.is_frozen = is_frozen;
  }

  pub fn freeze(&mut self){
    self.set_is_frozen(true);
  }

  pub fn unfreeze(&mut self){  // You mean thaw?
    self.set_is_frozen(false);
  }

  pub fn inc_inact_rounds(&mut self)  {
    self.inact_rounds += 1;
  }

  pub fn reset_inact_rounds(&mut self)  {
    self.inact_rounds = 0;
  }

  pub fn set_strengthened(&mut self, is_strengthened: bool) {
    if is_strengthened {
      self.is_strengthened = true;
      self.update_approx(&self.literals);
    }
    else {
      self.is_strengthened = false;
    }
  }

  pub fn set_learned(&mut self, is_learned: bool) {
    sassert!(self.is_learned() != is_learned);

    self.is_learned = is_learned;
  }

  /// Gives the raw value of the first literal
  pub fn new_clause_offset(&self) -> ClauseOffset {
    self.literals[0].index()
  }

  /// Gives the raw value of the first literal
  pub fn set_new_clause_offset(&mut self, offset: ClauseOffset) {
    self.literals[0] = Literal(offset);
  }

  // endregion Getters and Setters

  // region Methods forwarded to `self.literals`

  pub fn contains_variable(&self, variable: BoolVariable) -> bool {
    self.literals.iter().any(|a| a.var() == variable)
  }

  pub fn contains_literal(&self, literal: Literal) -> bool {
    self.literals.contains(&literal)
  }

  /// Remove every instance of `literal`. This operation is done in-place.
  pub fn eliminate(&mut self, literal: Literal) {
    let initial_size = self.literals.len();

    self.literals.retain(
      | w | w!=literal
    );

    let number_removed = initial_size - self.literals.len();
    if number_removed > 0 {
      self.set_size(self.size() - number_removed);
      self.set_strengthened(true);
    }
  }

  // endregion Methods forwarded to `self.literals`

  pub fn update_approx(&mut self, values: &[Literal]) {
    self.approx = VariableApproximateSet::with_values(values.iter().map(|a| a.var()).collect())
  }

  fn new(id: u32, literals: LiteralVector, learned: bool) -> Self {
    Self {
      id,
      literals,
      is_learned: learned,
      ..Clause::default()
    }
  }

  /*

    literal & operator[](unsigned idx) { SASSERT(idx < m_size); return m_lits[idx]; }
    literal const & operator[](unsigned idx) const { SASSERT(idx < m_size); return m_lits[idx]; }

    bool satisfied_by(model const & m) const;

   */

}

impl Default for Clause {
  fn default() -> Self {
    Self {
      approx         :  VariableApproximateSet::default(),
      literals       :  LiteralVector::default(),
      id             :  0,
      size           :  0,
      capacity       :  0,
      inact_rounds   :  8,
      glue           :  8,
      psm            :  8,
      is_strengthened:  true,
      is_removed     :  true,
      is_learned     :  true,
      is_used        :  true,
      is_frozen      :  true,
      reinit_stack   :  true,
    }
  }
}

impl Index<usize> for Clause {
  type Output = Literal;

  fn index(&self, index: usize) -> &Self::Output {
    sassert!(idx < self.size);

    &self.literals[index]
  }
}

impl Index<u32> for Clause {
  type Output = Literal;

  fn index(&self, index: u32) -> &Self::Output {
    sassert!(idx < self.size);

    &self.literals[index as usize]
  }
}


/// A wrapper type for `Clause` that provides a much smaller representation
/// for binary clauses. Only a subset of the `ClauseCore` API is provided.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum ClauseWrapper {
  Binary{
    literal1  : Literal,
    literal2  : Literal,
    is_learned: bool
  },
  Nonbinary(Box<Clause>)
}

impl ClauseWrapper {
  pub fn size(&self) -> usize {
    match self {
      ClauseWrapper::Binary { .. } => 2,
      ClauseWrapper::Nonbinary(c)  => c.size()
    }
  }

  pub fn contains_literal(&self, literal: Literal) -> bool {
    match self {

      ClauseWrapper::Binary { literal1, literal2, .. } => {
        (literal == literal1) || (literal == literal2)
      },

      ClauseWrapper::Nonbinary(c)  => {
        c.contains_literal(literal)
      }

    }
  }

  pub fn contains_variable(&self, variable: BoolVariable) -> bool {
    match self {

      ClauseWrapper::Binary { literal1, literal2, .. } => {
        (variable == literal1.var()) || (variable == literal2.var())
      },

      ClauseWrapper::Nonbinary(c)  => {
        c.contains_variable(variable)
      }

    }
  }

  pub fn is_removed(&self) -> bool {
    if let ClauseWrapper::Nonbinary(clause) = self {
      clause.is_removed()
    } else {
      false
    }
  }

  pub fn is_learned(&self) -> bool {
    if let ClauseWrapper::Nonbinary(clause) = self {
      clause.is_learned()
    } else {
      false
    }
  }

}


impl Index<usize> for ClauseWrapper {
  type Output = Literal;

  fn index(&self, index: usize) -> &Self::Output {
    match self {

      ClauseWrapper::Binary { literal1, literal2, .. } => {
        sassert!(index < 2);

        match index {
          0 => literal1,
          1 => literal2,
          _ => panic!("Index out of bounds")
        }
      },

      ClauseWrapper::Nonbinary(c)  => {
        c[index]
      }

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
