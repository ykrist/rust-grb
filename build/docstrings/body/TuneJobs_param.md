Enables distributed parallel tuning, which can significantly increase the performance of the tuning tool. A value of n
causes the tuning tool to distribute tuning work among n parallel jobs. These jobs are distributed among a set of
machines. Use the `WorkerPool` parameter to provide a distributed worker cluster.

Note that distributed tuning is most effective when the worker machines have similar performance. Distributed tuning
doesn't attempt to normalize performance by server, so it can incorrectly attribute a boost in performance to a
parameter change when the associated setting is tried on a worker that is significantly faster than the others.