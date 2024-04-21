# CODEOWNERS Validation

Active checks:
- Checks the files and directories mentioned in the `CODEOWNERS` file actually exist in the repository.

TODO:
- Check files not owned
- Check owners exist in GitHub
- Check duplicate patterns
- Check syntax

## Currently useable as a GitHub Action

This GitHub Action validates the `CODEOWNERS` file in your repository to ensure that the specified code ownership rules are being followed. 

## Usage

To use the CODEOWNERS Validation GitHub Action in your repository, follow these steps:

- Inside the `.github/workflows` directory, create a new YAML file (e.g., `codeowners-validation.yml`) with the following content:

   ```yaml
   name: CODEOWNERS Validation

   on:
     push:
       branches: [main]
     pull_request:
       branches: [main]

   jobs:
     validate-codeowners:
       runs-on: ubuntu-latest

       steps:
         - uses: actions/checkout@v4

         - name: Run CODEOWNERS Validation
           uses: ryanjamesflanagan/codeowners-validation-action@v1
   ```
