# Show Stoppers

###  `LocalSearch::import`

The `Extension` part of the `LocalSearch::import` is broken, because `extract_pb()` as currently designed requires two mutable borrows of `self`. It's a weird design, anyway, because it is a function that takes two closures.

### Hierarchical inheritance: `Solver` is a `solver_core`, `local_search` is a `LocalSearchCore`

`LocalSearchCore` is an interface. `LocalSearch` implements that interface. `Solver` elaborates on
`SolverCore`.

## Symbols & Parameters

| Module          | Type                     | Description                                                                   |
| :-------------- | :----------------------- | :---------------------------------------------------------------------------- |
| parameters.rs   | `Parameters`             | Metadata and `HashMap<&'s str, Parameter<'s>>`                                |
| parameters.rs   | `Parameter`              | Name, description, and `ParameterValue<'s>`                                   |
| parameters.rs   | `ParameterValue`         | Enum wrapper for `u64`, `f64`, `bool`, `&str` called `ParameterValue::Symbol` |
| parameters.rs   | `ParameterValue::Symbol` | A variant of the `ParameterValue` enum                                        |
| parameters.rs   | `ParametersRef`          | Type def for `Rc<RefCell<Parameters<'s>>>`                                    |
| symbol_table.rs | `SymbolTable`            | Type def of `HashIndexing<SymbolData<'s>, usize>`                             |
| symbol_table.rs | `Symbol`                 | Type def for `usize`                                                          |
| symbol_table.rs | `SymbolData`             | Enum wrapper for `&str`, `i64`, and a null variant, `SymbolData::Null`        |
| symbol_map      | `HashIndexing<T, D>`     | Fast bidirectional lookup                                                     |

## Known issues

`LocalSearch::add_cardinality` was changed to take a vector rather than a pointer and size, but the callers have not been adjusted. It should probably take a slice instead, too.



# Redundancies

## Boolean variables and values

### Singular

| Name         | Values                                  | Repr      |
| :----------- | :-------------------------------------- | :-------- |
| `bool`       | `true`<br>`false`                       | primitive |
| `LiftedBool` | `False=-1`<br>`True=1`<br>`Undefined=0` | Enum      |
| `Theory`     | ??                                      | `i32`     |

### Meta Variables

| Name            | Values            | Repr                     | Description                                 |
| :-------------- | :---------------- | :----------------------- | :------------------------------------------ |
| `BoolVariable`  | any `u32` value   | `u32`                    | An identifier of a variable in the program. |
| `Literal`       | any `u32` value   | `Literal(BoolVariable)`  | `BoolVariable` & value                      |
| `DimacsLiteral` | Same as `Literal` | `DimacsLiteral(Literal)` | Newtype of `Literal`                        |

### Plural


| Name              | Values       | Repr  |                                                      |
| :---------------- | :----------- | :---- | :--------------------------------------------------- |
| `LiteralVector`   | ??           | `Vec` |                                                      |
| `Vec<LiftedBool>` | `LiftedBool` | `Vec` | assignment from `BoolVariable` index to `LiftedBool` |
| `Model`           | `LiftedBool` | `Vec` | Holds a `Vec<LiftedBool>`                            |


# Generated files

The z3 build system uses a Python script to generate source files. The input to the script are *.PYG
files (PYthon Generated files). ZSat uses standard JSON as input instead.

Configuration parameters in the z3 code base are defined in *.pyg files and generated at
compile time. In contrast, we read in the parameter database from a JSON file at runtime.


# Dictionary of terms

| z3                      | zsat                         |
| :---------------------- | :--------------------------- |
| `lbool`                 | `LiftedBool`                 |
| `default_exception`     | `errors::Error`              |
| `sat_param_exception`   | `errors::Error`              |
| `solver_exception`      | `errors::Error`              |
| `bool_var`              | `BoolVariable`               |
| `bool_var_vector`       | `BoolVariableVector`         |
| `ext_constraint_idx`    | `ExternalConstraintIndex`    |
| `ext_justification_idx`  | `ExternalJustificationIndex`  |
| `literal_approx_set`    | `LiteralApproximateSet`      |
| `var_approx_set`        | `VariableApproximateSet`     |
| `negate`                | `negate_literals`            |
| `mus`                   | `MinimalUnsatisfiableSet`     |
| `SASSERT`               | `log_assert!`                |
