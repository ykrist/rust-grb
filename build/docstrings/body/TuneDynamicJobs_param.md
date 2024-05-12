Enables distributed parallel tuning, which can significantly increase the performance of the tuning tool. A value of n
causes the tuning tool to use a dynamic set of up to n workers in parallel. These workers are used for a limited amount
of time and afterwards potentially released so that they are available for other remote jobs. A value of -1 allows the
solver to use an unlimited number of workers. Note that this parameter can be combined with `TuneJobs` to get a static
set of workers and a dynamic set of workers for distributed tuning. You can use the `WorkerPool` parameter to provide a
distributed worker cluster.

Note that distributed tuning is most effective when the worker machines have similar performance. Distributed tuning
doesn't attempt to normalize performance by server, so it can incorrectly attribute a boost in performance to a
parameter change when the associated setting is tried on a worker that is significantly faster than the others.