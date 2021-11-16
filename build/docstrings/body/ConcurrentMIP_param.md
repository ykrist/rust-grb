This parameter enables the concurrent MIP solver. When the parameter is set to value n, the MIP solver performs n
independent MIP solves in parallel, with different parameter settings for each. Optimization terminates when the first
solve completes.

By default, Gurobi chooses the parameter settings used for each independent solve automatically. You can create
concurrent environments to choose your own parameter settings (refer to the concurrent optimization section for
details). The intent of concurrent MIP solving is to introduce additional diversity into the MIP search. This approach
can sometimes solve models much faster than applying all available threads to a single MIP solve, especially on very
large parallel machines.

The concurrent MIP solver divides available threads evenly among the independent solves. For example, if you have 6
threads available and you set `ConcurrentMIP` to 2, the concurrent MIP solver will allocate 3 threads to each
independent solve. Note that the number of independent solves launched will not exceed the number of available threads.

The concurrent MIP solver produces a slightly different log from the standard MIP solver, and provides different
callbacks as well. Please refer to the concurrent optimizer discussion for additional details.

Concurrent MIP is not deterministic. If runtimes for different independent solves are very similar, and if the model has
multiple optimal solutions, you may get slightly different results from multiple runs on the same model.

Note: Only affects mixed integer programming (MIP) models