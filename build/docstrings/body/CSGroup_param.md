Specifies one or more groups of cluster nodes to control the placement of the job. The list is a comma-separated string
of group names, with optionally a priority for a group. For example, specifying group1:10,group2:50 means that the job
will run on machines of group1 or group2, and if the job is queued, it will have priority 10 on group1 and 50 on group2.
Note that if the group is not specified, the job may run on any node. If there are no nodes in the cluster having the
specified groups, the job will be rejected.

Refer to the Gurobi Remote Services Reference Manual for more information on starting Compute Server jobs and in
particular to Gurobi Remote Services cluster Grouping for more information on grouping cluster nodes.

You must set this parameter through either a license file (using GROUP=name) or an empty environment. Changing the
parameter after your environment has been created will have no effect.