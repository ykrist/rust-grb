Tag for variables.

If you will be retrieving the solution to your model in JSON format, you might define a tag for every variable that you
plan to retrieve solution information for. Each variable tag must be unique, and if any tag is used (variable tag,
constraint tag, quadratic constraint tag) only tagged elements will appear in the JSON solution string. Tags must
consist of printable US-ASCII characters. Using extended characters or escaped characters will result in an error. The
maximum supported length for a tag is 10240 characters.