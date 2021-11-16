When solving a MIP model, the Gurobi Optimizer maintains a solution pool that contains the best solutions found during
the search. The `PoolIgnore` attribute allows you to discard some solutions. Specifically, if multiple solutions differ
only in variables where `PoolIgnore` is set to 1, only the solution with the best objective will be kept in the pool.
The default value for the attribute is 0, meaning that the variable should be used to distinguish solutions.

This attribute is particularly helpful when used in conjunction with the PoolSearchMode parameter. By identifying
variables that do not capture meaningful differences between solutions, you can make sure that the pool contains some
interesting variety.