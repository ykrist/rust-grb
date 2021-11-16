Controls the automatic reformulation of SOS1 constraints. Such constraints can be handled directly by the MIP branch-
and-cut algorithm, but they are often handled more efficiently by reformulating them using binary or integer variables.
There are several diffent ways to perform this reformulation; they differ in their size and strength. Smaller
reformulations add fewer variables and constraints to the model. Stronger reformulations reduce the number of branch-
and-cut nodes required to solve the resulting model.

Options 0 and 1 of this parameter encode an SOS1 constraint using a formulation whose size is linear in the number of
SOS members. Option 0 uses a so-called multiple choice model. It usually produces an LP relaxation that is easier to
solve. Option 1 uses an incremental model. It often gives a stronger representation, reducing the amount of branching
required to solve harder problems.

Options 2 and 3 of this parameter encode the SOS1 using a formulation of logarithmic size. They both only apply when all
the variables in the SOS1 are non-negative. Option 3 additionally requires that the sum of the variables in the SOS1 is
equal to 1. Logarithmic formulations are often advantageous when the SOS1 constraint has a large number of members.
Option 2 focuses on a formulation whose LP relaxation is easier to solve, while option 3 has better branching behaviour.

The default value of -1 chooses a reformulation for each SOS1 constraint automatically.

Note that the reformulation of SOS1 constraints is also influenced by the `PreSOS1BigM` parameter. To shut off the
reformulation entirely you should set that parameter to 0.