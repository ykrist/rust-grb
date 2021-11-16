This parameter allows you to set a limit (in seconds) on how long a new Compute Server job will wait in queue before it
gives up (and reports a JOB_REJECTED error). Note that there might be a delay of up to 20 seconds for the actual
signaling of the time out.

Any negative value will allow a job to sit in the Compute Server queue indefinitely.

You must set this parameter through a gurobi.lic file (using QUEUETIMEOUT=n) or an empty environment. Changing the
parameter after your environment has been created will have no effect.

Refer to the Gurobi Remote Services Reference Manual for more information on starting Compute Server jobs.