The `NumericFocus` parameter controls the degree to which the code attempts to detect and manage numerical issues. The
default setting (0) makes an automatic choice, with a slight preference for speed. Settings 1-3 increasingly shift the
focus towards being more careful in numerical computations. With higher values, the code will spend more time checking
the numerical accuracy of intermediate results, and it will employ more expensive techniques in order to avoid potential
numerical issues.