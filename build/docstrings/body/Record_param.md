Enables API call recording. When enabled, Gurobi will write one or more files (named gurobi000.grbr or similar) that
capture the sequence of Gurobi commands that your program issued. This file can subsequently be replayed using the
Gurobi command-line tool. Replaying the file will repeat the exact same sequence of commands, and when completed will
show the time spent in Gurobi API routines, the time spent in Gurobi algorithms, and will indicate whether any Gurobi
environments or models were leaked by your program. Replay files are particularly useful in tech support situations.
They provide an easy way to relay to Gurobi tech support the exact sequence of Gurobi commands that led to a question or
issue.

This parameter must be set before starting an empty environment (or in a gurobi.env file). All Gurobi commands will be
recorded until the environment is freed or the program ends.