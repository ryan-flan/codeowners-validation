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
           uses: ryan-flan/codeowners-validation-action@v1
   ```
   
Now, whenever a push or pull request is made to the main branch, the CODEOWNERS Validation GitHub Action will automatically run and validate the CODEOWNERS file.
Action Inputs

## Action inputs/outputs

The CODEOWNERS Validation GitHub Action currently doesn't require any inputs. It automatically looks for the CODEOWNERS file in the `.github/` directory of your repository. In future it's likely I will add options to:

- Customise which checks are run
- Point to different `CODEOWNERS` location

The CODEOWNERS Validation GitHub Action provides the following output:

The result of the CODEOWNERS file validation.

It can have one of the following values:
- success: The CODEOWNERS file is valid, and all the specified files and directories exist in the repository.
- failure: The CODEOWNERS file is invalid, or some of the specified files or directories are missing in the repository.

## Contributing

If you have any suggestions, bug reports, or feature requests, please open an issue or submit a pull request on the GitHub repository.

## License

This project is licensed under the MIT License.
