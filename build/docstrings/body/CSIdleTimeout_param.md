This parameter allows you to set a limit on how long a Compute Server job can sit idle before the server kills the job
(in seconds). A job is considered idle if the server is not currently performing an optimization and the client has not
issued any additional commands.

The default value will allow a job to sit idle indefinitely in all but one circumstance. Currently the only exception is
the Gurobi Instant Cloud, where the default setting will automatically impose a 30 minute idle time limit (1800
seconds). If you are using an Instant Cloud pool, the actual value will be the maximum between this parameter value and
the idle timeout defined by the pool.

You must set this parameter through either a gurobi.lic file (using IDLETIMEOUT=n) or an empty environment. Changing the
parameter after your environment has been created will have no effect.

Refer to the Gurobi Remote Services Reference Manual for more information on starting Compute Server jobs.