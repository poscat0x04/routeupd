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
          devDep = [ "sys-subsystem-net-devices-${cfg.interface}.device" ];
        in {
          options.services.routeupd = with lib; {
            enable = mkEnableOption "routeupd service";

            interface = mkOption {
              type = types.str;
            };

            table = mkOption {
              type = types.int;
            };
          };

          config = lib.mkIf cfg.enable {
            systemd.services.routeupd = {
              wantedBy = devDep;
              after = [ "network-online.target" "nss-lookup.target" ] ++ devDep;
              bindsTo = devDep;
              serviceConfig = {
                DynamicUser = true;
                User = "routeupd";
                Group = "routeupd";
                AmbientCapabilities = [ "CAP_NET_ADMIN" ];
                CapabilityBoundingSet = [ "CAP_NET_ADMIN" ];
                SystemCallArchitectures = [ "native" ];
                ProtectClock = true;
                ProtectControlGroups = true;
                ProtectHome = true;
                ProtectHostname = true;
                ProtectKernelLogs = true;
                ProtectKernelModules = true;
                ProtectKernelTunables = true;
                ProtectProc = "noaccess";
                RestrictAddressFamilies = [ "AF_UNIX" "AF_INET" "AF_INET6" "AF_NETLINK" ];
                RestrictNamespaces = true;
                Restart = "always";
                RestartSec = "3s";
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
