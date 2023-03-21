{ config, lib, options, specialArgs }:
let
  var = options.variable;
  # really should do TF_VAR so can arbitrary global suffixes and experimentation
  projet-name = "ggchain";
  # just ensure we do not to modify manually
  tags = {
    tool = "terranix";
  };
in
rec {
  provider = {
    aws = {
      region = "eu-west-1";
    };
  };

  resource = {
    aws_s3_bucket = {
      terraform-backend = {
        # just for testing
        bucket = "new-just-some-storage-${projet-name}";
        inherit tags;
      };
    };
  };

  backend = {
    local = {
      # that value is decrypted by script, update, and encrypted back
      path = "terraform.tfstate";
    };
  };

}
