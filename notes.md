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
