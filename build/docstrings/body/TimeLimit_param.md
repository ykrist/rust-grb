Limits the total time expended (in seconds). Optimization returns with a TIME_LIMIT status if the limit is exceeded (see
the Status Code section for further details).

Note that optimization may not stop immediately upon hitting the time limit. It will stop after performing the required
additional computations of the attributes associated with the terminated optimization. As a result, the Runtime
attribute may be larger than the specified `TimeLimit` upon completion, and repeating the optimization with a
`TimeLimit` set to the Runtime attribute of the stopped optimization may result in additional computations and a larger
attribute value.