# BASICEST BASIC FIT APP
Goals = Take lines of exercises and output Visual poster-like workout plans. Essentially only write an exercise plan, get pictures including set/rep + optional percentages and optional rest times between sets. Aesthetics and Instant Interpretation are key.
NonGoals -> Progress Tracking, Complete coverage,


## Architecture
DSL for exercises
```
line        := ws? name ws? ":" ws? scheme ws? rest? ws?
scheme      := sets_reps | rep_percent_list
sets_reps   := integer ws? "x" ws? integer
rep_percent_list := rep_percent (ws+ rep_percent)*
rep_percent := integer ws? "@" ws? number ws? "%"
rest        := "#" ws? integer ("s" | "m")
```
Basicall
