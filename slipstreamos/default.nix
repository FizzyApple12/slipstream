{
  inputs,
  nixos-raspberrypi,
  ...
}:
nixos-raspberrypi.lib.nixosSystem {
  system = "aarch64-linux";
  specialArgs = inputs;
  modules = [
    ({...}: {
      imports = with nixos-raspberrypi.nixosModules; [
        sd-image
        raspberry-pi-5.base
        raspberry-pi-5.page-size-16k
        raspberry-pi-5.display-vc4
        ./configtxt.nix
      ];
    })
    ({lib, ... }:
    let
      inherit (lib) mkDefault;
    in
    {
      documentation = {
        enable = mkDefault false;
        doc.enable = mkDefault false;
        info.enable = mkDefault false;
        man.enable = mkDefault false;
        nixos.enable = mkDefault false;
      };

      environment = {
        defaultPackages = mkDefault [ ];
        stub-ld.enable = mkDefault false;
      };

      programs = {
        command-not-found.enable = mkDefault false;
        fish.generateCompletions = mkDefault false;
      };

      services = {
        logrotate.enable = mkDefault false;
        udisks2.enable = mkDefault false;
      };

      xdg = {
        autostart.enable = mkDefault false;
        icons.enable = mkDefault false;
        mime.enable = mkDefault false;
        sounds.enable = mkDefault false;
      };
    })
    ({lib, ...}: {
      time = {
        timeZone = "UTC";
      };

      networking = {
        hostName = "slipstream";
      };

      boot = {
        loader = {
          raspberry-pi = {
            bootloader = "kernel";
          };

          timeout = 0;
        };

        tmp = {
          useTmpfs = true;
        };

        supportedFilesystems = {
          zfs = lib.mkForce false;
        };

        plymouth = {
          enable = true;
        };

        consoleLogLevel = 3;

        initrd = {
          verbose = false;
        };

        kernelParams = [
          "quiet"
          "rd.udev.log_level=3"
          "rd.systemd.show_status=auto"
        ];
      };

      users = {
      	mutableUsers = false;

      	users.dj = {
      	  initialPassword = "thisIsMyMusic";
      	  isNormalUser = true;

      	  extraGroups = [
      	    "wheel"
      	    "networkmanager"
      	    "gpio"
      	    "i2c"
      	    "input"
      	    "plugdev"
      	    "spi"
      	    "video"
      	    "render"
      	  ];
       	};
      };

      services = {
        getty = {
          autologinUser = "dj";
        };

        openssh = {
          enable = true;
        };
      };

      security = {
        sudo = {
          enable = true;
          wheelNeedsPassword = false;
        };
      };

      nix = {
     	enable = true;
      };

      system = {
      	switch = {
       	  enable = false;
        };

      	stateVersion = "26.11";
      };
    })
    ({ nixpkgs, ... }: let
      system = "aarch64-linux";
      pkgs = import nixpkgs { inherit system; };
      slipstream-embedded = (pkgs.callPackage ../package.nix {});
    in {
      environment = {
        systemPackages = [
          slipstream-embedded
          pkgs.cage
        ];
      };

      systemd = {
        services = {
          dj-usb-mkdir = {
            enable = true;
            after = [ "multi-user.target" ];
            wantedBy = [ "default.target" ];
            description = "Ensure DJ USB mount directories exist";
            serviceConfig = {
              Type = "simple";
              ExecStart = pkgs.writeShellScript "dj-usb-mkdir.sh" ''
                ${pkgs.coreutils}/bin/mkdir -p /djusb/usb0 /djusb/usb1
                ${pkgs.coreutils}/bin/chown dj:users /djusb /djusb/usb0 /djusb/usb1
              '';
            };
          };
          ddj-flx10-driver = {
            enable = true;
            after = [ "multi-user.target" ];
            wantedBy = [ "default.target" ];
            description = "Run the DDJ-FLX10 driver";
            serviceConfig = {
              Type = "simple";
              User = "dj";
              Group = "users";
              ReadWritePaths = "/tmp/ /tmp/slipstream_ipc";
              ExecStart = "${slipstream-embedded.outPath}/bin/ddj_flx10 --log-path=\"/home/dj/ddj_flx10.log\"";
            };
          };
        };
      };

      services = {
        pipewire = {
          enable = true;
        };

        udev = { # ports usb 2-1, usb 5-1
          extraRules = ''
            SUBSYSTEM=="block", KERNELS=="2-1", ENV{ID_BUS}=="usb", ENV{ID_FS_USAGE}=="filesystem", ACTION=="add", RUN+="${pkgs.writeShellScript "udev-mount-usb0.sh" ''
            	${pkgs.systemd}/bin/systemd-mount --no-block /dev/$1 /djusb/usb0
            ''} $kernel"
            SUBSYSTEM=="block", KERNELS=="2-1", ENV{ID_BUS}=="usb", ENV{ID_FS_USAGE}=="filesystem", ACTION=="remove", RUN+="${pkgs.writeShellScript "udev-unmount-usb0.sh" ''
            	${pkgs.systemd}/bin/systemd-mount --no-block -u /djusb/usb0
            ''}"
            SUBSYSTEM=="block", KERNELS=="5-1", ENV{ID_BUS}=="usb", ENV{ID_FS_USAGE}=="filesystem", ACTION=="add", RUN+="${pkgs.writeShellScript "udev-mount-usb1.sh" ''
            	${pkgs.systemd}/bin/systemd-mount --no-block /dev/$1 /djusb/usb1
            ''} $kernel"
            SUBSYSTEM=="block", KERNELS=="5-1", ENV{ID_BUS}=="usb", ENV{ID_FS_USAGE}=="filesystem", ACTION=="remove", RUN+="${pkgs.writeShellScript "udev-unmount-usb1.sh" ''
            	${pkgs.systemd}/bin/systemd-mount --no-block -u /djusb/usb1
            ''}"
          '';
        };

        cage = {
          enable = true;
          extraArguments = [
            # "-m" "extend"
          ];
          program = "${slipstream-embedded.outPath}/bin/slipstream-embedded --log-path=\"/home/dj/slipstream.log\"";
          user = "dj";
        };
      };

      security = {
        rtkit = {
          enable = true;
        };
      };
    })
  ];
}
