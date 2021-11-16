Modifies the tuning criterion for the tuning tool. The primary tuning criterion is always to minimize the runtime
required to find a proven optimal solution. However, for MIP models that don't solve to optimality within the specified
time limit, a secondary criterion is needed. Set this parameter to 1 to use the optimality gap as the secondary
criterion. Choose a value of 2 to use the objective of the best feasible solution found. Choose a value of 3 to use the
best objective bound. Choose 0 to ignore the secondary criterion and focus entirely on minimizing the time to find a
proven optimal solution. The default value of -1 chooses automatically.

Note that for multi-objective problems value 1 and 3 are unsupported. See the Multiple Objectives section for more
details on this.