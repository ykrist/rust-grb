Controls symmetry detection. A value of -1 corresponds to an automatic setting. Other options are off (0), conservative
(1), or aggressive (2).

`Symmetry` can impact a number of different parts of the algorithm, including presolve, the MIP tree search, and the LP
solution process. Default settings are quite effective, so changing the value of this parameter rarely produces a
significant benefit.