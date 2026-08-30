Every time-step in a cellular automata can be thought of as a function T(S,R) -> S, where S is the type of States and R is the type of Rules. 
This program seeks to explore the consequences when S = R, and what happens when using a state as it's own rule to advance.

We define a "cellular automata" as such:
- a set of nodes N labeled 0..|N|
- a set of "states" S.
- a set of functions A where each function a is of the form f(N) -> N (these are the "neighbor functions" that denote how nodes are connected together. Note that a cell can be its own neighbor)
- a function R(S^A) -> S (this is the "rule" that determines how the state of a cell's "neighborhood" influences its state on the following turn)
- a function I(N) -> S (this denotes the "initial state" of the cellular automata).

In a cellular automata, the state of any node n at timestep t can be found by the function G_t(n), defined as follows:
G_0(n) = I(n)
G_t(n) = R(G_{t-1}(a(n)) for all a in A)
This of course means that the state of a cell depends on the state of its neighbors at the last timestep

An autocellular system has a bijection Q between the set of possible neighborhoods S^A and the set of nodes N, and defines its rule as follows:
R_t(u) = G_t(Q(u))
Thus, the rule becomes dependent on t, changing with time.

This repository exists to create, manipulate, and study autocellular systems.
