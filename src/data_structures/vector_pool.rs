/*!

A VectorPool is a shared pool of learned clauses that reuses memory and functions essentially as a bump allocator.


 */

// todo: replace VectorPool with the bumpalo crate, because this is ridiculous.
// todo: make this generic over the index type.

use std::collections::HashSet;
use std::error::Error;

use crate::log::{
  log_at_level,
  log_assert,
  verify
};

pub type VectorIndex    = usize;
pub type VectorIndexSet = HashSet<usize>;

// todo: figure out what VectorPool needs.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct VectorPool {
  // vectors: Vec<PooledClause>,

  /// Stores contiguous runs of the form |owner|length|0|1|2|...|n-1|.
  // todo: This data layout is presumably for cache locality, but I doubt it matters, and it
  // complicates the code significantly.
  vectors: Vec<VectorIndex>,
  /// The "real" number of elements stored in `vectors`, as opposed to what `vectors.len()` reports.
  size   : usize,
  /// A "pointer" to the end of the last vector. This is necessary because the space allocated for the vector may not
  /// be filled.
  tail   : usize,
  /// `heads[i]` tracks the true index of the ith vector in vectors.
  heads  : Vec<VectorIndex>, // One per thread
  ///
  at_end : Vec<bool>,        // One per thread
}

impl VectorPool {

  /// Given an `index` of the head of a vector, advances `index` to the head of the next vector. If the head of the
  /// last vector is provided, `index` is "wrapped around" to `0`, which is the head of the first vector.
  pub fn next(&self, index: &mut VectorIndex) {
    sassert!(index < self.size);

    let n = index + 2 + self.get_length(*index);
    if n >= self.size {
      *index = 0;
    }
    else {
      *index = n;
    }
  }

  /// Extracts the owner of the vector at `index` in `vectors`.
  pub fn get_owner(&self, index: VectorIndex) -> VectorIndex {
    self.vectors[index]
  }

  /// Extracts the length of the vector at `index` in `vectors`.
  pub fn get_length(&self, index: VectorIndex) -> usize {
    self.vectors[index+1]
  }

  /// Gives a pointer to the data of the vector at `index`
  fn get_ptr(&self, index: VectorIndex) -> *const VectorIndex {
    return self.vectors.as_ptr() + index + 2; // todo: Why add 2 here?
  }

  /// Clears `vectors` and resets all bookkeeping to initial state. Resizes `vectors` and bookkeeping
  /// vectors according to `num _threads` and `size`.  Note that `num _threads` is equal to the number of
  /// vectors, and `size` should be twice the number of vectors plus the sum of the lengths of all vectors.
  pub fn reserve(&mut self, num_threads: usize, size: usize) {
    self.vectors.clear();
    self.vectors.resize(size, 0);
    self.heads.clear();
    self.heads.resize(num_threads, 0);
    self.at_end.clear();
    self.at_end.resize(num_threads, true);
    self.tail = 0;
    self.size = size;
  }

  /// Vectors are added to the pool one at a time. This method initializes the process of adding a vector.
  pub fn begin_add_vector(&mut self, owner: VectorIndex, n: usize) {
    sassert!(self.tail < self.size);

    let capacity = n + 2;
    self.vectors.resize(capacity, 0.into());

    log_at_level(
      3,
      format!("{}: begin-add {} tail: {} size: {}\n" , owner, n , self.tail, self.size).as_str()
    );

    for i in 0..self.heads.size() {
      while (self.tail < self.heads[i]) && (self.heads[i] < self.tail + capacity) {
        self.next(&mut self.heads[i]);
      }
      self.at_end[i] = false;
    }
    self.vectors[self.tail] = owner;
    self.tail += 1;
    self.vectors[self.tail] = n;
    self.tail += 1;
  }

  /// After a vector is added, we make sure that `tail` does not point past the end of `vectors`.
  pub fn end_add_vector(&mut self, ) {
    // todo: Under what circumstances would `tail` be _greater_ than `size`? Surely that is an error state.
    if self.tail >= self.size {
      self.tail = 0;
    }
  }

  /// Inserts `e` at the "end" of the last vector, which is pointed to by `tail`, and then `tail` is incremented.
  pub fn add_vector_elem(&mut self, e: VectorIndex) {
    self.vectors[self.tail] = e;
    self.tail += 1;
  }

  /// Returns a pointer to the vector data of the last vector?
  pub fn get_vector(&mut self, owner: VectorIndex,  ptr: &mut VectorIndex)
    -> Result<(*const VectorIndex, size), dyn Error>
  {
    let mut head = self.heads[owner];
    let mut iterations: usize = 0;

    while (head != self.tail) || (!self.at_end[owner]) {
      iterations += 1;

      sassert!((head < self.size) && (self.tail < self.size));

      let is_self = (owner == self.get_owner(head));
      self.next(&mut self.heads[owner]);

      {
        let log_level = if iterations > self.size { 0 } else { 3 };
        log_at_level(
          log_level,
          format!(
            "{}: [{}:{}] tail: {}\n",
            owner,
            head,
            self.heads[owner],
            self.tail
          ).as_str()
        );
      }

      self.at_end[owner] = (self.heads[owner] == self.tail);

      if !is_self {
        let n = self.get_length(head);
        unsafe {
          // todo: Change this type to VectorIndex.
          let ptr: *const VectorIndex = self.get_ptr(head);
        }
        return Ok((ptr, n));
      }
      head = self.heads[owner];
    }

    return Err(Error);
  }

}



#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
