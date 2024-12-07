{
  pkgs,
  nixosModule,
}: let
  listen_address = "127.0.0.1:1234";
in
  pkgs.nixosTest {
    name = "basic";
    nodes.machine = {pkgs, ...}: {
      imports = [nixosModule];
      networking.hostId = "039419bd"; #arbitrary
      services.null-webhook = {
        enable = true;
        inherit listen_address;
      };
    };
    testScript = ''
      machine.wait_for_unit("default.target")
      machine.wait_for_unit("null-webhook.service")
      machine.succeed("curl http://${listen_address}")
    '';
  }
