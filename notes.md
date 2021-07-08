# Show Stoppers

###  `LocalSearch::import`

The `Extension` part of the `LocalSearch::import` is broken, because `extract_pb()` as currently designed requires two mutable borrows of `self`. It's a weird design, anyway, because it is a function that takes two closures. 

### Hierarchical inheritance: `Solver` is a `local_search` is a `LocalSearchCore`

`LocalSearchCore` is an interface. `LocalSearch` implements that interface. `Solver` elaborates on `LocalSearch`. Could possibly use composition, but it's not clear how that affects other subclasses of `LocalSearch`. 

These dependencies are weird, as `LocalSearch` references `Solver` explicitly in `LocalSearch::add()`, `LocalSearch::reinit_with_solver()`,  and `LocalSearch::import()`.

## Known issues

`LocalSearch::add_cardinality` was changed to take a vector rather than a pointer and size, but the callers have not been adjusted. It should probably take a slice instead, too.



# Redundancies

## Boolean variables and values

### Singular

| Name         | Values                                  | Repr      |
|:-------------|:----------------------------------------|:----------|
| `bool`       | `true`<br>`false`                       | primitive |
| `LiftedBool` | `False=-1`<br>`True=1`<br>`Undefined=0` | Enum      |
| `Theory`     | ??                                      | `i32`     |

### Meta Variables

| Name            | Values            | Repr                     | Description                                 |
|:----------------|:------------------|:-------------------------|:--------------------------------------------|
| `BoolVariable`  | any `u32` value   | `u32`                    | An identifier of a variable in the program. |
| `Literal`       | any `u32` value   | `Literal(BoolVariable)`  | `BoolVariable` & value                      |
| `DimacsLiteral` | Same as `Literal` | `DimacsLiteral(Literal)` | Newtype of `Literal`                        |

### Plural


| Name              | Values       | Repr  |                                                      |
|:------------------|:-------------|:------|:-----------------------------------------------------|
| `LiteralVector`   | ??           | `Vec` |                                                      |
| `Vec<LiftedBool>` | `LiftedBool` | `Vec` | assignment from `BoolVariable` index to `LiftedBool` |
| `Model`           | `LiftedBool` | `Vec` | Holds a `Vec<LiftedBool>`                            |

# Dictionary of terms

| z3                                                          | zsat                         |
|:------------------------------------------------------------|:-----------------------------|
| `lbool`                                                     | `LiftedBool`                 |
| `default_exception`, ,                                      | `errors::Error`              |
| `sat_param_exception`                                       | `errors::Error`              |
| `solver_exception`                                          | `errors::Error`              |
| `bool_var`                                                  | `BoolVariable`               |
| `bool_var_vector`                                           | `BoolVariableVector`         |
| `ext_constraint_idx`                                        | `ExternalConstraintIndex`    |
| `ext_justification_idx`                                     | `ExternalJustificationIndex` |
| `literal_approx_set`                                        | `LiteralApproximateSet`      |
| `var_approx_set`                                            | `VariableApproximateSet`     |
| `negate`                                                    | `negate_literals`            |
|                                                             |                              |
|                                                             |                              |
