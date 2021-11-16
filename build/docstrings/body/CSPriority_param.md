The priority of the Compute Server job. Priorities must be between -100 and 100, with a default value of 0 (by
convention). Higher priority jobs are chosen from the server job queue before lower priority jobs. A job with priority
100 runs immediately, bypassing the job queue and ignoring the job limit on the server. You should exercise caution with
priority 100 jobs, since they can severely overload a server, which can cause jobs to fail, and in extreme cases can
cause the server to crash.

Refer to the Gurobi Remote Services Reference Manual for more information on starting Compute Server jobs.

You must set this parameter through either a gurobi.lic file (using PRIORITY=n) or an empty environment. Changing the
parameter after your environment has been created will have no effect.