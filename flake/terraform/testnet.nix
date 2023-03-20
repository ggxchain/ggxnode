# just link i used to steal config
# https://zimbatm.com/notes/deploying-to-aws-with-terraform-and-nix
# https://xeiaso.net/blog/paranoid-nixos-aws-2021-08-11
# https://github.com/dzmitry-lahoda/web3nix
# security setup of some resources basically low to speed up, needs iterations of hardening
{ config, lib, options, specialArgs }:
let
  var = options.variable;

  # just ensure we do not to modify manually
  tags = {
    tool = "terranix";
  };
  disk_size_in_gb = 84;
  instance_type = "t2.medium";
in
rec {
  variable = {
    # for PD.js and Metamask connection
    DOMAIN_NAME = {
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
  terraform = {
    required_providers = {
      acme = {
        source = "vancluever/acme";
        version = "2.13.1";
      };
      uptime = {
        source = "onesdata/uptime";
        version = "1.0.1";
      };
    };
  };
  provider = {
    aws = {
      region = "\${var.AWS_REGION}";
    };
    acme = {
      server_url = "https://acme-staging-v02.api.letsencrypt.org/directory";
    };
    uptime = { };
  };

  output = {
    node_public_dns_a = {
      value = "\${resource.aws_instance.node-a.public_dns}";
    };
    node_public_ip_a = { value = "\${resource.aws_instance.node-a.public_ip}"; };

    ssh_a = { value = "ssh -i ./terraform/testnet/id_rsa.pem root@\${resource.aws_instance.node-a.public_dns}"; };

    node_public_dns_b = {
      value = "\${resource.aws_instance.node-b.public_dns}";
    };
    node_public_ip_b = { value = "\${resource.aws_instance.node-b.public_ip}"; };

    ssh_b = { value = "ssh -i ./terraform/testnet/id_rsa.pem root@\${resource.aws_instance.node-b.public_dns}"; };

    node_public_dns_c = {
      value = "\${resource.aws_instance.node-c.public_dns}";
    };
    node_public_ip_c = { value = "\${resource.aws_instance.node-c.public_ip}"; };

    ssh_c = { value = "ssh -i ./terraform/testnet/id_rsa.pem root@\${resource.aws_instance.node-c.public_dns}"; };
    
    node_public_dns_d = {
      value = "\${resource.aws_instance.node-d.public_dns}";
    };
    node_public_ip_d = { value = "\${resource.aws_instance.node-d.public_ip}"; };

    ssh_d = { value = "ssh -i ./terraform/testnet/id_rsa.pem root@\${resource.aws_instance.node-d.public_dns}"; };
  };

  # just for running some machines, here will be nixos-generators based VM uploaded to S3 with running validator 
  data = {
    local_sensitive_file = {
      node-a-key-ed = {
        filename = "\${path.module}/../../.secret/node-a.ed25519.json";
      };
      node-a-key-sr = {
        filename = "\${path.module}/../../.secret/node-a.sr25519.json";
      };

      node-b-key-ed = {
        filename = "\${path.module}/../../.secret/node-b.ed25519.json";
      };
      node-b-key-sr = {
        filename = "\${path.module}/../../.secret/node-b.sr25519.json";
      };
      node-c-key-ed = {
        filename = "\${path.module}/../../.secret/node-c.ed25519.json";
      };
      node-c-key-sr = {
        filename = "\${path.module}/../../.secret/node-c.sr25519.json";
      };

      node-d-key-ed = {
        filename = "\${path.module}/../../.secret/node-d.ed25519.json";
      };
      node-d-key-sr = {
        filename = "\${path.module}/../../.secret/node-d.sr25519.json";
      };
    };

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
  resource =
    let
      mkSubZoneA = name: ip: {
        zone_id = "\${aws_route53_zone.primary.zone_id}";
        name = "${name}.\${var.DOMAIN_NAME}";
        type = "A";
        ttl = 300;
        # better use elastic ip to split machine from IP and make IP more stable (sta)
        records = [ ip ];
      };

      mkUpTime = name: {
        name = "Node";
        address = "${name}.\${var.DOMAIN_NAME}";
        contact_groups = [ "Default" ];
        interval = 1;
        locations = [ "US East" "United Kingdom" ];
      };

      mkMonitoredZone = name: ip: {
        uptime_check_http = {
          "${name}" = mkUpTime name;
        };
        aws_route53_health_check = {
          "${name}" = {
            fqdn = "\${aws_route53_record.${name}.name}";
            port = 443;
            type = "HTTPS";
            resource_path = "/";
            failure_threshold = "5";
            request_interval = "30";
            inherit tags;
          };
        };
        aws_route53_record = {
          "${name}" = mkSubZoneA name ip;
        };
      };

      a = mkMonitoredZone "node-a" "\${aws_instance.node-a.public_ip}";
      b = mkMonitoredZone "node-b" "\${aws_instance.node-b.public_ip}";
      c = mkMonitoredZone "node-c" "\${aws_instance.node-c.public_ip}";
      d = mkMonitoredZone "node-d" "\${aws_instance.node-d.public_ip}";
      
      monitored-zones = builtins.foldl' lib.recursiveUpdate {} [a b c d];
    in
    monitored-zones // {
      # generate a SSH key-pair
      tls_private_key = {
        machine = {
          algorithm = "RSA";
        };
      };

      # Record the SSH public key into AWS
      aws_key_pair = {
        machine = {
          key_name = "centralization-risk-\${var.VALIDATOR_NAME}";
          public_key = "\${tls_private_key.machine.public_key_openssh}";
        };
      };

      acme_registration = {
        reg = {
          account_key_pem = "\${tls_private_key.machine.private_key_pem}";
          email_address = "\${aws_route53domains_registered_domain.nodes.admin_contact[0].email}";
        };
      };
      # move to web layer to avoid rebuilds
      acme_certificate = {
        certificate = {
          account_key_pem = "\${acme_registration.reg.account_key_pem}";
          common_name = "\${aws_route53_zone.primary.name}";
          subject_alternative_names = [ "*.\${aws_route53_zone.primary.name}" ];

          dns_challenge = {
            provider = "route53";
            config = {
              AWS_DEFAULT_REGION = "\${var.AWS_REGION}";
              AWS_HOSTED_ZONE_ID = "\${aws_route53_zone.primary.zone_id}";
              AWS_PROPAGATION_TIMEOUT = 900;
              AWS_POLLING_INTERVAL = 60;
              AWS_MAX_RETRIES = 100;
            };
          };
        };
      };

      # Store the private key locally. This is going to be used by the deploy_nixos module below
      # to deploy NixOS.
      local_sensitive_file =
        {
          machine_ssh_key = {
            content = "\${tls_private_key.machine.private_key_pem}";
            filename = "id_rsa.pem";
            file_permission = "0600";
          };
        };

      aws_security_group = {
        machine = {
          name = "\${var.VALIDATOR_NAME}";
        };
      };

      # A bunch of rules for the group
      aws_security_group_rule = {
        machine_ingress_ssh = {
          description = "non secure access via all ports";
          type = "ingress";
          from_port = 0;
          to_port = 65535;
          protocol = "all";
          cidr_blocks = [ "0.0.0.0/0" ];
          security_group_id = "\${aws_security_group.machine.id}";
        };
        machine_egress_all = {
          description = "Allow to connect to the whole Internet";
          type = "egress";
          from_port = 0;
          to_port = 0;
          protocol = "-1";
          cidr_blocks = [ "0.0.0.0/0" ];
          security_group_id = "\${aws_security_group.machine.id}";
        };
      };

      # A bunch of IAM resources needed to give permissions to the instance
      aws_iam_role = {
        machine = {
          name = "machine-\${var.VALIDATOR_NAME}";

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
          name = "vmimport-\${var.VALIDATOR_NAME}";
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
      };

      aws_instance =
        let
          mkNode = key: {
            ami = "ami-05173ee0952a97126"; # us tf data either from cloud or from backend file
            inherit instance_type;
            security_groups = [
              "\${aws_security_group.machine.name}"
            ];
            key_name = "\${aws_key_pair.machine.key_name}";
            inherit tags;
            associate_public_ip_address = true;
            root_block_device = {
              volume_size = disk_size_in_gb;
            };

            # not ideal, better mount via user_data in script/cloud-init
            # or mount volume or build image
            provisioner = {
              file = [{
                source = "\${path.module}/../../.secret/${key}.ed25519.json";
                destination = "ed25519.json";
                connection = {
                  type = "ssh";
                  user = "root";
                  private_key = "\${local_sensitive_file.machine_ssh_key.content}";
                  host = "\${self.public_ip}";
                };
              }
                {
                  source = "\${path.module}/../../.secret/${key}.sr25519.json";
                  destination = "sr25519.json";
                  connection = {
                    type = "ssh";
                    user = "root";
                    private_key = "\${local_sensitive_file.machine_ssh_key.content}";
                    host = "\${self.public_ip}";
                  };
                }];
            };

            # really can mount keys and chain data storage here via ebs_block_device
          };
        in
        {
          node-a = mkNode "node-a";
          node-b = mkNode "node-b";
          node-c = mkNode "node-c";
          node-d = mkNode "node-d";
        };

      aws_route53domains_registered_domain = {
        nodes = {
          domain_name = "\${var.DOMAIN_NAME}";
          registrant_privacy = false;
          admin_privacy = false;
          tech_privacy = false;
          inherit tags;
          name_server = [
            { name = "\${aws_route53_zone.primary.name_servers[0]}"; }
            { name = "\${aws_route53_zone.primary.name_servers[1]}"; }
            { name = "\${aws_route53_zone.primary.name_servers[2]}"; }
            { name = "\${aws_route53_zone.primary.name_servers[3]}"; }
          ];
        };
      };

      aws_route53_zone = {
        primary = {
          name = "\${var.DOMAIN_NAME}";
        };
      };
    };
  backend = {
    local = {
      # that value is decrypted by script, update, and encrypted back
      path = "terraform-testnet.tfstate";
    };
  };
}
