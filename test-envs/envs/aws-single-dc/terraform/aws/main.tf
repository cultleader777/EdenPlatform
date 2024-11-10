
variable "project_name" {
    type = string
    default = "aws-single-dc"
}

provider "aws" {
    region = "us-west-1"
    alias = "us-west-1"
}

resource "aws_vpc" "epl-vpc-us-west-1" {
    provider = aws.us-west-1
    enable_dns_support = "true"
    enable_dns_hostnames = "false"
    instance_tenancy = "default"
    cidr_block = "10.17.0.0/16"
    assign_generated_ipv6_cidr_block = true
}

resource "aws_s3_bucket" "artefacts_bucket" {
    provider = aws.us-west-1

    bucket = "${var.project_name}-artefacts"
}

resource "aws_iam_role" "vmimport_role" {
    provider = aws.us-west-1

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
    provider = aws.us-west-1

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
    provider = aws.us-west-1

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
        home_region = "us-west-1"
        regions = ""
        bucket = aws_s3_bucket.artefacts_bucket.id
        service_role_name = aws_iam_role.vmimport_role.name
        state_dir = "${path.cwd}/image-x86_64/ec2-images"
        image_s3_key = aws_s3_object.default_vm_image_object_x86_64.key
        project_name = var.project_name
      }
    }
}


resource "aws_security_group" "public-node-epl-vpc-us-west-1" {
    provider = aws.us-west-1

    vpc_id = aws_vpc.epl-vpc-us-west-1.id
    name = "Public node"
}

resource "aws_vpc_security_group_egress_rule" "public-node-sg-epl-vpc-us-west-1-all" {
    provider = aws.us-west-1

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-1.id

    cidr_ipv4   = "0.0.0.0/0"
    ip_protocol = "-1"
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-epl-vpc-us-west-1-all" {
    provider = aws.us-west-1

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-1.id

    cidr_ipv4   = "10.0.0.0/8"
    ip_protocol = "-1"
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-epl-vpc-us-west-1-ssh" {
    provider = aws.us-west-1

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-1.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = 22
    ip_protocol = "tcp"
    to_port     = 22
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-epl-vpc-us-west-1-http" {
    provider = aws.us-west-1

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-1.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = 80
    ip_protocol = "tcp"
    to_port     = 80
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-epl-vpc-us-west-1-https" {
    provider = aws.us-west-1

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-1.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = 443
    ip_protocol = "tcp"
    to_port     = 443
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-epl-vpc-us-west-1-dns-tcp" {
    provider = aws.us-west-1

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-1.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = 53
    ip_protocol = "tcp"
    to_port     = 53
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-epl-vpc-us-west-1-dns-udp" {
    provider = aws.us-west-1

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-1.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = 53
    ip_protocol = "udp"
    to_port     = 53
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-epl-vpc-us-west-1-ping" {
    provider = aws.us-west-1

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-1.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = -1
    to_port     = -1
    ip_protocol = "icmp"
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-epl-vpc-us-west-1-wireguard" {
    provider = aws.us-west-1

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-1.id

    cidr_ipv4   = "0.0.0.0/0"
    from_port   = 51820
    ip_protocol = "udp"
    to_port     = 51820
}



resource "aws_vpc_security_group_ingress_rule" "public-node-sg-ipv6-epl-vpc-us-west-1-http" {
    provider = aws.us-west-1

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-1.id

    cidr_ipv6   = "::/0"
    from_port   = 80
    ip_protocol = "tcp"
    to_port     = 80
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-ipv6-epl-vpc-us-west-1-https" {
    provider = aws.us-west-1

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-1.id

    cidr_ipv6   = "::/0"
    from_port   = 443
    ip_protocol = "tcp"
    to_port     = 443
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-ipv6-epl-vpc-us-west-1-dns-tcp" {
    provider = aws.us-west-1

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-1.id

    cidr_ipv6   = "::/0"
    from_port   = 53
    ip_protocol = "tcp"
    to_port     = 53
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-ipv6-epl-vpc-us-west-1-dns-udp" {
    provider = aws.us-west-1

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-1.id

    cidr_ipv6   = "::/0"
    from_port   = 53
    ip_protocol = "udp"
    to_port     = 53
}

resource "aws_vpc_security_group_ingress_rule" "public-node-sg-ipv6-epl-vpc-us-west-1-ping" {
    provider = aws.us-west-1

    security_group_id = aws_security_group.public-node-epl-vpc-us-west-1.id

    cidr_ipv6   = "::/0"
    from_port   = -1
    to_port     = -1
    ip_protocol = "icmpv6"
}



resource "aws_security_group" "private-node-epl-vpc-us-west-1" {
    provider = aws.us-west-1

    vpc_id = aws_vpc.epl-vpc-us-west-1.id
    name = "Private node"
}

resource "aws_vpc_security_group_egress_rule" "private-node-sg-epl-vpc-us-west-1-all" {
    provider = aws.us-west-1

    security_group_id = aws_security_group.private-node-epl-vpc-us-west-1.id

    cidr_ipv4   = "0.0.0.0/0"
    ip_protocol = "-1"
}

resource "aws_vpc_security_group_ingress_rule" "private-node-sg-epl-vpc-us-west-1-all" {
    provider = aws.us-west-1

    security_group_id = aws_security_group.private-node-epl-vpc-us-west-1.id

    cidr_ipv4   = "10.0.0.0/8"
    ip_protocol = "-1"
}


resource "aws_internet_gateway" "igw-epl-vpc-us-west-1" {
    provider = aws.us-west-1

    vpc_id = aws_vpc.epl-vpc-us-west-1.id
}

resource "aws_route_table" "internet-epl-vpc-us-west-1" {
    provider = aws.us-west-1

    vpc_id = aws_vpc.epl-vpc-us-west-1.id
}

resource "aws_route" "igw-route-epl-vpc-us-west-1" {
    provider = aws.us-west-1

    route_table_id         = aws_route_table.internet-epl-vpc-us-west-1.id
    destination_cidr_block = "0.0.0.0/0"
    gateway_id             = aws_internet_gateway.igw-epl-vpc-us-west-1.id
}

resource "aws_route" "igw-route-ipv6-epl-vpc-us-west-1" {
    provider = aws.us-west-1

    route_table_id              = aws_route_table.internet-epl-vpc-us-west-1.id
    destination_ipv6_cidr_block = "::/0"
    gateway_id                  = aws_internet_gateway.igw-epl-vpc-us-west-1.id
}


data "local_file" "ami_image_x86_64_us-west-1" {
    filename = "${path.cwd}/image-x86_64/ec2-images/us-west-1.x86_64-linux.ami_id"

    depends_on = [ null_resource.ami_images_all_regions_x86_64 ]
}

resource "aws_subnet" "subnet_10_17_0_0p24" {
    provider = aws.us-west-1

    vpc_id = aws_vpc.epl-vpc-us-west-1.id
    cidr_block = "10.17.0.0/24"
    availability_zone = "us-west-1b"
    ipv6_cidr_block = "${cidrsubnet(aws_vpc.epl-vpc-us-west-1.ipv6_cidr_block, 8, 17)}"
}

resource "aws_route_table_association" "igw-subnet_10_17_0_0p24" {
    provider = aws.us-west-1

    subnet_id = aws_subnet.subnet_10_17_0_0p24.id
    route_table_id = aws_route_table.internet-epl-vpc-us-west-1.id
}

resource "aws_instance" "server-a" {
    provider = aws.us-west-1

    ami = trimspace(data.local_file.ami_image_x86_64_us-west-1.content)
    source_dest_check = true
    instance_type = "c5n.large"
    associate_public_ip_address = false

    tags = {
        Name = "aws-single-dc - server-a"
    }

    root_block_device {
      tags = {
          Name = "aws-single-dc - server-a - root"
      }
      volume_size = 20
      volume_type = "gp2"

    }

    private_ip = "10.17.0.10"
    subnet_id = aws_subnet.subnet_10_17_0_0p24.id
    vpc_security_group_ids = [ aws_security_group.private-node-epl-vpc-us-west-1.id ]

}

resource "aws_instance" "server-b" {
    provider = aws.us-west-1

    ami = trimspace(data.local_file.ami_image_x86_64_us-west-1.content)
    source_dest_check = true
    instance_type = "c5n.large"
    associate_public_ip_address = false

    tags = {
        Name = "aws-single-dc - server-b"
    }

    root_block_device {
      tags = {
          Name = "aws-single-dc - server-b - root"
      }
      volume_size = 20
      volume_type = "gp2"

    }

    private_ip = "10.17.0.11"
    subnet_id = aws_subnet.subnet_10_17_0_0p24.id
    vpc_security_group_ids = [ aws_security_group.private-node-epl-vpc-us-west-1.id ]

}

resource "aws_instance" "server-c" {
    provider = aws.us-west-1

    ami = trimspace(data.local_file.ami_image_x86_64_us-west-1.content)
    source_dest_check = false
    instance_type = "c4.2xlarge"
    associate_public_ip_address = true
    ipv6_address_count = 1

    tags = {
        Name = "aws-single-dc - server-c"
    }

    root_block_device {
      tags = {
          Name = "aws-single-dc - server-c - root"
      }
      volume_size = 20
      volume_type = "gp2"

    }

    private_ip = "10.17.0.12"
    subnet_id = aws_subnet.subnet_10_17_0_0p24.id
    vpc_security_group_ids = [ aws_security_group.public-node-epl-vpc-us-west-1.id ]

}

resource "aws_instance" "server-d" {
    provider = aws.us-west-1

    ami = trimspace(data.local_file.ami_image_x86_64_us-west-1.content)
    source_dest_check = false
    instance_type = "c5n.large"
    associate_public_ip_address = true
    ipv6_address_count = 1

    tags = {
        Name = "aws-single-dc - server-d"
    }

    root_block_device {
      tags = {
          Name = "aws-single-dc - server-d - root"
      }
      volume_size = 20
      volume_type = "gp3"
      iops = 4000
      throughput = 200

    }

    private_ip = "10.17.0.13"
    subnet_id = aws_subnet.subnet_10_17_0_0p24.id
    vpc_security_group_ids = [ aws_security_group.public-node-epl-vpc-us-west-1.id ]

}

resource "aws_ebs_volume" "server-d-disk-sdf" {
    provider = aws.us-west-1

    availability_zone = "us-west-1b"
    size = 20
    type = "gp3"
    iops = 5000
    throughput = 250

    tags = {
      Name = "aws-single-dc - server-d - sdf"
    }
}

resource "aws_volume_attachment" "server-d-disk-sdf" {
    provider = aws.us-west-1

    device_name = "/dev/sdf"
    volume_id = aws_ebs_volume.server-d-disk-sdf.id
    instance_id = aws_instance.server-d.id
}

resource "aws_ebs_volume" "server-d-disk-sdg" {
    provider = aws.us-west-1

    availability_zone = "us-west-1b"
    size = 20
    type = "io1"
    iops = 1000

    tags = {
      Name = "aws-single-dc - server-d - sdg"
    }
}

resource "aws_volume_attachment" "server-d-disk-sdg" {
    provider = aws.us-west-1

    device_name = "/dev/sdg"
    volume_id = aws_ebs_volume.server-d-disk-sdg.id
    instance_id = aws_instance.server-d.id
}

resource "aws_ebs_volume" "server-d-disk-sdh" {
    provider = aws.us-west-1

    availability_zone = "us-west-1b"
    size = 100
    type = "io2"
    iops = 20000

    tags = {
      Name = "aws-single-dc - server-d - sdh"
    }
}

resource "aws_volume_attachment" "server-d-disk-sdh" {
    provider = aws.us-west-1

    device_name = "/dev/sdh"
    volume_id = aws_ebs_volume.server-d-disk-sdh.id
    instance_id = aws_instance.server-d.id
}
