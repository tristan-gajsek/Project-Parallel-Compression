list:
    @just -l

compress algorithm world_size *args:
    @mpirun -np {{world_size}} cargo r -q -- -a {{algorithm}} c {{args}}

decompress algorithm world_size:
    @mpirun -np {{world_size}} cargo r -q -- -a {{algorithm}} d

example:
    cargo b
    cp -v target/debug/parallel-compression ~/main
    scp target/debug/parallel-compression 192.168.0.102:~/main
    mpirun -v -np 2 --hostfile hosts ~/main c < justfile
