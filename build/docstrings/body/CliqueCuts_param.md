Controls clique cut generation. Use 0 to disable these cuts, 1 for moderate cut generation, or 2 for aggressive cut
generation. The default -1 value choose automatically. Overrides the `Cuts` parameter.

We have observed that setting this parameter to its aggressive setting can produce a significant benefit for some large
set partitioning models.

Note: Only affects mixed integer programming (MIP) models