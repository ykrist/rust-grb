Determines how large a (relative) gap to tolerate in stored solutions. When this parameter is set to a non-default
value, solutions whose objective values exceed that of the best known solution by more than the specified (relative) gap
are discarded. For example, if the MIP solver has found a solution at objective 100, then a setting of PoolGap=0.2 would
discard solutions with objective worse than 120 (assuming a minimization objective).

Note: Only affects mixed integer programming (MIP) models