Rayon **find_map_any()** demonstration program

This small program shows how effective and simple Rust's *Rayon* crate **find_map_any()** can
be used to utilize all CPU cores. 

However, it also shows that **find_map_any()** still has a bug in that it can get in a loop 
such that a thread closure correctly ending with a *not-None* option can at times *never end*.
This behavior has been observed to occur not always.  In certain cases it almost never occurs
(e.g for `test[1]`), in other cases sometimes (like for `test[2]`), and in even other cases
it happens every time (see `test[2]`).

Hopefully the Rayon support team can fix this bug.

P.S.: The `factorize_iterative()` function of mine was an experiment to try out *find_map_any()*
with something not immediately trivial, but is of course not meant for production use.  Better and 
fast algorithms exist, also in crates.io.