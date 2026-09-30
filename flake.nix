{
  description = "naruwm — development environment and nested Wayland compositor";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      perSystem = system:
        let
          pkgs = import nixpkgs { inherit system; };
          libraries = with pkgs; [
            wayland libxkbcommon libGL libglvnd
            libx11 libxcursor libxi libxrandr
          ];
          runtimePath = pkgs.lib.makeLibraryPath libraries;
        in {
          packages.default = pkgs.rustPlatform.buildRustPackage {
            pname = "naruwm";
            version = "0.1.0";
            src = pkgs.lib.fileset.toSource {
              root = ./.;
              fileset = pkgs.lib.fileset.unions [
                ./Cargo.toml ./Cargo.lock ./src ./LICENSE ./LICENSES ./THIRD_PARTY.md
              ];
            };
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [ pkgs.pkg-config pkgs.makeWrapper ];
            buildInputs = libraries;
            postFixup = ''
              wrapProgram $out/bin/naruwm --prefix LD_LIBRARY_PATH : "${runtimePath}"
            '';
          };
          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              cargo rustc rustfmt clippy rust-analyzer pkg-config
              weston wayland-utils python3
            ];
            buildInputs = libraries;
            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
            shellHook = ''
              export LD_LIBRARY_PATH="${runtimePath}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
            '';
          };
        };
    in {
      packages = forAllSystems (system: (perSystem system).packages);
      devShells = forAllSystems (system: (perSystem system).devShells);
      checks = forAllSystems (system: { build = self.packages.${system}.default; });
    };
}
