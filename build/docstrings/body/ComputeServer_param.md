Set this parameter to the name of a node in the Remote Services cluster where you'd like your Compute Server job to run.
You can refer to the server using its name or its IP address. If you are using a non-default port, the server name
should be followed by the port number (e.g., server1:61000).

You will also need to set the `ServerPassword` parameter to supply the client password for the specified cluster.

You can provide a comma-separated list of nodes to increase robustness. If the first node in the list doesn't respond,
the second will be tried, etc.

Refer to the Gurobi Remote Services Reference Manual for more information on starting Compute Server jobs.

You must set this parameter through either a gurobi.lic file (using COMPUTESERVER=server) or an empty environment.
Changing the parameter after your environment has been created will have no effect.