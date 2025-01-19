list:
    @just -l

compress algorithm world_size *args:
    @mpirun -np {{world_size}} cargo r --release -q -- -a {{algorithm}} c {{args}}

decompress algorithm world_size:
    @mpirun -np {{world_size}} cargo r --release -q -- -a {{algorithm}} d

compress-stats algorithm world_size *args:
    @mpirun -np {{world_size}} cargo r --release -q -- -p -a {{algorithm}} c {{args}}

decompress-stats algorithm world_size:
    @mpirun -np {{world_size}} cargo r --release -q -- -p -a {{algorithm}} d

example:
    cargo b
    cp -v target/debug/parallel-compression ~/main
    scp target/debug/parallel-compression 192.168.0.102:~/main
    mpirun -v -np 2 --hostfile hosts ~/main c < justfile
