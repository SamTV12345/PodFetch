# Content of this folder

This directory contains two other directories named:
- postgres-docker: Contains postgres terraform configuration files
- sqlite-docker: Contains sqlite configuration files


## How to use
### Install terraform
- Download terraform from [here](https://www.terraform.io/downloads.html)
- Extract the zip file
- Add the extracted folder to your PATH environment variable
- Run `terraform --version` to verify the installation
- You should see something like this:

- Before continuing review the variables in variables.tf and change accordingly.
- Run `terraform init` to initialize terraform
- Run `terraform plan` to see what terraform will do
- Run `terraform apply` to apply the changes

Now you should be able to access podfetch using the `SERVER_URL`
