Determines whether a linear constraint is treated as a lazy constraint or a user cut.

At the beginning of the MIP solution process, any constraint whose `Lazy` attribute is set to 1, 2, or 3 (the default
value is 0) is treated as a lazy constraint; it is removed from the model and placed in the lazy constraint pool. `Lazy`
constraints remain inactive until a feasible solution is found, at which point the solution is checked against the lazy
constraint pool. If the solution violates any lazy constraints, the solution is discarded and one or more of the
violated lazy constraints are pulled into the active model.

Larger values for this attribute cause the constraint to be pulled into the model more aggressively. With a value of 1,
the constraint can be used to cut off a feasible solution, but it won't necessarily be pulled in if another lazy
constraint also cuts off the solution. With a value of 2, all lazy constraints that are violated by a feasible solution
will be pulled into the model. With a value of 3, lazy constraints that cut off the relaxation solution at the root node
are also pulled in.

Any constraint whose `Lazy` attribute is set to -1 is treated as a user cut; it is removed from the model and placed in
the user cut pool. User cuts may be added to the model at any node in the branch-and-cut search tree to cut off
relaxation solutions.

The main difference between user cuts and lazy constraints is that that the former are not allowed to cut off integer-
feasible solutions. In other words, they are redundant for the MIP model, and the solver is free to decide whether or
not to use them to cut off relaxation solutions. The hope is that adding them speeds up the overall solution process.
`Lazy` constraints have no such restrictions. They are essential to the model, and the solver is forced to apply them
whenever a solution would otherwise not satisfy them.

Note that deleting constraints from your model will cause this attribute to be discarded. If you'd like it to persist,
your program will need to repopulate it after deleting the constraints and making a subsequent model update call.

Note that no corresponding attribute is available for other constraint types (quadratic, SOS, or general constraints).
This attribute only affects MIP models.