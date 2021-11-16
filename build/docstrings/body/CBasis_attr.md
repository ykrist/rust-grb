The status of a given linear constraint in the current basis. Possible values are 0 (basic) or -1 (non-basic). A
constraint is basic when its slack variable is in the simplex basis. Note that, if you wish to specify an advanced
starting basis, you must set basis status information for all constraints and variables in the model. Only available for
basic solutions.

Note that if you provide a valid starting extreme point, either through PStart, DStart, or through VBasis, CBasis, then
LP presolve will be disabled by default. For models where presolve greatly reduces the problem size, this might hurt
performance. To allow presolve, it needs to set parameter LPWarmStart to 2.