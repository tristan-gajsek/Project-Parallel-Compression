{
  outputs =
    { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      devShells.${system}.default =
        with pkgs;
        mkShell rec {
          shellHook = "exec zsh";
          buildInputs = [
            llvmPackages.clang
            llvmPackages.libclang
            mpi
            mpi.dev
            pkg-config
          ];
          nativeBuildInputs = buildInputs;
          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
        };
    };
}
