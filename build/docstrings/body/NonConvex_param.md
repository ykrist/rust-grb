Sets the strategy for handling non-convex quadratic objectives or non-convex quadratic constraints. With setting 0, an
error is reported if the original user model contains non-convex quadratic constructs. With setting 1, an error is
reported if non-convex quadratic constructs could not be discarded or linearized during presolve. With setting 2, non-
convex quadratic problems are solved by means of translating them into bilinear form and applying spatial branching. The
default -1 setting is currently equivalent to 1, and may change in future releases to be equivalent to 2.

Note: Only affects QP, QCP, MIQP, and MIQCP models