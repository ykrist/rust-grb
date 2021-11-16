The constraint dual value in the current solution (also known as the shadow price).

Given a linear programming problem

$\begin{array}{ll} \mathrm{minimize} & c'x \\ \mathrm{subject\ to} & Ax \ge b \\ & x \ge 0 \end{array}$

and a corresponding dual problem

$\begin{array}{ll} \mathrm{maximize} & b'y \\ \mathrm{subject\ to} & A'y \le c \\ & y \ge 0 \end{array}$

the `Pi` attribute returns $y$.

Of course, not all models fit this canonical form. In general, dual values have the following properties:

Dual values for $\ge$ constraints are $\ge 0$.

Dual values for $\le$ constraints are $\le 0$.

Dual values for $=$ constraints are unconstrained.

For models with a maximization sense, the senses of the dual values are reversed: the dual is $\ge 0$ for a $\le$
constraint and $\le 0$ for a $\ge$ constraint.

Note that constraint dual values for linear constraints of QCP models are only available when the QCPDual parameter is
set to 1. To query the duals of the quadratic constraints in a QCP model, see QCPi.

Only available for convex, continuous models.