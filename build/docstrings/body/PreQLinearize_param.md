Controls presolve Q matrix linearization. Binary variables in quadratic expressions provide some freedom to state the
same expression in multiple different ways. Options 1 and 2 of this parameter attempt to linearize quadratic constraints
or a quadratic objective, replacing quadratic terms with linear terms, using additional variables and linear
constraints. This can potentially transform an MIQP or MIQCP model into an MILP. Option 1 focuses on producing an MILP
reformulation with a strong LP relaxation, with a goal of limiting the size of the MIP search tree. Option 2 aims for a
compact reformulation, with a goal of reducing the cost of each node. Option 0 attempts to leave Q matrices unmodified;
it won't add variables or constraints, but it may still perform adjustments on quadratic objective functions to make
them positive semi-definite (PSD). The default setting (-1) chooses automatically.

Note: Only affects MIQP and MIQCP models