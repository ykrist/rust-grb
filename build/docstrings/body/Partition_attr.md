Variable partition. The MIP solver can perform a solution improvement heuristic using user-provided partition
information. The provided partition number can be positive, which indicates that the variable should be included when
the correspondingly numbered sub-MIP is solved, 0 which indicates that the variable should be included in every sub-MIP,
or -1 which indicates that the variable should not be included in any sub-MIP. Variables that are not included in the
sub-MIP are fixed to their values in the current incumbent solution. By default, all variables start with a value of -1.

To give an example, imagine you are solving a model with 400 variables and you set the partition attribute to -1 for
variables 0-99, 0 for variables 100-199, 1 for variables 200-299, and 2 for variables 300-399. The heuristic would solve
two sub-MIP models: sub-MIP 1 would fix variables 0-99 and 300-399 to their values in the incumbent and solve for the
rest, while sub-MIP 2 would fix variables 0-99 and 200-299.

Use the PartitionPlace parameter to control where the partition heuristic runs.

Only affects MIP models.