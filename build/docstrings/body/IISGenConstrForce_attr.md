When computing an Irreducible Inconsistent Subsystem (IIS) for an infeasible model, indicates whether the general
constraint should be included or excluded from the IIS.

The default value of -1 lets the IIS algorithm decide.

If the attribute is set to 0, the constraint is not eligible for inclusion in the IIS.

If the attribute is set to 1, the constraint is included in the IIS and the IIS algorithm never considers the
possibility of removing it.

Note that setting this attribute to 0 may make the resulting subsystem feasible (or consistent), which would then make
it impossible to construct an IIS. Trying anyway will result in a GRB_ERROR_IIS_NOT_INFEASIBLE error. Similarly, setting
this attribute to 1 may result in an IIS that is not irreducible. More precisely, the system would only be irreducible
with respect to the model elements that have force values of -1 or 0.

See the Model.computeIIS documentation for more details.