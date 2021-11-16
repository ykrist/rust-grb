Limits the number of MIP nodes explored. Optimization returns with an NODE_LIMIT status if the limit is exceeded (see
the Status Code section for further details). Note that if multiple threads are used for the optimization, the actual
number of explored nodes may be slightly larger than the set limit.

Note: Only affects mixed integer programming (MIP) models