Linear objective coefficient. In our object-oriented interfaces, you typically use the setObjective method to set the
objective, but this attribute provides an alternative for setting or modifying linear objective terms.

Note that this attribute interacts with our piecewise-linear objective feature. If you set a piecewise-linear objective
function for a variable, that will automatically set the `Obj` attribute to zero. Similarly, if you set the `Obj`
attribute for a variable, that will automatically delete any previously specified piecewise-linear objective.