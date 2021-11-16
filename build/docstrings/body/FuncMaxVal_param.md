Very large values in piecewise-linear approximations can cause numerical issues. This parameter limits the bounds on the
variables that participate in function constraints. Specifically, if $x$ or $y$ participate in a function constraint,
any bound larger than `FuncMaxVal` (in absolute value) will be truncated.