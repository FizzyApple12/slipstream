{
  inputs,
  nixos-raspberrypi,
  development,
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
    {
      documentation = {
        enable = lib.mkDefault false;
        doc.enable = lib.mkDefault false;
        info.enable = lib.mkDefault false;
        man.enable = lib.mkDefault false;
        nixos.enable = lib.mkDefault false;
      };

      environment = {
        defaultPackages = lib.mkDefault [ ];
        stub-ld.enable = lib.mkDefault false;
      };

      programs = {
        command-not-found.enable = lib.mkDefault false;
        fish.generateCompletions = lib.mkDefault false;
      };

      services = {
        logrotate.enable = lib.mkDefault false;
        udisks2.enable = lib.mkDefault false;
      };

      xdg = {
        autostart.enable = lib.mkDefault false;
        icons.enable = lib.mkDefault false;
        mime.enable = lib.mkDefault false;
        sounds.enable = lib.mkDefault false;
      };
    })
    (if development then
    ({lib, nixpkgs, ...}:
    let
      system = "aarch64-linux";
      pkgs = import nixpkgs { inherit system; };
      qtEnv = with pkgs.qt6; env "qt-custom-${qtbase.version}" [
        qtdeclarative
        qtwayland
        qtshadertools
        qtsvg
        qtvirtualkeyboard
      ];
    in {
      environment = {
        systemPackages = [
          pkgs.vim
          pkgs.bash
          pkgs.git
          pkgs.pkg-config
          pkgs.cmake
          pkgs.rustup
          pkgs.clang
          pkgs.llvmPackages.bintools
          pkgs.yaml-language-server
          pkgs.rpiboot
          pkgs.minicom
          pkgs.qtcreator
          pkgs.btop
          pkgs.gdb
          pkgs.heaptrack
          pkgs.valgrind
          pkgs.zed-editor
          qtEnv
        ];
      };

      services = {
        openssh = {
          enable = lib.mkForce true;
        };
      };

      nix = {
        enable = lib.mkForce true;

        settings = {
          experimental-features = ["flakes" "nix-command"];
        };
      };
    })
    else
    ({...}:{}))
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

      hardware = {
        graphics = {
          enable = true;
        };
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
          enable = false;
        };
      };

      security = {
        sudo = {
          enable = true;
          wheelNeedsPassword = false;
        };
      };

      nix = {
     	enable = false;
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

          alsa = {
            enable = true;

            support32Bit = true;
          };

          pulse = {
            enable = true;
          };

          jack = {
            enable = true;
          };
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
