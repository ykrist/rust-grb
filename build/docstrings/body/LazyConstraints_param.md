Programs that add lazy constraints through a callback must set this parameter to value 1. The parameter tells the Gurobi
algorithms to avoid certain reductions and transformations that are incompatible with lazy constraints.

Note that if you use lazy constraints by setting the Lazy attribute (and not through a callback), there's no need to set
this parameter.

Note: Only affects mixed integer programming (MIP) models