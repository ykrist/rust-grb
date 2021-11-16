This attribute is used to query the objective value obtained for objective $n$ by the $k$-th solution stored in the pool
of feasible solutions found so far for the problem. You set $n$ using the ObjNumber parameter, while you set $k$ using
the SolutionNumber parameter.

The number of objectives in the model can be queried (or modified) using the `NumObj` attribute; while the number of
stored solutions can be queried using the `SolCount` attribute.

Please refer to the discussion of Multiple Objectives for more information on the use of alternative objectives.