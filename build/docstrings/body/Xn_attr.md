The variable value in a sub-optimal MIP solution. Use parameter SolutionNumber to indicate which alternate solution to
retrieve. Solutions are sorted in order of worsening objective value. Thus, when SolutionNumber is 1, `Xn` returns the
second-best solution found. When SolutionNumber is equal to its default value of 0, querying attribute `Xn` is
equivalent to querying attribute X.

The number of sub-optimal solutions found during the MIP search will depend on the values of a few parameters. The most
important of these are PoolSolutions, PoolSearchMode, and PoolGap. Please consult the section on Solution Pools for a
more detailed discussion of this topic.