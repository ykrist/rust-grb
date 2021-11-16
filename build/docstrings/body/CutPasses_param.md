A non-negative value indicates the maximum number of cutting plane passes performed during root cut generation. The
default value chooses the number of cut passes automatically.

You should experiment with different values of this parameter if you notice the MIP solver spending significant time on
root cut passes that have little impact on the objective bound.

Note: Only affects mixed integer programming (MIP) models