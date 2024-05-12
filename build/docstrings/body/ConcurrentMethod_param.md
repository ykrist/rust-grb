This parameter is only evaluated when solving an LP with a concurrent solver (Method = 3 or 4). It controls which
methods are run concurrently by the concurrent solver. Options are:

-1=automatic,

0=barrier, dual, primal simplex,

1=barrier and dual simplex,

2=barrier and primal simplex, and

3=dual and primal simplex.

Which methods are actually run also depends on the number of threads available.