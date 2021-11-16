Determines the format of the presolved version of an MIQCP model. Option 0 leaves the model in MIQCP form, so the
branch-and-cut algorithm will operate on a model with arbitrary quadratic constraints. Option 1 always transforms the
model into MISOCP form; quadratic constraints are transformed into second-order cone constraints. Option 2 always
transforms the model into disaggregated MISOCP form; quadratic constraints are transformed into rotated cone
constraints, where each rotated cone contains two terms and involves only three variables.

The default setting (-1) choose automatically. The automatic setting works well, but there are cases where forcing a
different form can be beneficial.

Note: Only affects MIQCP models