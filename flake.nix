{
  # NOTE: This `flake.nix` is just an entrypoint into `package.nix`
  #       Where possible, all metadata should be defined in `package.nix` for non-flake consumers
  description = "webhook that does nothing with the information";

  inputs = {
    crane.url = "github:ipetkov/crane";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-24.11";
    flake-compat.url = "github:nix-community/flake-compat";
  };

  outputs = {
    # common
    self,
    nixpkgs,
    flake-compat,
    # rust
    crane,
    advisory-db,
  }: let
    target_systems = [
      "x86_64-linux"
      "aarch64-darwin"
    ];
    flake-utils = import ./nix/flake-utils.nix;
    arguments.for_package = {
      inherit
        advisory-db
        crane
        ;
      inherit (flake-utils.lib) mkApp;
    };
    nixos = import ./nix/nixos.nix {
      overlay = self.overlays.default;
    };

    systemd = import ./nix/systemd.nix;
  in
    flake-utils.lib.eachSystem target_systems (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };

        package = pkgs.callPackage ./nix/package.nix arguments.for_package;

        alejandra = pkgs.callPackage ./nix/alejandra.nix {};

        systemd-render-check = systemd.render_check {
          inherit
            nixpkgs
            pkgs
            ;
          null-webhook = package.${package.crate-name};
          inherit (nixos) nixosModules;
        };
      in {
        inherit (package) apps;

        checks =
          package.checks
          // alejandra.checks;

        packages = let
          inherit (package) crate-name;

          vm-tests = pkgs.callPackage ./nix/vm-tests {
            inherit (nixos) nixosModules;
          };

          all-long-tests = pkgs.symlinkJoin {
            name = "all-long-tests";
            paths = [
              vm-tests
              package.tests-ignored
              systemd-render-check
            ];
          };
        in {
          ${crate-name} = package.${crate-name};
          default = package.${crate-name};

          inherit
            all-long-tests
            vm-tests
            systemd-render-check
            ;

          # alias for convenience
          ci = all-long-tests;
        };

        devShells = {
          default = package.devShellFn {
            packages = [
              pkgs.alejandra
              pkgs.bacon
              pkgs.cargo-expand
              pkgs.cargo-outdated
              pkgs.cargo-insta
            ];
          };
        };
      }
    )
    // {
      overlays.default = final: prev: let
        package = final.callPackage ./nix/package.nix arguments.for_package;
      in {
        # NOTE: infinite recursion when using `${crate-name} = ...` syntax
        inherit (package) null-webhook;
      };

      inherit (nixos) nixosModules;

      lib = {
        systemd = {
          inherit (systemd) service render_service;
        };
      };
    };
}
