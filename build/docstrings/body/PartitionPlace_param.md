Setting the Partition attribute on at least one variable in a model enables the partitioning heuristic, which uses
large-neighborhood search to try to improve the current incumbent solution.

This parameter determines where that heuristic runs. Options are:

Before the root relaxation is solved (16)

At the start of the root cut loop (8)

At the end of the root cut loop (4)

At the nodes of the branch-and-cut search (2)

When the branch-and-cut search terminates (1)

The parameter value is a bit vector, where each bit turns the heuristic on or off at that place. The numerical values
next to the options listed above indicate which bit controls the corresponding option. Thus, for example, to enable the
heuristic at the beginning and end of the root cut loop (and nowhere else), you would set the 8 bit and the 4 bit to 1,
which would correspond to a parameter value of 12.

The default value of 15 indicates that we enable every option except the first one listed above.