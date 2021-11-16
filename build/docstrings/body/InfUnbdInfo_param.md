Determines whether simplex (and crossover) will compute additional information when a model is determined to be
infeasible or unbounded. Set this parameter if you want to query the unbounded ray for unbounded models (through the
UnbdRay attribute), or the infeasibility proof for infeasible models (through the FarkasDual and FarkasProof
attributes).

Note that if a model is found to be either infeasible or unbounded, and you simply want to know which one it is, you
should use the `DualReductions` parameter instead. It performs much less additional computation.

Note: LP only