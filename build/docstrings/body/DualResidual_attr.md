Reporting dual constraint violations for the simplex solver is actually more complex than it may appear, due to the
treatment of reduced costs for bounded variables. The simplex solver introduces explicit non-negative reduced-cost
variables inside the algorithm. Thus, $a^Ty \ge c$ becomes $a^Ty - z = c$ (where $y$ is the dual vector and $z$ is the
reduced cost). In this formulation, errors can show up in two places: (i) as bound violations on the computed reduced-
cost variable values, and (ii) as differences between $a^Ty - z$ and $c$. We report the former as `DualVio` and the
latter as DualResidual.

`DualResidual` reports the maximum (unscaled) dual constraint error.

Only available for continuous models.