
variable "project_name" {
    type = string
    default = "gcloud-aws-multi-dc"
}

provider "aws" {
    region = "us-west-2"
    alias = "us-west-2"
}

resource "aws_vpc" "epl-vpc-us-west-2" {
    provider = aws.us-west-2
    enable_dns_support = "true"
    enable_dns_hostnames = "false"
    instance_tenancy = "default"
    cidr_block = "10.19.0.0/16"
    assign_generated_ipv6_cidr_block = true
}

resource "aws_s3_bucket" "artefacts_bucket" {
    provider = aws.us-west-2

    bucket = "${var.project_name}-artefacts"
}

resource "aws_iam_role" "vmimport_role" {
    provider = aws.us-west-2

    name = "${var.project_name}-vmimport"
    assume_role_policy = jsonencode({
        Version = "2012-10-17"
        Statement = [
            {
                Effect = "Allow"
                Principal = { Service = "vmie.amazonaws.com" }
                Action = "sts:AssumeRole"
                Condition = {
                  StringEquals = {
                    "sts:Externalid" = "vmimport"
                  }
                }
            },
        ]
    })
}

resource "aws_iam_role_policy" "vmimport_role_policy" {
    provider = aws.us-west-2

    name = "${var.project_name}-vmimport"
    role = aws_iam_role.vmimport_role.name

    policy = jsonencode({
       Version = "2012-10-17"
       Statement = [
          {
             Effect = "Allow",
             Action = [
                "s3:GetBucketLocation",
                "s3:GetObject",
                "s3:ListBucket",
             ],
             Resource = [
                "arn:aws:s3:::${aws_s3_bucket.artefacts_bucket.id}",
                "arn:aws:s3:::${aws_s3_bucket.artefacts_bucket.id}/*"
             ]
          },
          {
             Effect = "Allow"
             Action = [
                "ec2:ModifySnapshotAttribute",
                "ec2:CopySnapshot",
                "ec2:RegisterImage",
                "ec2:Describe*",
             ],
             Resource = "*"
          }
       ]
    })
}

resource "aws_s3_object" "default_vm_image_object_x86_64" {
    provider = aws.us-west-2

    bucket = aws_s3_bucket.artefacts_bucket.id
    key = basename(trimspace(file("image-x86_64/aws-image.txt")))
    source = trimspace(file("image-x86_64/aws-image.txt"))
}

resource "null_resource" "ami_images_all_regions_x86_64" {
    depends_on = [ aws_s3_object.default_vm_image_object_x86_64 ]

    provisioner "local-exec" {
      command = "$AWS_IMAGE_UPLOAD_SCRIPT ${path.cwd}/image-x86_64/result"
      working_dir = "${path.cwd}/image-x86_64"
      environment = {
        home_region = "us-west-2"
        regions = ""
        bucket = aws_s3_bucket.artefacts_bucket.id
        service_role_name = aws_iam_role.vmimport_role.name
        state_dir = "${path.cwd}/image-x86_64/ec2-images"
        image_s3_key = aws_s3_object.default_vm_image_object_x86_64.key
        project_name = var.project_name
      }
    }
}


resource "aws_security_group" "public-node-epl-vpc-us-west-2" {
    provider = aws.us-west-2

    vpc_id = aws_vpc.epl-vpc-us-west-2.id
    name = "Public node"
}

resource "aws_vpc_security_group_egress_rule" "public-node-sg-epl-vpc-us-west-2-all" {
    provider = aws.us-west-2

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-2.id

    cidr_ipv4   = "0.0.0.0/0"
    ip_protocol = "-1"
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-epl-vpc-us-west-2-all" {
    provider = aws.us-west-2

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-2.id

    cidr_ipv4   = "10.0.0.0/8"
    ip_protocol = "-1"
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-epl-vpc-us-west-2-ssh" {
    provider = aws.us-west-2

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-2.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = 22
    ip_protocol = "tcp"
    to_port     = 22
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-epl-vpc-us-west-2-http" {
    provider = aws.us-west-2

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-2.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = 80
    ip_protocol = "tcp"
    to_port     = 80
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-epl-vpc-us-west-2-https" {
    provider = aws.us-west-2

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-2.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = 443
    ip_protocol = "tcp"
    to_port     = 443
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-epl-vpc-us-west-2-dns-tcp" {
    provider = aws.us-west-2

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-2.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = 53
    ip_protocol = "tcp"
    to_port     = 53
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-epl-vpc-us-west-2-dns-udp" {
    provider = aws.us-west-2

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-2.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = 53
    ip_protocol = "udp"
    to_port     = 53
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-epl-vpc-us-west-2-ping" {
    provider = aws.us-west-2

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-2.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = -1
    to_port     = -1
    ip_protocol = "icmp"
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-epl-vpc-us-west-2-wireguard" {
    provider = aws.us-west-2

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-2.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = 51820
    ip_protocol = "udp"
    to_port     = 51820
}



resource "aws_vpc_security_group_ingress_rule" "public-node-sg-ipv6-epl-vpc-us-west-2-http" {
    provider = aws.us-west-2

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-2.id

    cidr_ipv6   = "::/0"
    from_port   = 80
    ip_protocol = "tcp"
    to_port     = 80
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-ipv6-epl-vpc-us-west-2-https" {
    provider = aws.us-west-2

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-2.id

    cidr_ipv6   = "::/0"
    from_port   = 443
    ip_protocol = "tcp"
    to_port     = 443
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-ipv6-epl-vpc-us-west-2-dns-tcp" {
    provider = aws.us-west-2

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-2.id

    cidr_ipv6   = "::/0"
    from_port   = 53
    ip_protocol = "tcp"
    to_port     = 53
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-ipv6-epl-vpc-us-west-2-dns-udp" {
    provider = aws.us-west-2

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-2.id

    cidr_ipv6   = "::/0"
    from_port   = 53
    ip_protocol = "udp"
    to_port     = 53
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-ipv6-epl-vpc-us-west-2-ping" {
    provider = aws.us-west-2

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-2.id

    cidr_ipv6   = "::/0"
    from_port   = -1
    to_port     = -1
    ip_protocol = "icmpv6"
}



resource "aws_security_group" "private-node-epl-vpc-us-west-2" {
    provider = aws.us-west-2

    vpc_id = aws_vpc.epl-vpc-us-west-2.id
    name = "Private node"
}

resource "aws_vpc_security_group_egress_rule" "private-node-sg-epl-vpc-us-west-2-all" {
    provider = aws.us-west-2

    security_group_id = aws_security_group.private-node-epl-vpc-us-west-2.id

    cidr_ipv4   = "0.0.0.0/0"
    ip_protocol = "-1"
}

resource "aws_vpc_security_group_ingress_rule" "private-node-sg-epl-vpc-us-west-2-all" {
    provider = aws.us-west-2

    security_group_id = aws_security_group.private-node-epl-vpc-us-west-2.id

    cidr_ipv4   = "10.0.0.0/8"
    ip_protocol = "-1"
}


resource "aws_internet_gateway" "igw-epl-vpc-us-west-2" {
    provider = aws.us-west-2

    vpc_id = aws_vpc.epl-vpc-us-west-2.id
}

resource "aws_route_table" "internet-epl-vpc-us-west-2" {
    provider = aws.us-west-2

    vpc_id = aws_vpc.epl-vpc-us-west-2.id
}

resource "aws_route" "igw-route-epl-vpc-us-west-2" {
    provider = aws.us-west-2

    route_table_id         = aws_route_table.internet-epl-vpc-us-west-2.id
    destination_cidr_block = "0.0.0.0/0"
    gateway_id             = aws_internet_gateway.igw-epl-vpc-us-west-2.id
}

resource "aws_route" "igw-route-ipv6-epl-vpc-us-west-2" {
    provider = aws.us-west-2

    route_table_id              = aws_route_table.internet-epl-vpc-us-west-2.id
    destination_ipv6_cidr_block = "::/0"
    gateway_id                  = aws_internet_gateway.igw-epl-vpc-us-west-2.id
}


data "local_file" "ami_image_x86_64_us-west-2" {
    filename = "${path.cwd}/image-x86_64/ec2-images/us-west-2.x86_64-linux.ami_id"

    depends_on = [ null_resource.ami_images_all_regions_x86_64 ]
}

resource "aws_subnet" "subnet_10_19_0_0p24" {
    provider = aws.us-west-2

    vpc_id = aws_vpc.epl-vpc-us-west-2.id
    cidr_block = "10.19.0.0/24"
    availability_zone = "us-west-2c"
    ipv6_cidr_block = "${cidrsubnet(aws_vpc.epl-vpc-us-west-2.ipv6_cidr_block, 8, 19)}"
}

resource "aws_route_table_association" "igw-subnet_10_19_0_0p24" {
    provider = aws.us-west-2

    subnet_id = aws_subnet.subnet_10_19_0_0p24.id
    route_table_id = aws_route_table.internet-epl-vpc-us-west-2.id
}

resource "aws_instance" "server-e" {
    provider = aws.us-west-2

    ami = trimspace(data.local_file.ami_image_x86_64_us-west-2.content)
    source_dest_check = false
    instance_type = "c5n.large"
    associate_public_ip_address = true
    ipv6_address_count = 1

    tags = {
        Name = "gcloud-aws-multi-dc - server-e"
    }

    root_block_device {
      tags = {
          Name = "gcloud-aws-multi-dc - server-e - root"
      }
      volume_size = 20
      volume_type = "gp2"

    }

    private_ip = "10.19.0.10"
    subnet_id = aws_subnet.subnet_10_19_0_0p24.id
    vpc_security_group_ids = [ aws_security_group.public-node-epl-vpc-us-west-2.id ]

}

resource "aws_instance" "server-f" {
    provider = aws.us-west-2

    ami = trimspace(data.local_file.ami_image_x86_64_us-west-2.content)
    source_dest_check = false
    instance_type = "c5n.large"
    associate_public_ip_address = true
    ipv6_address_count = 1

    tags = {
        Name = "gcloud-aws-multi-dc - server-f"
    }

    root_block_device {
      tags = {
          Name = "gcloud-aws-multi-dc - server-f - root"
      }
      volume_size = 20
      volume_type = "gp2"

    }

    private_ip = "10.19.0.11"
    subnet_id = aws_subnet.subnet_10_19_0_0p24.id
    vpc_security_group_ids = [ aws_security_group.public-node-epl-vpc-us-west-2.id ]

}

resource "aws_instance" "server-i" {
    provider = aws.us-west-2

    ami = trimspace(data.local_file.ami_image_x86_64_us-west-2.content)
    source_dest_check = true
    instance_type = "c5n.large"
    associate_public_ip_address = false

    tags = {
        Name = "gcloud-aws-multi-dc - server-i"
    }

    root_block_device {
      tags = {
          Name = "gcloud-aws-multi-dc - server-i - root"
      }
      volume_size = 20
      volume_type = "gp2"

    }

    private_ip = "10.19.0.12"
    subnet_id = aws_subnet.subnet_10_19_0_0p24.id
    vpc_security_group_ids = [ aws_security_group.private-node-epl-vpc-us-west-2.id ]

}
