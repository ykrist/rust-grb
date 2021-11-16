Priorities on user hints. After providing variable hints through the `VarHintVal` attribute, you can optionally also
provide hint priorities to give an indication of your level of confidence in your hints.

Hint priorities are relative. If you are more confident in the hint value for one variable than for another, you simply
need to set a larger priority value for the more solid hint. The default hint priority for a variable is 0.

Please refer to the `VarHintVal` discussion for more details on the role of variable hints.

Only affects MIP models.