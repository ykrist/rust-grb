When using a token license, set this parameter to the name of the token server. You can refer to the server using its
name or its IP address.

You can provide a comma-separated list of token servers to increase robustness. If the first server in the list doesn't
respond, the second will be tried, etc.

You must set this parameter through either a gurobi.lic file (using TOKENSERVER=server) or an empty environment.
Changing the parameter after your environment has been created will have no effect.