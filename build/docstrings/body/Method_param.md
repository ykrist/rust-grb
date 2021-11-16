Algorithm used to solve continuous models or the initial root relaxation of a MIP model. Options are: -1=automatic,
0=primal simplex, 1=dual simplex, 2=barrier, 3=concurrent, 4=deterministic concurrent, 5=deterministic concurrent
simplex.

In the current release, the default Automatic (-1) setting will typically choose non-deterministic concurrent (Method=3)
for an LP, barrier (Method=2) for a QP or QCP, and dual (Method=1) for the MIP root relaxation. Only the simplex and
barrier algorithms are available for continuous QP models. Only primal and dual simplex are available for solving the
root of an MIQP model. Only barrier is available for continuous QCP models.

Concurrent optimizers run multiple solvers on multiple threads simultaneously, and choose the one that finishes first.
Method=3 and Method=4 will run dual simplex, barrier, and sometimes primal simplex (depending on the number of available
threads). Method=5 will run both primal and dual simplex. The deterministic options (Method=4 and Method=5) give the
exact same result each time, while Method=3 is often faster but can produce different optimal bases when run multiple
times.

The default setting is rarely significantly slower than the best possible setting, so you generally won't see a big gain
from changing this parameter. There are classes of models where one particular algorithm is consistently fastest,
though, so you may want to experiment with different options when confronted with a particularly difficult model.

Note that if memory is tight on an LP model, you should consider using the dual simplex method (Method=1). The
concurrent optimizer, which is typically chosen when using the default setting, consumes a lot more memory than dual
simplex alone.