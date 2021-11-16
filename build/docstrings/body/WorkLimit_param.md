Limits the total work expended (in work units). Optimization returns with a WORK_LIMIT status if the limit is exceeded
(see the Status Code section for further details).

In contrast to the TimeLimit, work limits are deterministic. This means that on the same hardware and with the same
parameter and attribute settings, a work limit will stop the optimization of a given model at the exact same point every
time. One work unit corresponds very roughly to one second on a single thread, but this greatly depends on the hardware
on which Gurobi is running and the model that is being solved.

Note that optimization may not stop immediately upon hitting the work limit. It will stop when the optimization is next
in a deterministic state, and it will then perform the required additional computations of the attributes associated
with the terminated optimization. As a result, the Work attribute may be larger than the specified `WorkLimit` upon
completion, and repeating the optimization with a `WorkLimit` set to the Work attribute of the stopped optimization may
result in additional computations and a larger attribute value.