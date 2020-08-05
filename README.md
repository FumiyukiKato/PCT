# PCT
Private Contact Tracing boosted by TEE(Intel SGX).  
As component of [this querying system](https://github.com/FumiyukiKato/tee-psi)  

## Setup

OS: Ubuntu 16.04 TLS

0. prepare CPU with Intel SGX instruction set and setup linux-sgx-driver

1. Clone Rust SGX SDK
```
$ git clone https://github.com/apache/incubator-teaclave-sgx-sdk
```

2. Clone this repository
```
$ git clone https://github.com/ylab-public/PCT
$ cd PCT
```

3. Set environment variables in `.env`. (using direnv)
```
RUST_SDK_ROOT=/path/to/incubator-teaclave-sgx-sdk
PCT_DIR=/path/to/PCT
```

4. Wake up docker container including AESM service
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
[docker-inside]$ bin/app 100000 data/sample/client.json data/sample/central.json true
```

#### data generator (python3)
```
$ python script/generator-script-name
```

#### materials

- Tamrakar, Sandeep, et al. "The circle game: Scalable private membership test using trusted hardware." Proceedings of the 2017 ACM on Asia Conference on Computer and Communications Security. 2017. [[pdf]](https://dl.acm.org/doi/pdf/10.1145/3052973.3053006)
- https://github.com/apache/incubator-teaclave-sgx-sdk
