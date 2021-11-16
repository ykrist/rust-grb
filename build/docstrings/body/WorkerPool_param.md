When using a distributed algorithm (distributed MIP, distributed concurrent, or distributed tuning), this parameter
allows you to specify a Remote Services cluster that will provide distributed workers. You should also specify the
access password for that cluster, if there is one, in the `WorkerPassword` parameter. Note that you don't need to set
either of these parameters if your job is running on a Compute Server node and you want to use the same cluster for the
distributed workers.

You can provide a comma-separated list of machines for added robustness. If the first node in the list is unavailable,
the client will attempt to contact the second node, etc.

To give an example, if you have a Remote Services cluster that uses port 61000 on a pair of machines named server1 and
server2, you could set `WorkerPool` to "server1:61000" or "server1:61000,server2:61000".