set export

list:
    @just -l

compress algorithm world_size *args:
    @mpirun -np {{world_size}} cargo r -q -- -a {{algorithm}} c {{args}}

decompress algorithm world_size:
    @mpirun -np {{world_size}} cargo r -q -- -a {{algorithm}} d
