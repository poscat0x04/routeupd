{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";
    flake-utils.url = "github:poscat0x04/flake-utils";
    nix-filter.url = "github:numtide/nix-filter";
  };

  outputs =
    { self
    , nixpkgs
    , flake-utils
    , nix-filter
    }:
  flake-utils.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { inherit system; overlays = [ self.overlay ]; };
    in {
      packages = rec {
        inherit (pkgs) routeupd;
        default = routeupd;
      };
    }) // {
      nixosModules.routeupd =
        { config, lib, pkgs, ... }:
        let
          cfg = config.services.routeupd;
          hasDep = !(builtins.isNull cfg.dependency);
          depUnit = lib.optional hasDep cfg.dependency;
        in {
          options.services.routeupd = with lib; {
            enable = mkEnableOption "routeupd service";

            interface = mkOption {
              type = types.str;
            };

            table = mkOption {
              type = types.int;
            };

            dependency = mkOption {
              type = types.nullOr types.str;
              default = null;
            };
          };

          config = lib.mkIf cfg.enable {
            systemd.services.routeupd = {
              after = [ "network-online.target" "nss-lookup.target" ] ++ depUnit;
              wantedBy = [ "multi-user.target" ];
              bindsTo = depUnit;
              serviceConfig = {
                DynamicUser = true;
                User = "routeupd";
                Group = "routeupd";
                AmbientCapabilities = [ "CAP_NET_ADMIN" ];
                Type = "notify";
                ExecStart = "${pkgs.routeupd}/bin/routeupd --daemon --interface ${cfg.interface} --table ${builtins.toString cfg.table}";
              };
            };
          };
        };
      overlay = final: prev: {
        routeupd = with final.rustPlatform; buildRustPackage {
          pname = "routeupd";
          version = "0.1.0";

          src = nix-filter.lib {
            root = ./.;
            include = [
              ./src
              ./Cargo.toml
              ./Cargo.lock
            ];
          };
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [ final.pkg-config ];
          buildInputs = [ final.openssl final.systemd.dev ];
        };
      };
    };
}
