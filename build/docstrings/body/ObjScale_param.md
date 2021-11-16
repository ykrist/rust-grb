When positive, divides the model objective by the specified value to avoid numerical issues that may result from very
large or very small objective coefficients. The default value of 0 decides on the scaling automatically. A value less
than zero uses the maximum coefficient to the specified power as the scaling (so ObjScale=-0.5 would scale by the square
root of the largest objective coefficient).

Note that objective scaling can lead to large dual violations on the original, unscaled objective when the optimality
tolerance with the scaled objective is barely satisfied, so it should be used sparingly. Note also that scaling will be
more effective when all objective coefficients are of similar orders of magnitude, as opposed to objectives with a wide
range of coefficients. In the latter case, consider using the Multiple Objectives feature instead.