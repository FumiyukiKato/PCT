# PCT
Private Contact Tracing boosted by TEE(Intel SGX).

## Build

set environment variables in `.env`.
```
RUST_SDK_ROOT=/path/to/incubator-teaclave-sgx-sdk
PCT_DIR=/path/to/PCT

PROJECT_NAME=PROJECT_NAME
```

#### wake up container including AESM service
```
$ bin/up
```

#### build
```
$ bin/in
[docker-inside]$ make
```

#### run
```
[docker-inside]$ bin/app
```
