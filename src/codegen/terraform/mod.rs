use crate::static_analysis::CheckedDB;

use super::{CodegenPlan, SshKeysSecrets, l1_provisioning::{utils::epl_arch_to_linux_arch, hardware::{zfs_args_aws, zfs_args_gcloud}}};

mod aws;
mod gcloud;

pub fn generate_terraform_outputs(checked: &CheckedDB, root_ssh_keys: &SshKeysSecrets, plan: &mut CodegenPlan) {
    let cd = &checked.projections.cloud_topologies;
    let default_nixpkgs = &checked.projections.default_used_nixpkgs_checksum;

    let data_dir = plan.root_dir().create_directory("data");
    data_dir.create_file_if_not_exists("supplements.edl", "".to_string());

    let admin_ssh_key = root_ssh_keys.public_root_ssh_key.value();
    let common_image_params = cloud_common_vm_image_params(&admin_ssh_key);
    let tf_dir = "terraform";
    let tf_dir = plan.root_dir().create_directory(tf_dir);
    let zfs_args = zfs_args_aws();
    if !cd.aws.is_empty() {
        let cloud_dir = tf_dir.create_directory("aws");
        for ua in &checked.projections.used_architectures {
            let linux_arch = epl_arch_to_linux_arch(ua);
            let dir_name = format!("image-{ua}");
            let arch_dir = cloud_dir.create_directory(&dir_name);
            arch_dir.create_file("aws-custom.nix", format!(r#"
{{ pkgs, modulesPath, ... }}:
{{
  imports = [
    "${{modulesPath}}/virtualisation/amazon-image.nix"
  ];

{zfs_args}

{common_image_params}
}}
"#));

            arch_dir.create_file("flake.nix", format!(r#"
{{
  inputs = {{
    nixpkgs.url = "github:NixOS/nixpkgs?rev={default_nixpkgs}";
    nixos-generators = {{
      url = "github:nix-community/nixos-generators?rev=246219bc21b943c6f6812bb7744218ba0df08600";
      inputs.nixpkgs.follows = "nixpkgs";
    }};
  }};
  outputs = {{ self, nixpkgs, nixos-generators, ... }}: {{
    packages.{linux_arch}-linux = {{
      default = nixos-generators.nixosGenerate {{
        system = "{linux_arch}-linux";
        modules = [
          # you can include your own nixos configuration here, i.e.
          ./aws-custom.nix
        ];
        format = "amazon-zfs";
      }};
    }};
  }};
}}
"#));
        }
        aws::generate_aws_outputs(checked, cloud_dir);
    }

    if !cd.gcloud.is_empty() {
        let zfs_args = zfs_args_gcloud();
        let cloud_dir = tf_dir.create_directory("gcloud");
        gcloud::generate_gcloud_outputs(checked, cloud_dir);
        for ua in &checked.projections.used_architectures {
            let linux_arch = epl_arch_to_linux_arch(ua);
            let dir_name = format!("image-{ua}");
            let arch_dir = cloud_dir.create_directory(&dir_name);
            arch_dir.create_file("gcloud-custom.nix", format!(r#"
{{ pkgs, modulesPath, ... }}:
{{
  imports = [
    "${{modulesPath}}/virtualisation/google-compute-config.nix"
  ];

{zfs_args}

{common_image_params}
}}
"#));
            arch_dir.create_file("flake.nix", format!(r#"
{{
  inputs = {{
    nixpkgs.url = "github:NixOS/nixpkgs?rev={default_nixpkgs}";
    nixos-generators = {{
      url = "github:nix-community/nixos-generators?rev=246219bc21b943c6f6812bb7744218ba0df08600";
      inputs.nixpkgs.follows = "nixpkgs";
    }};
  }};
  outputs = {{ self, nixpkgs, nixos-generators, ... }}: {{
    packages.{linux_arch}-linux = {{
      default = nixos-generators.nixosGenerate {{
        system = "{linux_arch}-linux";
        modules = [
          # you can include your own nixos configuration here, i.e.
          ./gcloud-custom.nix
        ];
        format = "gce-zfs";
      }};
    }};
  }};
}}
"#));
        }
    }
}

fn cloud_common_vm_image_params(admin_ssh_key: &str) -> String {
    let admin_ssh_key = admin_ssh_key.trim();
    format!(r#"
  users.users.root.hashedPassword = "!";
  security.sudo.wheelNeedsPassword = false;
  users.users.admin = {{
    isNormalUser = true;
    home = "/home/admin";
    extraGroups = [ "wheel" ];
    openssh.authorizedKeys.keys = [
      "{admin_ssh_key}"
    ];
  }};

  # we use cloud native firewalls
  networking.firewall.enable = false;
  # always have lan interface as eth0, anything else is confusing
  networking.usePredictableInterfaceNames = false;
  # bare minimum packages for fast bootstrap l1 provisioning
  environment.systemPackages = with pkgs; [
    gzip
    sqlite
    tmux
    git
    procmail # lockfile command for l1
  ];
"#)
}
