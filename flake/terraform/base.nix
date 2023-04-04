# just link i used to steal config
# https://zimbatm.com/notes/deploying-to-aws-with-terraform-and-nix
# https://xeiaso.net/blog/paranoid-nixos-aws-2021-08-11
# https://github.com/dzmitry-lahoda/web3nix
# security setup of some resources basically low to speed up, needs iterations of hardening
{ config, lib, options, specialArgs }:
let
  var = options.variable;

  # just ensure we do not to modify manually
  # really need to map all resources and automatically tag
  tags = {
    tool = "terranix";
  };
  volume_size_gb = 82;
in
rec {
  variable = {
    NODE_IMAGE = {
      type = "string";
    };

    AWS_REGION = {
      type = "string";
    };

    # assuming that it can be run by other validators
    VALIDATOR_NAME = {
      type = "string";
      description = "should be more than 3 but less then 12 symbols, only lower case letters";
    };
  };

  provider = {
    aws = {
      region = "\${var.AWS_REGION}";
    };
  };

  output = { 
   ami-a = {
      value = "\${resource.aws_ami.node-image-base.id}";
    };
  };

  # just for running some machines, here will be nixos-generators based VM uploaded to S3 with running validator 
  data = {
    # Permissions for the AWS instance
    aws_iam_policy_document = {
      machine = {
        statement = {
          sid = "1";
          # not secure
          actions = [
            "s3:ListAllMyBuckets"
            "s3:GetBucketLocation"
            "s3:ListBucket"
            "s3:GetObject"
            "s3:GetBucketLocation"
          ];

          resources = [
            "arn:aws:s3:::*"
          ];
        };
      };
    };

  };
  resource = {
    # A bunch of IAM resources needed to give permissions to the instance
    aws_iam_role = {
      machine = {
        name = "\${var.VALIDATOR_NAME}";

        assume_role_policy = ''
          {
            "Version": "2012-10-17",
            "Statement": [
              {
                "Action": "sts:AssumeRole",
                "Principal": {
                  "Service": "ec2.amazonaws.com"
                },
                "Effect": "Allow",
                "Sid": ""
              }
            ]
          }
        '';
      };
      vmimport = {
        inherit tags;
        name = "vmimport\${var.VALIDATOR_NAME}" ;
        assume_role_policy = ''
                  {
              "Version": "2012-10-17",
              "Statement": [
                  {
                      "Effect": "Allow",
                      "Principal": { "Service": "vmie.amazonaws.com" },
                      "Action": "sts:AssumeRole",
                      "Condition": {
                          "StringEquals":{
                              "sts:Externalid": "vmimport"
                          }
                      }
                  }
              ]
          }
        '';
      };

    };

    aws_iam_role_policy = {
      machine = {
        name = "\${var.VALIDATOR_NAME}";
        role = "\${aws_iam_role.machine.name}";
        policy = "\${data.aws_iam_policy_document.machine.json}";
      };

      vmimport_policy = {
        name = "vmimport\${var.VALIDATOR_NAME}";
        role = "\${aws_iam_role.vmimport.id}";
        policy = ''
          {
            "Version": "2012-10-17",
            "Statement": [
              {
                "Effect": "Allow",
                "Action": [
                  "s3:ListBucket",
                  "s3:GetObject",
                  "s3:GetBucketLocation"
                ],
                "Resource": [
                  "''${aws_s3_bucket.deploy.arn}",
                  "''${aws_s3_bucket.deploy.arn}/*"
                ]
              },
              {
                "Effect": "Allow",
                "Action": [
                  "s3:GetBucketLocation",
                  "s3:GetObject",
                  "s3:ListBucket",
                  "s3:PutObject",
                  "s3:GetBucketAcl"
                ],
                "Resource": [
                  "''${aws_s3_bucket.deploy.arn}",
                  "''${aws_s3_bucket.deploy.arn}/*"
                ]
              },
              {
                "Effect": "Allow",
                "Action": [
                  "ec2:ModifySnapshotAttribute",
                  "ec2:CopySnapshot",
                  "ec2:RegisterImage",
                  "ec2:Describe*"
                ],
                "Resource": "*"
              }
            ]
              }
        '';
      };

    };


    aws_s3_bucket = {
      deploy = {
        bucket = "deploy-instance-storage-\${var.VALIDATOR_NAME}";
        inherit tags;
      };
    };


    aws_s3_object = {
      nixos-a = {
        bucket = "\${aws_s3_bucket.deploy.bucket}";
        key = "nixos-amazon-\${var.VALIDATOR_NAME}.vhd";

        source = "\${var.NODE_IMAGE}";
        source_hash = "\${filemd5(var.NODE_IMAGE)}";


        lifecycle = {
          ignore_changes = [
            "key"
            "etag"
          ];
        };
      };
    };

    aws_ebs_snapshot_import = {
      nixos-a = {
        disk_container = {
          format = "VHD";
          user_bucket = {
            s3_bucket = "\${aws_s3_bucket.deploy.bucket}";
            s3_key = "\${aws_s3_object.nixos-a.key}";
          };
        };
        role_name = "\${aws_iam_role.vmimport.name}";
      };
    };

    aws_ami = {
      node-image-base = {
        name = "node-image-base-\${var.VALIDATOR_NAME}";
        architecture = "x86_64";
        virtualization_type = "hvm";
        root_device_name = "/dev/xvda";
        ena_support = true;
        sriov_net_support = "simple";

        ebs_block_device = {
          device_name = "/dev/xvda";
          snapshot_id = "\${aws_ebs_snapshot_import.nixos-a.id}";
          volume_size = volume_size_gb;
          delete_on_termination = true;
          volume_type = "gp3";
        };
      };
    };
  };


  backend = {
    local = {
      # that value is decrypted by script, update, and encrypted back
      path = "terraform-base.tfstate";
    };
  };

}
