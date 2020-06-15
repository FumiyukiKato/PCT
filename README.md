# PCT
Private Contact Tracing boosted by TEE(Intel SGX).  
As component of [this querying system](https://github.com/FumiyukiKato/tee-psi)  

**move from https://github.com/FumiyukiKato/PCT**

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


#### materials

- Tamrakar, Sandeep, et al. "The circle game: Scalable private membership test using trusted hardware." Proceedings of the 2017 ACM on Asia Conference on Computer and Communications Security. 2017. [[pdf]](https://dl.acm.org/doi/pdf/10.1145/3052973.3053006)
- https://github.com/apache/incubator-teaclave-sgx-sdk
