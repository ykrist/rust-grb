Controls the number of threads to apply to parallel algorithms (concurrent LP, parallel barrier, parallel MIP, etc.).
The default value of 0 is an automatic setting. It will generally use as many threads as there are virtual processors.
The number of virtual processors may exceed the number of cores due to hyperthreading or other similar hardware
features.

While you will generally get the best performance by using all available cores in your machine, there are a few
exceptions. One is of course when you are sharing a machine with other jobs. In this case, you should select a thread
count that doesn't oversubscribe the machine.

We have also found that certain classes of MIP models benefit from reducing the thread count, often all the way down to
one thread. Starting multiple threads introduces contention for machine resources. For classes of models where the first
solution found by the MIP solver is almost always optimal, and that solution isn't found at the root, it is often better
to allow a single thread to explore the search tree uncontested.

Another situation where reducing the thread count can be helpful is when memory is tight. Each thread can consume a
significant amount of memory.

We've made the pragmatic choice to impose a soft limit of 32 threads for the automatic setting (0). If your machine has
more, and you find that using more increases performance, you should feel free to set the parameter to a larger value.