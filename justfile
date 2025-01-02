run WORLD_SIZE *ARGS:
    @mpirun -np {{WORLD_SIZE}} cargo r -q -- {{ARGS}}
