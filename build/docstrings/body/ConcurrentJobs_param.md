Enables distributed concurrent optimization, which can be used to solve LP or MIP models on multiple machines. A value
of n causes the solver to create n independent models, using different parameter settings for each. Each of these models
is sent to a distributed worker for processing. Optimization terminates when the first solve completes. Use the
`ComputeServer` parameter to indicate the name of the cluster where you would like your distributed concurrent job to
run (or use `WorkerPool` if your client machine will act as manager and you just need a pool of workers).

By default, Gurobi chooses the parameter settings used for each independent solve automatically. You can create
concurrent environments to choose your own parameter settings (refer to the concurrent optimization section for
details). The intent of concurrent MIP solving is to introduce additional diversity into the MIP search. By bringing the
resources of multiple machines to bear on a single model, this approach can sometimes solve models much faster than a
single machine.

The distributed concurrent solver produces a slightly different log from the standard solver, and provides different
callbacks as well. Please refer to the Distributed Algorithms section of the Gurobi Remote Services Reference Manual for
additional details.