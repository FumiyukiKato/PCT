/*
 * Copyright (C) 2011-2020 Intel Corporation. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 *
 *   * Redistributions of source code must retain the above copyright
 *     notice, this list of conditions and the following disclaimer.
 *   * Redistributions in binary form must reproduce the above copyright
 *     notice, this list of conditions and the following disclaimer in
 *     the documentation and/or other materials provided with the
 *     distribution.
 *   * Neither the name of Intel Corporation nor the names of its
 *     contributors may be used to endorse or promote products derived
 *     from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 *
 */

#include "Enclave_t.h"

#include "string.h"
#include "stdlib.h"
#include "stdio.h"
#include "sgx_trts.h"
#include "sgx_thread.h"
#include "sgx_tseal.h"
#include <unordered_set>
#include <string>
#include "sgx_tcrypto.h"
#include "BloomFilter.h"

#define SGXSSL_CTR_BITS	128

void printByteArray(const uint8_t *arr, size_t size) {
    for (int i = 0; i < size; ++i) {
        char string_buf[8192] = {'\0'};
        snprintf(string_buf, 8192, "%u ", unsigned(arr[i]));
        print(string_buf);
    }
    print("\n");
}

void printInt(const int i) {
    char string_buf[BUFSIZ] = {'\0'};
    snprintf(string_buf, BUFSIZ, "%u\n", i);
    print(string_buf);
}

std::unordered_set<Data, Hasher> server_data_map = {};

BloomFilter *bf = NULL;

/* Suppose Remote Attestation is done, any untrusted applications cannot see this shared key. */
sgx_aes_ctr_128bit_key_t SHARED_KEY = {1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1};


// https://github.com/FumiyukiKato/tee-psi/blob/b2b241987498e4efea20f83e1a57062f5a692c87/SMCServer/enclave/src/lib.rs#L337
// decrypt client data

sgx_status_t upload_server_data(const uint64_t *server_data, size_t server_data_size)
{
    sgx_status_t ret;
    // print("[SGX] upload_server_data\n");

    for (int i=0; i<server_data_size/2; i++) {
        Data data { server_data[2*i], server_data[2*i+1] };
        server_data_map.insert(data);
        // char string_buf[BUFSIZ] = {'\0'};
        // snprintf(string_buf, BUFSIZ, "%#x: %u\n", server_data[2*i], server_data[2*i+1]);
        // print(string_buf);
        // print(std::to_string(i).c_str());
        // print("\n");


    }

    print("[SGX] server_data_map length: ");
    printInt(server_data_map.size());

    return ret;
}


sgx_status_t upload_and_psi(const uint8_t *client_data_buf, size_t client_data_buf_size, uint8_t *result, size_t client_data_size)
{
    sgx_status_t ret;

    /* decryption */
    uint8_t counter_block[16] = {0};
    uint32_t ctr_inc_bits = SGXSSL_CTR_BITS;
    uint8_t *decrypted_buf = (uint8_t*)malloc(sizeof(uint8_t)* client_data_buf_size);

    if ((decrypted_buf == NULL))
	{
        print("This is invalid");
		return SGX_ERROR_INVALID_PARAMETER;
	}
    ret = sgx_aes_ctr_decrypt(&SHARED_KEY, client_data_buf, client_data_buf_size, counter_block, ctr_inc_bits, decrypted_buf);
    if (ret != SGX_SUCCESS) {
        free(decrypted_buf);
        print("[SGX] error sgx_aes_ctr_decrypt: ");
        printInt(ret);
        return ret;
    }
    // print("[SGX] decrypted buf\n");
    // printByteArray(decrypted_buf, client_data_buf_size);


    /* psi */
    const uint8_t t = 1;
    const uint8_t f = 0;
    uint8_t *result_buf = (uint8_t*)malloc(sizeof(uint8_t)* client_data_size);
    for (int i=0; i<client_data_size; i++) {
        uint64_t u64_1, u64_2;
        memcpy(&u64_1, decrypted_buf +i*16, sizeof(u64_1));
        memcpy(&u64_2, decrypted_buf +i*16+8, sizeof(u64_2));
        Data data = Data {u64_1, u64_2};
        // char string_buf[BUFSIZ] = {'\0'};
        // snprintf(string_buf, BUFSIZ, "%#x: %u\n", u64_1, u64_2);
        // print(string_buf);
        if (server_data_map.find(data) != server_data_map.end()) {
            memcpy(result_buf +i, &t, sizeof(t));
        } else {
            memcpy(result_buf +i, &f, sizeof(f));
        }
    }
    print("[SGX] client_data_size length: ");
    printInt(client_data_size);
    free(decrypted_buf);

    /* encryption */
    uint8_t new_counter_block[16] = {0};
    ret = sgx_aes_ctr_encrypt(&SHARED_KEY, result_buf, client_data_size, new_counter_block, ctr_inc_bits, result);
    if (ret != SGX_SUCCESS) {
        free(result_buf);
        return ret;
    }
    free(result_buf);
    
    return ret;
}