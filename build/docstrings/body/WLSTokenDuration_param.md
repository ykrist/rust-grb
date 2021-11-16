When using a WLS license, this parameter can be used to adjust the lifespan (in minutes) of a token. A token enables
Gurobi to run on that client for the life of the token. Gurobi will automatically request a new token if the current one
expires, but it won't notify the WLS server if it completes its work before the current token expires. A shorter
lifespan is better for short-lived usage. A longer lifespan is better for environments where the network connection to
the WLS server is unreliable.