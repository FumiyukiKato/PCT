## Preliminary experiment

Trusted hardware based approach vs software-based approach


#### SGX-based approach
- You need to prepare machine with Intel SGX and setup sdk.
```
$ cd sgxPSI/psi
$ make
$ ./app 1000000
```

#### Fastest OT-based approach
- Malicious Secure [RR17](https://eprint.iacr.org/2017/769) based on simple hashing and OTs
  - To execute https://github.com/osu-crypto/libPSI, you need to solve dependencies following its instructions.
  - There is no need to SGX implementation.

```
$ cd libPSI
$ bin/frontend.exe -rr17a -n 1000000
```
