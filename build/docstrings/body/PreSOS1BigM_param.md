Controls the automatic reformulation of SOS1 constraints into binary form. SOS1 constraints are often handled more
efficiently using a binary representation. The reformulation often requires big-M values to be introduced as
coefficients. This parameter specifies the largest big-M that can be introduced by presolve when performing this
reformulation. Larger values increase the chances that an SOS1 constraint will be reformulated, but very large values
(e.g., 1e8) can lead to numerical issues.

The default value of -1 chooses a threshold automatically. You should set the parameter to 0 to shut off SOS1
reformulation entirely, or a large value to force reformulation.