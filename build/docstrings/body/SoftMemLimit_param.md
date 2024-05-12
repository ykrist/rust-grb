Limits the total amount of memory (in GB, i.e., $10^9$ bytes) available to Gurobi. If more is needed, Gurobi will
terminate with a MEM_LIMIT status code (see the Status Code section for further details).

In contrast to the `MemLimit` parameter, the `SoftMemLimit` parameter leads to a graceful exit of the optimization, such
that it is possible to retrieve solution information afterwards or (in the case of a MIP solve) resume the optimization.

A disadvantage compared to `MemLimit` is that the `SoftMemLimit` is only checked at places where optimization can be
terminated gracefully, so memory use may exceed the limit between these checks.

Note that allocated memory is tracked across all models within a Gurobi environment. If you create multiple models in
one environment, these additional models will count towards overall memory consumption.

Memory usage is also tracked across all threads. One consequence of this is that termination may be non-deterministic
for multi-threaded runs.