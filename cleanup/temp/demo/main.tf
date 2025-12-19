resource "aws_instance" "example" {
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "t2.micro"

  tags = {
    Name = "Example"
  }
}

resource "aws_s3_bucket" "example" {
  bucket = "my-example-bucket"

  tags = {
    Environment = "Dev"
  }
}