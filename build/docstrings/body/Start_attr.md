The current MIP start vector. The MIP solver will attempt to build an initial solution from this vector when it is
available. Note that the start can be partially populated -- the MIP solver will attempt to fill in values for missing
start values. If you wish to leave the start value for a variable undefined, you can either avoid setting the `Start`
attribute for that variable, or you can set it to a special undefined value (GRB_UNDEFINED in C and C++, or
GRB.UNDEFINED in Java, .NET, and Python).

If the Gurobi MIP solver log indicates that your MIP start didn't produce a new incumbent solution, note that there can
be multiple explanations. One possibility is that your MIP start is infeasible. Another, more common possibility is that
one of the Gurobi heuristics found a solution that is as good as the solution produced by the MIP start, so the MIP
start solution was cut off. Finally, if you specified a partial MIP start, it is possible that the limited MIP
exploration done on this partial start was insufficient to find a new incumbent solution. You can try setting the
StartNodeLimit parameter to a larger value if you want Gurobi to work harder to try to complete the partial start.

If you solve a sequence of models, where one is built by modifying the previous one, and if you don't provide a MIP
start, then Gurobi will try to construct one automatically from the solution of the previous model. If you don't want it
to try this, you should reset the model before starting the subsequent solve. If you provided a MIP start but would
prefer to use the previous solution as the start instead, you should clear your start (by setting the `Start` attribute
to undefined for all variables).

If you have multiple start vectors, you can provide them to Gurobi by using the `Start` attribute in combination with
the `NumStart` attribute and the StartNumber parameter. Specifically, use the `NumStart` attribute to indicate how many
start vectors you will supply. Then set the StartNumber parameter to a value between 0 and NumStart-1 to indicate which
start you are supplying. For each value of StartNumber, populate the `Start` attribute to supply that start. Gurobi will
use all of the provided starts. As an alternative, you can append new MIP start vectors to your model by setting the
StartNumber parameter to -1. In this case, whenever you read a MIP start, or use a function to set a MIP start value for
a set of variables, a new MIP start will be created, the parameter `NumStart` will be increased, and any unspecified
variable will be left as undefined.

If you want to diagnose an infeasible MIP start, you can try fixing the variables in the model to their values in your
MIP start (by setting their lower and upper bound attributes). If the resulting MIP model is infeasible, you can then
compute an IIS on this model to get additional information that should help to identify the cause of the infeasibility.

Only affects MIP models.