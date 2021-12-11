/*!
Types associated with configuration parameters.

Configuration parameters in the z3 code base are defined in *.pyg files and generated at
compile time. In contrast, we read in the parameter database from a JSON file at runtime.

*/

use std::{
  fs::read_to_string,
  collections::HashMap,
  path::Path,
  error::Error,
  cell::RefCell,
  rc::Rc
};

use json::{
  parse as parse_json,
  JsonValue,
  Result as JsonResult,
  JsonError
};
// use term::terminfo::Error::IoError;
use std::ops::Index;

// todo: Should this be copy on write?
pub type ParametersRef<'s> = Rc<RefCell<Parameters<'s>>>;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum ParameterValue<'s> {
  UnsignedInteger(u64),
  Bool(bool),
  Double(f64),
  Symbol(&'s str)
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Parameter<'s> {
  name       : &'static str,
  value      : ParameterValue<'s>,
  description: &'static str
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Parameters<'s> {
  module     : &'s str,
  export     : bool,      // todo: Is this relevant? Kept it from z3.
  description: &'s str,
  parameters : HashMap<&'s str, Parameter<'s>>
}

impl<'s> Parameters<'s>{

  /// Get's the `Parameter` associated  with `symbol` and returns its `ParameterValue`.
  pub fn get_value(&self, symbol: &str) -> Option<ParameterValue> {
    self.parameters
        .get(symbol)
        .and_then(| v | Some(v.value))
  }
}

impl<'s> Index<&str> for Parameters<'s>{
  type Output = Parameter<'s>;

  fn index(&self, index: &str) -> &Self::Output {
    self.parameters.index(index)
  }

}

fn json_value_to_parameter_value<'a, 'b, 'c>(datatype: &'a str, json_value: &'b JsonValue) -> JsonResult<ParameterValue<'c>> {
  match datatype {

    "UINT"   => if let Some(number) = json_value.as_u64() {
        Ok(ParameterValue::UnsignedInteger(number))
      } else {
        Err(
          JsonError::wrong_type(
            format!("Expected a parameter datatype, found `{}`.", json_value).as_str()
          )
        )
      },

    "BOOL"   => if let Some(value)= json_value.as_bool() {
        Ok(ParameterValue::Bool(value))
      } else {
        Err(
          JsonError::wrong_type(
            format!("Expected a bool, found `{}`.", json_value).as_str()
          )
        )
      }

    "DOUBLE" => if let Some(number)= json_value.as_f64() {
        Ok(ParameterValue::Double(number))
      } else {
        Err(
          JsonError::wrong_type(
            format!("Expected a double, found `{}`.", json_value).as_str()
          )
        )
      },

    "SYMBOL" => if let Some(text) = json_value.as_str() {
        Ok(ParameterValue::Symbol(text.clone()))
      } else {
        Err(
          JsonError::wrong_type(
            format!("Expected a symbol, found `{}`.", json_value).as_str()
          )
        )
      },

    _other   =>
      Err(
        JsonError::wrong_type(
          format!("Expected a parameter datatype, found `{}`.", _other).as_str()
        )
      )

  }
}

/// Builds the `Parameters` map by reading in the parameters database from the given file.
pub fn deserialize_parameters(file_path: &str) -> JsonResult<Parameters> {
  let json_string = read_to_string(Path::new(file_path))?.as_str();
  let object = parse_json(json_string)?;
  let mut parameters = HashMap::<&'static str, Parameter>::new();

  if let JsonValue::Array(records) = object["parameters"]?{
    for record in records {
      let key = record["param"].as_str()?;
      let parameter =
          Parameter {
            name: key,
            value: json_value_to_parameter_value(record["type"].as_str()?, &record["default"])?,
            description: record["description"].as_str()?
          };

      parameters[key] = parameter;
    }
  } else {
    return Err(
              JsonError::wrong_type(
              format!("Expected parameters to be a list, got {}.", object["parameters"]).as_str()
              )
            );
  }

  Ok(
    Parameters{
      module: object["module"].as_str()?,
      export: object["export"].as_bool()?,
      description: object["description"].as_str()?,
      parameters
    }
  )
}


static mut GLOBAL_PARAMETERS: HashMap<&str, ParametersRef> = HashMap::new();
static PARAMETER_PATHS  : HashMap<&str, &str> = container!{
  "sat" => "../../resources/sat_params"
};

/// Lazily deserializes the parameters for `module` when they are requested.
pub fn get_global_parameters(module: &str) -> Result<ParametersRef, dyn Error> {
  unsafe {
    match GLOBAL_PARAMETERS.get(module) {

      None => {
        let path: &str = match PARAMETER_PATHS.get(module) {
                     None       => Err(Error::DeserializeParameters),
                     Some(&path) => Ok(path)
                   }?;
        let parameters    : Parameters    = deserialize_parameters(path)?;
        let parameters_ref: ParametersRef = Rc::new(RefCell::new(parameters));

        GLOBAL_PARAMETERS.insert(module, parameters_ref.clone());

        Ok(parameters_ref)
      }

      Some(&parameters_ref) => Ok(parameters_ref)

    }
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn get_params() {
    let p    : Result<ParametersRef, dyn Error> = get_global_parameters("sat");
    let p_ref: ParametersRef = p.unwrap();
    let param: &Parameter    = &p_ref.borrow()["restart.emafastglue"];

    assert_eq!(param.value, ParameterValue::Double(3e-2))
  }
}
