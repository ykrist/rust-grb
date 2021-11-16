Reporting constraint violations for the simplex solver is actually more complex than it may appear, due to the treatment
of slacks on linear inequality constraints. The simplex solver introduces explicit non-negative slack variables inside
the algorithm. Thus, for example, $a^Tx \le b$ becomes $a^Tx + s = b$. In this formulation, constraint errors can show
up in two places: (i) as bound violations on the computed slack variable values, and (ii) as differences between $a^Tx +
s$ and $b$. We report the former as `ConstrVio` and the latter as ConstrResidual.

Only available for continuous models. For MIP models, constraint violations are reported in ConstrVio.