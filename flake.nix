{
  description = "deadlocked dev shell for nix users";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default-linux";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    systems,
    ...
  }: let
    eachSystem = nixpkgs.lib.genAttrs (import systems);

    pkgsFor = system:
      import nixpkgs {
        inherit system;
        overlays = [];
      };
  in {
    packages = eachSystem (system: {
      source2viewer = nixpkgs.legacyPackages.${system}.callPackage ./nix/source2viewer.nix {};
    });

    devShells = eachSystem (system: {
      default = (pkgsFor system).callPackage ./nix/shell.nix {};
    });

    formatter.x86_64-linux = inputs.nixpkgs.legacyPackages.x86_64-linux.alejandra;
  };
}
