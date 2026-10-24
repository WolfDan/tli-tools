{
  description = "Tartarus Architect Overlay — Native Rust GPUI desktop overlay, Monte Carlo solver, CV scanner & UE4 Pak extractor";

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

      # Nightly toolchain required for edition = "2024", rust-version = "1.95",
      # and `#![feature(portable_simd)]` in tartarus-scan / tartarus-overlay.
      rustToolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain:
        toolchain.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
            "rustfmt"
          ];
        });

      craneLib = (crane.mkLib pkgs).overrideToolchain (_: rustToolchain);

      baseFileset = lib.fileset.unions [
        ./Cargo.toml
        ./Cargo.lock
        (lib.fileset.maybeMissing ./.cargo)
        (lib.fileset.maybeMissing ./.config)
        (craneLib.fileset.commonCargoSources ./crates)
      ];

      # Filtered workspace source tree (excludes README, assets, flake files, tests, etc.)
      src = lib.fileset.toSource {
        root = ./.;
        fileset = baseFileset;
      };

      # Source tree including `./tests` fixtures for nextest runs
      testSrc = lib.fileset.toSource {
        root = ./.;
        fileset = lib.fileset.unions [
          baseFileset
          (lib.fileset.maybeMissing ./tests)
        ];
      };

      # Shared native & dlopen'd runtime libraries required by gpui-ce, wgpu, wayland, x11rb, and ksni
      runtimeLibs = with pkgs; [
        vulkan-loader
        libGL
        wayland
        libxkbcommon
        fontconfig
        freetype
        dbus
        libx11
        libxcursor
        libxi
        libxrandr
        libxext
        libxfixes
        libxtst
        libxcb
        libxrender
      ];

      commonEnv =
        {
          # Required by bindgen (used by gpui-ce and gpui_ce_media)
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        }
        // lib.optionalAttrs (pkgs.stdenv.hostPlatform.isLinux && pkgs.stdenv.hostPlatform.isx86_64) {
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "${pkgs.clang}/bin/clang";
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-C target-feature=+ssse3,+sse4.2,+avx2,+fma,+aes -C link-arg=-fuse-ld=${pkgs.mold}/bin/mold";
        };

      commonArgs =
        commonEnv
        // {
          inherit src;
          strictDeps = true;

          nativeBuildInputs =
            [
              pkgs.pkg-config
              pkgs.clang
            ]
            ++ lib.optionals pkgs.stdenv.hostPlatform.isLinux [
              pkgs.mold
            ];

          buildInputs =
            runtimeLibs
            ++ [
              pkgs.wayland-protocols
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

      # 1. Main GPUI Desktop Overlay
      tartarus-overlay = craneLib.buildPackage (individualCrateArgs
        // {
          pname = "tartarus-overlay";
          cargoExtraArgs = "-p tartarus-overlay";
          nativeBuildInputs = commonArgs.nativeBuildInputs ++ [pkgs.makeWrapper];
          postInstall = ''
            wrapProgram $out/bin/tartarus-overlay \
              --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath runtimeLibs}"
          '';
          meta = {
            description = "Native GPUI desktop overlay and Monte Carlo solver for Torchlight: Infinite Tartarus (S16)";
            mainProgram = "tartarus-overlay";
          };
        });

      # 2. UE 4.27 Pak v12 & Oodle Asset Extractor CLI
      tartarus-extract = craneLib.buildPackage (individualCrateArgs
        // {
          pname = "tartarus-extract";
          cargoExtraArgs = "-p tartarus-extract";
          meta = {
            description = "Programmatic UE 4.27 Pak v12 & Oodle asset/texture extractor for Torchlight: Infinite";
            mainProgram = "tartarus-extract";
          };
        });

      # 3. S16 Game Database, Visual Asset Catalog & CV Table Generator CLI
      tartarus-data = craneLib.buildPackage (individualCrateArgs
        // {
          pname = "tartarus-data";
          cargoExtraArgs = "-p tartarus-data";
          meta = {
            description = "Authoritative S16 Tartarus game database, visual asset catalog renderer, and Rust CV table generator";
            mainProgram = "tartarus-data";
          };
        });

      # Helper script to regenerate and verify the cargo-hakari workspace-hack crate
      hakariRegenApp = pkgs.writeShellApplication {
        name = "tartarus-hakari-regen";
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
        # Build all workspace binary packages as part of `nix flake check`
        inherit tartarus-overlay tartarus-extract tartarus-data;

        # Verify `workspace-hack` is up-to-date and properly wired across all crates
        tartarus-hakari = craneLib.mkCargoDerivation {
          inherit src;
          pname = "tartarus-hakari";
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
        tartarus-clippy = craneLib.cargoClippy (commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--workspace --all-targets -- --deny warnings";
          });

        # Verify Rustdoc builds cleanly without warnings
        tartarus-doc = craneLib.cargoDoc (commonArgs
          // {
            inherit cargoArtifacts;
            cargoDocExtraArgs = "--workspace --no-deps";
          });

        # Check Rust formatting with rustfmt
        tartarus-fmt = craneLib.cargoFmt {
          inherit src;
        };

        # Check TOML formatting with taplo
        tartarus-toml-fmt = craneLib.taploFmt {
          src = lib.sources.sourceFilesBySuffices src [".toml"];
        };

        # Audit Cargo.lock against RustSec advisory-db
        tartarus-audit = craneLib.cargoAudit {
          inherit src advisory-db;
        };

        # Run the full test suite in release mode with cargo-nextest
        tartarus-nextest = craneLib.cargoNextest (commonArgs
          // {
            src = testSrc;
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
            cargoNextestPartitionsExtraArgs = "--no-tests=pass";
            cargoNextestExtraArgs = "--workspace";
          });
      };

      packages = {
        default = tartarus-overlay;
        inherit tartarus-overlay tartarus-extract tartarus-data;
        deps = cargoArtifacts;
      };

      apps = {
        default = flake-utils.lib.mkApp {
          drv = tartarus-overlay;
        };
        extract = flake-utils.lib.mkApp {
          drv = tartarus-extract;
        };
        data = flake-utils.lib.mkApp {
          drv = tartarus-data;
        };
        hakari = flake-utils.lib.mkApp {
          drv = hakariRegenApp;
        };
      };

      formatter = pkgs.alejandra;

      devShells.default = craneLib.devShell (commonEnv
        // {
          # Automatically inherits inputsFrom all checks (cargo-hakari, cargo-nextest,
          # cargo-audit, taplo, pkg-config, clang, mold, runtimeLibs, and rustToolchain)
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
            vulkan-tools
            wayland-utils
          ];

          LD_LIBRARY_PATH = lib.makeLibraryPath runtimeLibs;
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        });
    });
}
