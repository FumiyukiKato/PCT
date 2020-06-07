#!/bin/bash
echo "start aesm service"
echo ""
LD_LIBRARY_PATH=/opt/intel/sgx-aesm-service/aesm /opt/intel/sgx-aesm-service/aesm/aesm_service
sh
