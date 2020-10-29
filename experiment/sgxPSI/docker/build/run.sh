# おそらくドライバのバージョンが古いという理由で/dev/sgx/enclaveではなく/dev/isgxを指定する
# aesmd-socketによる名前解決は特にされていないので/var/run/aesmdのままで
docker run --env http_proxy --env https_proxy --device=/dev/isgx -v /var/run/aesmd:/var/run/aesmd -it sgx_psi
