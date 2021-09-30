# PCT-TEE: Private Contact Tracing with Trusted Execution Environment

## Description

Existing Bluetooth-based Private Contact Tracing (PCT) systems can privately detect whether people have come into direct contact with COVID-19 patients. However, we find that the existing systems lack functionality and flexibility, which may hurt the success of the contact tracing. Specifically, they cannot detect indirect contact (e.g., people may be exposed to coronavirus because of used the same elevator even without direct contact); they also cannot flexibly change the rules of “risky contact”, such as how many hours of exposure or how close to a COVID-19 patient that is considered as risk exposure, which may be changed with the environmental situation.

In this work, we propose an efficient and secure contact tracing system that enables to trace both direct contact and indirect contact. To address the above problems, we need to utilize users’ trajectory data for private contact tracing, which we call trajectory-based PCT. We formalize this problem as Spatiotemporal Private Set Intersection. By analyzing different approaches such as homomorphic encryption that could be extended to solve this problem, we identify that Trusted Execution Environment (TEE) is a proposing method to achieve our requirements. The major challenge is how to design algorithms for spatiotemporal private set intersection under limited secure memory of TEE. To this end, we design a TEE-based system with flexible trajectory data encoding algorithms. Our experiments on real-world data show that the proposed system can process thousands of queries on tens of million records of trajectory data in a few seconds.


## Publication

- [**PSBD 2020**] Secure and Efficient Trajectory-Based Contact Tracing using Trusted Hardware.<br>
Fumiyuki Kato, Yang Cao, Masatoshi Yoshikawa.<br>
7th International Workshop on Privacy and Security of Big Data (PSBD 2020) @IEEE BigData 2020 <br>
https://arxiv.org/abs/2010.13381

- [**ACM Transactions on Spatial Algorithms and Systems**] PCT-TEE: Trajectory-based Private Contact Tracing System with Trusted Execution Environment <br>
Fumiyuki Kato, Yang Cao, Yoshikawa Masatoshi <br>
https://arxiv.org/abs/2012.03782

<img width="500" alt="carousel" src="https://user-images.githubusercontent.com/27177602/91183605-a3236200-e726-11ea-894b-ae7f419ca0b8.png">

## preliminary experiment
SOTA software-based approach vs hardware-based approach in PSI

`experiment/`

## Setup

OS: Ubuntu

0. prepare CPU with Intel SGX instruction set. Docker and docker-compose are needed.
And install linux-sgx-driver from https://github.com/intel/linux-sgx-driver.


1. Clone Rust SGX SDK
```
$ git clone https://github.com/apache/incubator-teaclave-sgx-sdk
```

2. Clone this repository
```
$ git clone https://github.com/ylab-public/PCT
$ cd PCT
```

3. Set environment variables like below. 
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
[docker-inside]$ make clean && QUERY_SIZE=1439 ENCODEDVALUE_SIZE=8 FEATURE="fsa st" make
```

#### run
```
[docker-inside]$ make clean && QUERY_SIZE=1439 ENCODEDVALUE_SIZE=8 FEATURE="fsa st" make && RUST_BACKTRACE=1 bin/app 10000 data/sample 2 data/sample/server.csv
```

```
    args[0] = threashold of each chunk block size
    args[1] = query data file dir (clientfile format => client-(theta_geo)-(theta_time)-(client_id)-(.+).csv
    args[2] = number of clients
    args[3] = central data file path"
```


#### Other materials
- SDK
  - https://github.com/apache/incubator-teaclave-sgx-sdk
- Experimental dataset source 
  - http://www.csis.u-tokyo.ac.jp/blog/research/joint-research/


