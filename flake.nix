{
  description = "TLI Tools — Data-oriented TigerStyle Rust workspace";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    rust-overlay,
    advisory-db,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };

      inherit (pkgs) lib;

      # Stable Rust toolchain supporting edition = "2024", slice::as_chunks,
      # and strict TigerStyle clippy/rustfmt verification without nightly churn.
      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = [
          "rust-src"
          "rust-analyzer"
          "clippy"
          "rustfmt"
        ];
      };

      craneLib = (crane.mkLib pkgs).overrideToolchain (_: rustToolchain);

      baseFileset = lib.fileset.unions [
        ./Cargo.toml
        ./Cargo.lock
        ./clippy.toml
        ./rustfmt.toml
        (lib.fileset.maybeMissing ./.cargo)
        (lib.fileset.maybeMissing ./.config)
        (craneLib.fileset.commonCargoSources ./crates)
      ];

      # Filtered workspace source tree (excludes README, flake files, git metadata, etc.)
      src = lib.fileset.toSource {
        root = ./.;
        fileset = baseFileset;
      };

      commonEnv = lib.optionalAttrs (pkgs.stdenv.hostPlatform.isLinux && pkgs.stdenv.hostPlatform.isx86_64) {
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "${pkgs.clang}/bin/clang";
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-C target-cpu=x86-64-v3 -C link-arg=-fuse-ld=${pkgs.mold}/bin/mold";
      };

      commonArgs =
        commonEnv
        // {
          inherit src;
          strictDeps = true;

          nativeBuildInputs =
            [
              pkgs.clang
            ]
            ++ lib.optionals pkgs.stdenv.hostPlatform.isLinux [
              pkgs.mold
            ];
        };

      # Build *only* the cargo dependencies for the entire workspace once,
      # unified via cargo-hakari (crates/workspace-hack) so all downstream crates
      # and checks share the exact same pre-compiled dependency artifacts.
      cargoArtifacts = craneLib.buildDepsOnly (commonArgs
        // {
          cargoExtraArgs = "--workspace";
        });

      individualCrateArgs =
        commonArgs
        // {
          inherit cargoArtifacts;
          # Tests are executed once workspace-wide via cargo-nextest in `checks`
          doCheck = false;
        };

      # Data-oriented domain library crate
      data = craneLib.buildPackage (individualCrateArgs
        // {
          pname = "data";
          cargoExtraArgs = "-p data";
          meta = {
            description = "Data-oriented domain crate for TLI Tools";
          };
        });

      # Data-oriented TigerStyle Unreal Engine .pak archive reader crate
      pak = craneLib.buildPackage (individualCrateArgs
        // {
          pname = "pak";
          cargoExtraArgs = "-p pak";
          meta = {
            description = "Data-oriented TigerStyle Unreal Engine .pak archive reader";
          };
        });

      # Helper script to regenerate and verify the cargo-hakari workspace-hack crate
      hakariRegenApp = pkgs.writeShellApplication {
        name = "tli-hakari-regen";
        runtimeInputs = [
          rustToolchain
          pkgs.cargo-hakari
        ];
        text = ''
          echo "==> Generating workspace-hack via cargo-hakari..."
          cargo hakari generate
          echo "==> Ensuring all workspace crates depend on workspace-hack..."
          cargo hakari manage-deps
          echo "==> Verifying workspace-hack feature unification..."
          cargo hakari verify
          echo "==> Updating Cargo.lock..."
          cargo update --workspace --offline || cargo generate-lockfile
          echo "Done!"
        '';
      };
    in {
      checks = {
        # Build all workspace packages as part of `nix flake check`
        inherit data pak;

        # Verify `workspace-hack` is up-to-date and properly wired across all crates
        tli-hakari = craneLib.mkCargoDerivation {
          inherit src;
          pname = "tli-hakari";
          cargoArtifacts = null;
          doInstallCargoArtifacts = false;

          buildPhaseCargoCommand = ''
            cargo hakari generate --diff
            cargo hakari manage-deps --dry-run
            cargo hakari verify
          '';

          nativeBuildInputs = [
            pkgs.cargo-hakari
          ];
        };

        # Run clippy across all workspace crates & targets, reusing cached artifacts
        tli-clippy = craneLib.cargoClippy (commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--workspace --all-targets -- --deny warnings";
          });

        # Verify Rustdoc builds cleanly without warnings
        tli-doc = craneLib.cargoDoc (commonArgs
          // {
            inherit cargoArtifacts;
            cargoDocExtraArgs = "--workspace --no-deps";
          });

        # Check Rust formatting with rustfmt
        tli-fmt = craneLib.cargoFmt {
          inherit src;
        };

        # Check TOML formatting with taplo
        tli-toml-fmt = craneLib.taploFmt {
          src = lib.sources.sourceFilesBySuffices src [".toml"];
        };

        # Audit Cargo.lock against RustSec advisory-db
        tli-audit = craneLib.cargoAudit {
          inherit src advisory-db;
        };

        # Run the full test suite with cargo-nextest
        tli-nextest = craneLib.cargoNextest (commonArgs
          // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
            cargoNextestPartitionsExtraArgs = "--no-tests=pass";
            cargoNextestExtraArgs = "--workspace";
          });
      };

      packages = {
        default = data;
        inherit data pak;
        deps = cargoArtifacts;
      };

      apps = {
        hakari =
          (flake-utils.lib.mkApp {
            drv = hakariRegenApp;
          })
          // {
            meta.description = "Regenerate and verify the cargo-hakari workspace-hack crate";
          };
      };

      formatter = pkgs.alejandra;

      devShells.default = craneLib.devShell (commonEnv
        // {
          # Automatically inherits inputsFrom all checks (cargo-hakari, cargo-nextest,
          # cargo-audit, taplo, clang, mold, and rustToolchain)
          checks = self.checks.${system};

          packages = with pkgs; [
            # Extra interactive dev tools not already pulled in via `checks`
            cargo-edit
            cargo-outdated
            cargo-watch
            cargo-bloat
            cargo-flamegraph
            cargo-deny
            alejandra
            nixd
            ripgrep
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        });
    });
}
