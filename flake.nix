{
  description = "WXOrca - AI-powered guide for IBM WatsonX Orchestrate";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # Build the Rust backend (agents + CLI)
        wxorca-backend = pkgs.rustPlatform.buildRustPackage {
          pname = "wxorca-agents";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
          ] ++ lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          # Build the CLI binary (release mode is default)
          cargoBuildFlags = [ "-p" "wxorca-agents" ];
        };

        # Build the frontend with bun
        wxorca-frontend = pkgs.stdenv.mkDerivation {
          pname = "wxorca-frontend";
          version = "0.1.0";
          src = ./frontend;

          nativeBuildInputs = [ pkgs.bun ];

          buildPhase = ''
            export HOME=$TMPDIR
            bun install --frozen-lockfile
            bun run build
          '';

          installPhase = ''
            mkdir -p $out
            cp -r dist/* $out/
          '';
        };

        # Backend OCI image
        backendImage = pkgs.dockerTools.buildLayeredImage {
          name = "wxorca-backend";
          tag = "latest";

          contents = with pkgs; [
            cacert
            busybox
            bun
          ];

          extraCommands = ''
            mkdir -p app/backend app/target/release
          '';

          config = {
            Env = [
              "PORT=3000"
              "NODE_ENV=production"
              "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
            ];
            ExposedPorts = {
              "3000/tcp" = {};
            };
            WorkingDir = "/app/backend";
            Cmd = [ "${pkgs.bun}/bin/bun" "run" "src/index.ts" ];
          };

          fakeRootCommands = ''
            cp -r ${./backend}/* app/backend/
            cp ${wxorca-backend}/bin/wxorca-cli app/target/release/ || true
          '';
        };

        # Frontend OCI image (static files served by a simple server)
        frontendImage = pkgs.dockerTools.buildLayeredImage {
          name = "wxorca-frontend";
          tag = "latest";

          contents = with pkgs; [
            cacert
            busybox
            static-web-server
          ];

          extraCommands = ''
            mkdir -p app/public
          '';

          config = {
            Env = [
              "SERVER_HOST=0.0.0.0"
              "SERVER_PORT=8080"
              "SERVER_ROOT=/app/public"
            ];
            ExposedPorts = {
              "8080/tcp" = {};
            };
            WorkingDir = "/app";
            Cmd = [ "${pkgs.static-web-server}/bin/static-web-server" ];
          };

          fakeRootCommands = ''
            cp -r ${wxorca-frontend}/* app/public/
          '';
        };

      in {
        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            openssl
            bun
            nodejs_20
            surrealdb

            # Dev tools
            cargo-watch
            cargo-edit
          ];

          shellHook = ''
            echo "🐳 WXOrca development environment"
            echo "   Rust: $(rustc --version)"
            echo "   Bun: $(bun --version)"
            echo "   SurrealDB: $(surreal version 2>/dev/null || echo 'not in PATH')"
          '';
        };

        # Packages
        packages = {
          default = wxorca-backend;
          backend = wxorca-backend;
          frontend = wxorca-frontend;

          # OCI images
          backend-image = backendImage;
          frontend-image = frontendImage;
        };

        # Apps for `nix run`
        apps = {
          default = {
            type = "app";
            program = "${wxorca-backend}/bin/wxorca-cli";
          };
        };
      }
    );
}
