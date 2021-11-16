If you find that the Gurobi optimizer exhausts memory when solving a MIP, you should modify the `NodefileStart`
parameter. When the amount of memory used to store nodes (measured in GB, i.e., $10^9$ bytes) exceeds the specified
parameter value, nodes are compressed and written to disk. We recommend a setting of 0.5, but you may wish to choose a
different value, depending on the memory available in your machine. By default, nodes are written to the current working
directory. The `NodefileDir` parameter can be used to choose a different location.

If you still exhaust memory after setting the `NodefileStart` parameter to a small value, you should try limiting the
thread count. Each thread in parallel MIP requires a copy of the model, as well as several other large data structures.
Reducing the `Threads` parameter can sometimes significantly reduce memory usage.

Note: Only affects mixed integer programming (MIP) models