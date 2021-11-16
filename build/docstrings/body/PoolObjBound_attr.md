Bound on the objective of undiscovered MIP solutions. The MIP solver stores solutions that it finds during the MIP
search, but it only provides quality guarantees for those whose objective is at least as good as PoolObjBound.
Specifically, further exploration of the MIP search tree will not find solutions whose objective is better than
PoolObjBound.

The difference between `PoolObjBound` and `ObjBound` is that the former gives an objective bound for undiscovered
solutions, while the latter gives a bound for any solution. Note that `PoolObjBound` and `ObjBound` can only have
different values if parameter PoolSearchMode is set to 2.

Please consult the section on Solution Pools for a more detailed discussion of this topic.