{
  inputs = {
    nixpkgs.url = "git+ssh://git@github.com/NixOS/nixpkgs?ref=nixos-24.05&shallow=1";
    flake-utils.url = "git+ssh://git@github.com/poscat0x04/flake-utils?shallow=1";
    nix-filter.url = "git+ssh://git@github.com/numtide/nix-filter?shallow=1";
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

            calendar = mkOption {
              type = types.str;
              default = "*-*-* 04:00:00";
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
                Restart = "on-failure";
                RestartSec = "3s";
                Type = "oneshot";
                ExecStart = "${pkgs.routeupd}/bin/routeupd --interface ${cfg.interface} --table ${builtins.toString cfg.table}";
              };
            };
            systemd.timers.routeupd = {
              wantedBy = [ "multi-user.target" ];
              timerConfig = {
                OnCalendar = cfg.calendar;
                Persistent = true;
                AccuracySec = "1h";
                Unit = "routeupd.service";
              };
            };
          };
        };
      overlay = final: prev: {
        routeupd = let
          cargo-toml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        in with final.rustPlatform; buildRustPackage {
          pname = cargo-toml.package.name;
          version = cargo-toml.package.version;

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
