# PCT
The experimental code of Private Contact Tracing using SGX.  
Overview of the components.

<img width="500" alt="carousel" src="https://user-images.githubusercontent.com/27177602/91183605-a3236200-e726-11ea-894b-ae7f419ca0b8.png">

## Setup

OS: Ubuntu 16.04 TLS

0. prepare CPU with Intel SGX instruction set. Docker and docker-compose are needed.

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
[docker-inside]$ bin/app 1000000 data/sample/client.json data/sample/central.json true
```

#### random data generator (python3)
```
$ python script/generator-script-name
```

#### Other materials
- SDK
  - https://github.com/apache/incubator-teaclave-sgx-sdk
- Experimental dataset source 
  - http://www.csis.u-tokyo.ac.jp/blog/research/joint-research/
