Determines the crossover strategy used to transform the interior solution produced by barrier into a basic solution
(note that crossover is not available for QP or QCP models). `Crossover` consists of three phases: (i) a primal push
phase, where primal variables are pushed to bounds, (ii) a dual push phase, where dual variables are pushed to bounds,
and (iii) a cleanup phase, where simplex is used to remove any primal or dual infeasibilities that remain after the push
phases are complete. The order of the first two phases and the algorithm used for the third phase are both controlled by
the `Crossover` parameter:

Parameter value First push Second push Cleanup

0 Disabled Disabled Disabled

1 Dual Primal Primal

2 Dual Primal Dual

3 Primal Dual Primal

4 Primal Dual Dual

The default value of -1 chooses the strategy automatically. Use value 0 to disable crossover; this setting returns the
interior solution computed by barrier.

Note: Barrier only