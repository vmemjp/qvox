{
  description = "qvox — Rust GUI client for Qwen3-TTS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, fenix, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
          config.cudaSupport = true;
        };

        fenixPkgs = fenix.packages.${system};
        rustToolchain = fenixPkgs.combine [
          fenixPkgs.stable.rustc
          fenixPkgs.stable.cargo
          fenixPkgs.stable.clippy
          fenixPkgs.stable.rustfmt
          fenixPkgs.stable.rust-src
        ];

        # System libraries needed at build and runtime
        nativeBuildInputs = with pkgs; [
          pkg-config
          cmake          # whisper.cpp (whisper-rs)
          rustToolchain
        ];

        buildInputs = with pkgs; [
          # TLS (reqwest)
          openssl

          # Audio (cpal / rodio)
          alsa-lib

          # GUI (iced) — Wayland + X11 + Vulkan
          wayland
          libxkbcommon
          libx11
          libxcursor
          libxi
          libxrandr
          vulkan-loader
          vulkan-headers
          libGL

          # CUDA (whisper-rs GPU)
          cudaPackages.cudatoolkit

          # Whisper.cpp build
          gcc
        ];

        # LD_LIBRARY_PATH for runtime linking
        libPath = pkgs.lib.makeLibraryPath buildInputs;
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "qvox";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          inherit nativeBuildInputs buildInputs;

          # whisper-rs needs to find CUDA
          CUDA_PATH = "${pkgs.cudaPackages.cudatoolkit}";

          postFixup = ''
            patchelf --set-rpath "${libPath}" $out/bin/qvox
          '';
        };

        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;

          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          CUDA_PATH = "${pkgs.cudaPackages.cudatoolkit}";

          LD_LIBRARY_PATH = libPath;

          shellHook = ''
            echo "qvox dev shell"
            echo "  rust   : $(rustc --version)"
            echo "  cargo  : $(cargo --version)"
            echo "  CUDA   : $CUDA_PATH"
          '';

          packages = with pkgs; [
            # Python (backend server)
            python3
            python3Packages.pip

            # Dev tools
            rust-analyzer
            cargo-watch
            cargo-nextest
            cargo-llvm-cov
          ];
        };
      });
}
