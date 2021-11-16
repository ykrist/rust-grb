Controls whether and how Gurobi uses warm start information for an LP optimization. The non default setting of 2 is
particularly useful for communicating advanced start information while retaining the performance benefits of presolve. A
warm start can consist of any combination of basis statuses, a primal start vector, or a dual start vector. It is
specified using the attributes VBasis and CBasis or PStart and DStart on the original model.

As a general rule, setting this parameter to 0 ignores any start information and solves the model from scratch. Setting
it to 1 (the default) uses the provided warm start information to solve the original, unpresolved problem, regardless of
whether presolve is enabled. Setting it to 2 uses the start information to solve the presolved problem, assuming that
presolve is enabled. This involves mapping the solution of the original problem into an equivalent (or sometimes nearly
equivalent) crushed solution of the presolved problem. If presolve is disabled, then setting 2 still prioritizes start
vectors, while setting 1 prioritizes basis statuses. Taken together, the `LPWarmStart` parameter setting, the LP
algorithm specified by Gurobi's `Method` parameter, and the available advanced start information determine whether
Gurobi will use basis statuses only, basis statuses augmented with information from start vectors, or a basis obtained
by applying the crossover method to the provided primal and dual start vectors to jump start the optimization.

When Gurobi's `Method` parameter requests the barrier solver, primal and dual start vectors are prioritized over basis
statuses (but only if you provide both). These start vectors are fed to the crossover procedure. This is the same
crossover that is used to compute a basic solution from the interior solution produced by the core barrier algorithm,
but in this case crossover is started from arbitrary start vectors. If you set the `LPWarmStart` parameter to 1,
crossover will be invoked on the original model using the provided vectors. Any provided basis information will not be
used in this case. If you set `LPWarmStart` to 2, crossover will be invoked on the presolved model using crushed start
vectors. If you set the parameter to 2 and provide a basis but no start vectors, the basis will be used to compute the
corresponding primal and dual solutions on the original model. Those solutions will then be crushed and used as primal
and dual start vectors for the crossover, which will then construct a basis for the presolved model. Note that for all
of these settings and start combinations, no barrier algorithm iterations are performed.

The simplex algorithms provide more warm-starting options, With a parameter value of 1, simplex will start from a
provided basis, if available. Otherwise, it uses a provided start vector to refine the crash basis it computes. Primal
simplex will use PStart and dual simplex will use DStart in this refinement process.

With a value of 2, simplex will use the crushed start vector on the presolved model (PStart for primal simplex, DStart
for dual) to refine the crash basis. This is true regardless of whether the start is derived from start vectors or a
starting basis from the original model. The difference is that if you provide an advanced basis, the basis will be used
to compute the corresponding primal and dual solutions on the original model from which the primal or dual start on the
presolved model will be derived.

Note: Only affects linear programming (LP) models