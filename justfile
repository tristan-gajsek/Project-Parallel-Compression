run WORLD-SIZE *ARGS:
    @mpirun -np {{WORLD-SIZE}} cargo r -q -- {{ARGS}}
