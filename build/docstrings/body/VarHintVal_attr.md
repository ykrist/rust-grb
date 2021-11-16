A set of user hints. If you know that a variable is likely to take a particular value in high quality solutions of a MIP
model, you can provide that value as a hint. You can also (optionally) provide information about your level of
confidence in a hint with the `VarHintPri` attribute.

The Gurobi MIP solver will use these variable hints in a number of different ways. Hints will affect the heuristics that
Gurobi uses to find feasible solutions, and the branching decisions that Gurobi makes to explore the MIP search tree. In
general, high quality hints should produce high quality MIP solutions faster. In contrast, low quality hints will lead
to some wasted effort, but shouldn't lead to dramatic performance degradations.

Variables hints and MIP starts are similar in concept, but they behave in very different ways. If you specify a MIP
start, the Gurobi MIP solver will try to build a single feasible solution from the provided set of variable values. If
you know a solution, you should use a MIP start to provide it to the solver. In contrast, variable hints provide
guidance to the MIP solver that affects the entire solution process. If you have a general sense of the likely values
for variables, you should provide them through variable hints.

If you wish to leave the hint value for a variable undefined, you can either avoid setting the `VarHintVal` attribute
for that variable, or you can set it to a special undefined value (GRB_UNDEFINED in C and C++, GRB.UNDEFINED in Java,
.NET, and Python, NA in R or nan in Matlab).

Note that deleting variables from your model will cause several attributes to be discarded (variable hints and branch
priorities). If you'd like them to persist, your program will need to repopulate them after deleting the variables and
making a subsequent model update call.

Only affects MIP models.