Enables distributed MIP. A value of n causes the MIP solver to divide the work of solving a MIP model among n machines.
Use the `ComputeServer` parameter to indicate the name of the cluster where you would like your distributed MIP job to
run (or use `WorkerPool` if your client machine will act as manager and you just need a pool of workers).

The distributed MIP solver produces a slightly different log from the standard MIP solver, and provides different
callbacks as well. Please refer to the Distributed Algorithms section of the Gurobi Remote Services Reference Manual for
additional details.

Note: Only affects mixed integer programming (MIP) models