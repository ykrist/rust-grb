Current relative MIP optimality gap; computed as

$\vert ObjBound-ObjVal\vert/\vert ObjVal\vert$ (where `ObjBound` and `ObjVal` are the MIP objective bound and incumbent
solution objective, respectively. Returns GRB_INFINITY when an incumbent solution has not yet been found, when no
objective bound is available, or when the current incumbent objective is 0.