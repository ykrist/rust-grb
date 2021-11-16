The status of a given variable in the current basis. Possible values are 0 (basic), -1 (non-basic at lower bound), -2
(non-basic at upper bound), and -3 (super-basic). Note that, if you wish to specify an advanced starting basis, you must
set basis status information for all constraints and variables in the model. Only available for basic solutions.

Note that if you provide a valid starting extreme point, either through PStart, DStart, or through VBasis, CBasis, then
LP presolve will be disabled by default. For models where presolve greatly reduces the problem size, this might hurt
performance. To allow presolve, it needs to set parameter LPWarmStart to 2.