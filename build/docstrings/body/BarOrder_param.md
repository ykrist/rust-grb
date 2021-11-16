Chooses the barrier sparse matrix fill-reducing algorithm. A value of 0 chooses Approximate Minimum Degree ordering,
while a value of 1 chooses Nested Dissection ordering. The default value of -1 chooses automatically. You should only
modify this parameter if you notice that the barrier ordering phase is consuming a significant fraction of the overall
barrier runtime.

Note: Barrier only