# Internal scripts

**DISCLAIMER**
HTTP req are done only at first test run. Later tests are reading HAR files
from the `recording/` dir. `make test` ensures that the previous state is
cleared.


Usecases:
1. As a dev I want to generate OpenAPI specification from HAR files
2. As a dev I want to have E2E tests

## Usage
```console
make test
```
```console
yarn run avantation recordings/path/to/file.har
```

verify with https://editor.swagger.io/
