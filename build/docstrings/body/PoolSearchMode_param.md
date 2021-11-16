Selects different modes for exploring the MIP search tree. With the default setting (PoolSearchMode=0), the MIP solver
tries to find an optimal solution to the model. It keeps other solutions found along the way, but those are incidental.
By setting this parameter to a non-default value, the MIP search will continue after the optimal solution has been found
in order to find additional, high-quality solutions. With a non-default value (PoolSearchMode=1 or PoolSearchMode=2),
the MIP solver will try to find n solutions, where n is determined by the value of the `PoolSolutions` parameter. With a
setting of 1, there are no guarantees about the quality of the extra solutions, while with a setting of 2, the solver
will find the n best solutions. The cost of the solve will increase with increasing values of this parameter.

Once optimization is complete, the PoolObjBound attribute can be used to evaluate the quality of the solutions that were
found. For example, a value of PoolObjBound=100 indicates that there are no other solutions with objective better 100,
and thus that any known solutions with objective better than 100 are better than any as-yet undiscovered solutions.

Note: Only affects mixed integer programming (MIP) models