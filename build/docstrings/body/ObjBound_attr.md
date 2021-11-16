The best known bound on the optimal objective. When solving a MIP model, the algorithm maintains both a lower bound and
an upper bound on the optimal objective value. For a minimization model, the upper bound is the objective of the best
known feasible solution, while the lower bound gives a bound on the best possible objective.

In contrast to ObjBoundC, this attribute takes advantage of objective integrality information to round to a tighter
bound. For example, if the objective is known to take an integral value and the current best bound is 1.5, `ObjBound`
will return 2.0 while `ObjBoundC` will return 1.5.