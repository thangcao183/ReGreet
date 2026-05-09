{
  description = "My ReGreet Dev Shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          pkg-config
          rustc
          cargo
        ];
        buildInputs = with pkgs; [
          gtk4
          glib
          cairo
          pango
          gdk-pixbuf
          graphene
          gst_all_1.gstreamer
          gst_all_1.gst-plugins-base
          gst_all_1.gst-plugins-good
          gst_all_1.gst-plugins-bad
          gst_all_1.gst-libav
          dbus
          accountsservice
          cage
        ];

        shellHook = ''
          echo "DevShell loaded!"
        '';
      };
    };
}
