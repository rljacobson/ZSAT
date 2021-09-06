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
use term::terminfo::Error::IoError;

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
  name         : &'static str,
  default_value: ParameterValue<'s>,
  description  : &'static str
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Parameters<'s> {
  module     : &'s str,
  export     : bool,      // todo: Is this relevant? Kept it from z3.
  description: &'s str,
  parameters : HashMap<&'s str, Parameter<'s>>
}

impl<'s> Parameters<'s>{
  // todo: Why have a getter for every variant of `ParameterValue`?
  pub fn get_symbol(&self, symbol: &str) -> Option<ParameterValue> {
    self.parameters
        .get(symbol)
        .and_then(| v | Some(v.default_value))
  }
}

fn json_value_to_parameter_value(datatype: &str, json_value: &JsonValue) -> JsonResult<ParameterValue> {
  match datatype {

    "UINT"   => Ok(ParameterValue::UnsignedInteger(json_value.as_u64()?)),

    "BOOL"   => Ok(ParameterValue::Bool(json_value.as_bool()?)),

    "DOUBLE" => Ok(ParameterValue::Double(json_value.as_f64()?)),

    "SYMBOL" => Ok(ParameterValue::Symbol(json_value.as_str()?)),

    _other   => Err(
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
            default_value: json_value_to_parameter_value(record["type"].as_str()?, &record["default"])?,
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
      module: &object["module"].as_str()?,
      export: object["export"].as_bool()?,
      description: &object["description"].as_str()?,
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
        let path = match PARAMETER_PATHS.get(module) {
                     None       => Err(Error::DeserializeParameters),
                     Some(path) => Ok(path)
                   }?;
        let parameters     = deserialize_parameters(path)?;
        let parameters_ref = Rc::new(RefCell::new(parameters));
        GLOBAL_PARAMETERS.insert(module, parameters_ref);

        Ok(parameters_ref.clone())
      }

      Some(parameters_ref) => Ok(parameters_ref.clone())

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
