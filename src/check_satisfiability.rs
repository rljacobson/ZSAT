/*!
  
  Checking satisfiability results in information like

   * The result (`LiftedBool`)
   * Collected statistics
   * The `Model`
   * The proof

  and so on. The `SatisfiabilityCheckResult` trait is implemented for the struct that manages this
  information.
  
*/

use std::rc::Rc;

use crate::{LiftedBool, Model};
use crate::symbol_table::Symbol;
use crate::missing_types::*;

type ExpressionVector = Vec<Expression>;

pub trait SatisfiabilityCheckResult {
  fn new(m: Rc<ASTManager>) -> Self;
  fn set_status(&mut self, r: LiftedBool) -> LiftedBool;
  fn status(&self) -> LiftedBool;
  /// Updates `statistics` with `self.statistics`
  fn collect_statistics(&self, statistics: &mut Statistics);
  /// Appends `self.core` to `ev` if `self.status == False`
  fn get_unsat_core(&self, ev: &mut ExpressionVector);
  fn set_model_converter(&mut self, mc: Rc<ModelConverter>);
  fn get_model_converter(&self) -> Option<Rc<ModelConverter>>;
  fn get_model_core(&self) -> Option<Rc<Model>>;
  fn get_model(&self) -> Option<Rc<Model>> {  // todo: Get rid of these output arguments.
    let maybe_model_core = self.get_model_core();
    let maybe_model_converter = self.get_model_converter();
    if let Some(model_core) = maybe_model_core {
      if let Some(model_converter) = maybe_model_converter{
        Some(Rc::new(model_converter.convert(model_core)))
      } else {
        Some(model_core.clone())
      }
    } else{
      None
    }
  }
  fn get_proof(&self) -> Rc<Proof>;
  fn reason_unknown(&self) -> String;
  fn set_reason_unknown(&mut self, msg: &str);
  fn set_reason_from_event_handler(&mut self, eh: &EventHandler){
    if eh.caller_id() != EventCaller::Unset{
      self.set_reason_unknown(format!("{}", eh.caller_id()).as_str());
    }
  }
  fn get_labels(&self) -> Option<&Vec<Symbol>>;
  // todo: In the absence of our own smart pointer type, do we need a manager at all?
  fn get_ast_manager(&self) -> &ASTManager;
  fn collect_timer_stats(&self, statistics: &mut Statistics);
}

/*
  class scoped_solver_time {
      check_sat_result& c;
      timer t;
  public:
      scoped_solver_time(check_sat_result& c):c(c) { c.m_time = 0; }
      ~scoped_solver_time() { c.m_time = t.get_seconds(); }
  };

*/

struct SimpleSatisfiabilityCheckResult {
  core           : ExpressionVector,
  model          : Option<Rc<Model>>,
  model_converter: Option<Rc<ModelConverter>>,
  proof          : Rc<Proof>,
  statistics     : Statistics,
  status         : LiftedBool,
  time           : f64,

  reason_unknown_msg: String,
}

impl SatisfiabilityCheckResult for SimpleSatisfiabilityCheckResult {

  fn new(m: Rc<ASTManager>) -> Self {
    Self{
      core           : vec![],
      model          : None,
      model_converter: None,
      proof          : Rc::new(Proof::new(m)),
      statistics          : Statistics::new(),
      status         : LiftedBool::Undefined,
      time           : 0f64,
      reason_unknown_msg: "".to_string()
    }
  }

  fn set_status(&mut self, lb: LiftedBool) -> LiftedBool {
    self.status = lb;
    lb
  }

  fn status(&self) -> LiftedBool{
    self.status
  }

  fn collect_statistics(&self, statistics: &mut Statistics) {
    statistics.update_with(self.statistics);
  }

  fn get_unsat_core(&self, ev: &mut ExpressionVector) {
    if self.status == LiftedBool::False{
      ev.extend(self.core.iter().cloned());
    }
  }

  fn set_model_converter(&mut self, mc: Rc<ModelConverter>){
    self.model_converter = Some(mc);
  }

  fn get_model_converter(&self) -> Option<Rc<ModelConverter>>{
    self.model_converter.clone()
  }

  fn get_model_core(&self) -> Option<Rc<Model>>{
    if self.status != LiftedBool::False {
      self.model.clone()
    } else {
      None
    }
  }

  fn get_proof(&self) -> Rc<Proof>{
    self.proof.clone()
  }
  fn reason_unknown(&self) -> String {
    self.reason_unknown_msg.clone()
  }
  fn set_reason_unknown(&mut self, msg: &str) {
    self.reason_unknown_msg = msg.to_string();
  }

  fn get_labels(&self) -> Option<&Vec<Symbol>>{
    None
  }

  fn get_ast_manager(&self) -> &ASTManager{
    self.proof.get_manager()
  }

  fn collect_timer_stats(&self, statistics: &mut Statistics){
    if self.time != 0.0 {
      statistics.update("time", self.time);
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
