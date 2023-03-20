{ config, lib, options, specialArgs }:
let
  var = options.variable;
in
rec {
  provider = {
    google = {
      region = "us-central1";
      zone = "us-central1-c";
      project = "\${ var.PROJECT }";
    };
  };
}
