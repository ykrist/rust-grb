The current simplex start vector. If you set `PStart` values for every variable in the model and `DStart` values for
every constraint, then simplex will use those values to compute a warm start basis. If you'd like to retract a
previously specified start, set any `PStart` value to GRB_UNDEFINED.

Note that any model modifications which are pending or are made after setting `PStart` (adding variables or constraints,
changing coefficients, etc.) will discard the start. You should only set this attribute after you are done modifying
your model.

Note also that you'll get much better performance if you warm start your linear program using a simplex basis (using
`VBasis` and CBasis). The `PStart` attribute should only be used in situations where you don't have a basis or you don't
want to disable presolve.

If you'd like to provide a feasible starting solution for a MIP model, you should input it using the `Start` attribute.

Only affects LP models; it will be ignored for QP, QCP, or MIP models.

Note that if you provide a valid starting extreme point, either through PStart, DStart, or through VBasis, CBasis, then
LP presolve will be disabled by default. For models where presolve greatly reduces the problem size, this might hurt
performance. To allow presolve, it needs to set parameter LPWarmStart to 2.