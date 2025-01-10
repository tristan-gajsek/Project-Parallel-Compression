# Parallel Compression

This program is a part of our college project, in which we had various amounts of JSON data that we wanted to compress. To do this, we made use of MPI for parallel process execution and our own implementation of a difference-based compression algorithm, and Huffman compression.

# Dependencies

If you'd like to run the program, first make sure you have the following dependencies installed on your system:

- a Rust compiler
- clang
- mpi
- pkg-config
- just (if you plan on using the justfile)

Alternatively, use the flake.nix file provided in the repository by running `nix develop`. This will put you into a shell with the required dependencies (excluding a Rust compiler), assuming you're using Nix.

# Usage

The program will take input through stdin and print the processed input to stdout. To see all available arguments, run `cargo r -- -h`. Since invoking the program can be verbose, we've provided a justfile for convenience. To use it, make sure you have the just code runner installed.

If you'd like to compress your data, use the `compress` recipe. The following command will compress the contents of `uncompressed-data` and output it's decompressed counterpart to stdout. The parameters to the recipe are, in order:

- The algorithm to be used, which can be either `huffman` or `delta`
- The amount of processes to spawn, which must be at least 2
- The amount of bytes of data to give to each process

```bash
just compress huffman 10 -s 100 < uncompressed-data
```

Similarly, you can use the `decompress` recipe to decompress the data you've compressed. The parameters are identical to the compress recipe, excluding the parameter for determining the amount of data each process must decompress, as that has to be identical to what was used with compression. Make sure the input is proper compressed data, or else the decompression will fail.

```bash
just decompress huffman 10 < compressed-data
```

Lastly, if you'd like information about the data that was compressed/decompressed, you can replace the `compress`/`decompress` recipes with the `compress-stats`/`decompress-stats` recipes. This will show you the amount of time it took to compress/decompress your data, along the the input and output size of the data provided.
