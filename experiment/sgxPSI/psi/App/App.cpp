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

#include <iostream>
#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <thread>
#include <vector>
#include <cstring>
#include "openssl/evp.h"
#include <iomanip>
#include <arpa/inet.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>
#include <netdb.h>
#include <errno.h>
#include <random>

#include <sgx_urts.h>
#include "App.h"
#include "Enclave_u.h"
#include "sgx_tcrypto.h"

#include <chrono>

#define SGXSSL_CTR_BITS	128
#define SHIFT_BYTE	8

const uint16_t BUFFER_SIZE = 0xFFFF;
const char *addr = "0.0.0.0";
const uint16_t port = 19999;
const uint16_t port2 = 19998;

sgx_aes_ctr_128bit_key_t SHARED_KEY = {1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1};

std::chrono::system_clock::time_point  start, end;

/*
* code taken from OpenSSL project.
* increment counter (128-bit int) by 1
*/
static void ctr128_inc(unsigned char *counter)
{
	unsigned int n = 16, c = 1;

	do {
		--n;
		c += counter[n];
		counter[n] = (unsigned char)c;
		c >>= SHIFT_BYTE;
	} while (n);
}

/* AES-CTR 128-bit
 * Parameters:
 *   Return:
 *     sgx_status_t - SGX_SUCCESS or failure as defined in sgx_error.h
 *   Inputs:
 *     sgx_aes_128bit_key_t *p_key - Pointer to the key used in encryption/decryption operation
 *     uint8_t *p_src - Pointer to the input stream to be encrypted/decrypted
 *     uint32_t src_len - Length of the input stream to be encrypted/decrypted
 *     uint8_t *p_ctr - Pointer to the counter block
 *     uint32_t ctr_inc_bits - Number of bits in counter to be incremented
 *   Output:
 *     uint8_t *p_dst - Pointer to the cipher text. Size of buffer should be >= src_len.
 */
sgx_status_t sgx_aes_ctr_encrypt(const sgx_aes_ctr_128bit_key_t *p_key, const uint8_t *p_src,
                                const uint32_t src_len, uint8_t *p_ctr, const uint32_t ctr_inc_bits,
                                uint8_t *p_dst)
{

	if ((src_len > INT_MAX) || (p_key == NULL) || (p_src == NULL) || (p_ctr == NULL) || (p_dst == NULL))
	{
		return SGX_ERROR_INVALID_PARAMETER;
	}

	/* SGXSSL based crypto implementation */
	sgx_status_t ret = SGX_ERROR_UNEXPECTED;
	int len = 0;
	EVP_CIPHER_CTX* ptr_ctx = NULL;

	// OpenSSL assumes that the counter is in the x lower bits of the IV(ivec), and that the
	// application has full control over overflow and the rest of the IV. This
	// implementation takes NO responsibility for checking that the counter
	// doesn't overflow into the rest of the IV when incremented.
	//
	if (ctr_inc_bits != SGXSSL_CTR_BITS)
	{
		return SGX_ERROR_INVALID_PARAMETER;
	}


	do {
		// Create and init ctx
		//
		if (!(ptr_ctx = EVP_CIPHER_CTX_new())) {
			ret = SGX_ERROR_OUT_OF_MEMORY;
			break;
		}

		// Initialise encrypt, key
		//
		if (1 != EVP_EncryptInit_ex(ptr_ctx, EVP_aes_128_ctr(), NULL, (unsigned char*)p_key, p_ctr)) {
			break;
		}

		// Provide the message to be encrypted, and obtain the encrypted output.
		//
		if (1 != EVP_EncryptUpdate(ptr_ctx, p_dst, &len, p_src, src_len)) {
			break;
		}

		// Finalise the encryption
		//
		if (1 != EVP_EncryptFinal_ex(ptr_ctx, p_dst + len, &len)) {
			break;
		}

		// Encryption success, increment counter
		//
		len = src_len;
		while (len >= 0) {
			ctr128_inc(p_ctr);
			len -= 16;
		}
		ret = SGX_SUCCESS;
	} while (0);

	//clean up ctx and return
	//
	if (ptr_ctx) {
		EVP_CIPHER_CTX_free(ptr_ctx);
	}
	return ret;
}

sgx_status_t sgx_aes_ctr_decrypt(const sgx_aes_ctr_128bit_key_t *p_key, const uint8_t *p_src,
                                const uint32_t src_len, uint8_t *p_ctr, const uint32_t ctr_inc_bits,
                                uint8_t *p_dst)
{

	if ((src_len > INT_MAX) || (p_key == NULL) || (p_src == NULL) || (p_ctr == NULL) || (p_dst == NULL)) {
		return SGX_ERROR_INVALID_PARAMETER;
	}

	/* SGXSSL based crypto implementation */
	sgx_status_t ret = SGX_ERROR_UNEXPECTED;
	int len = 0;
	EVP_CIPHER_CTX* ptr_ctx = NULL;

	// OpenSSL assumes that the counter is in the x lower bits of the IV(ivec), and that the
	// application has full control over overflow and the rest of the IV. This
	// implementation takes NO responsibility for checking that the counter
	// doesn't overflow into the rest of the IV when incremented.
	//
	if (ctr_inc_bits != SGXSSL_CTR_BITS) {
		return SGX_ERROR_INVALID_PARAMETER;
	}

	do {
		// Create and initialise the context
		//
		if (!(ptr_ctx = EVP_CIPHER_CTX_new())) {
			ret = SGX_ERROR_OUT_OF_MEMORY;
			break;
		}

		// Initialise decrypt, key and CTR
		//
		if (!EVP_DecryptInit_ex(ptr_ctx, EVP_aes_128_ctr(), NULL, (unsigned char*)p_key, p_ctr)) {
			break;
		}

		// Decrypt message, obtain the plaintext output
		//
		if (!EVP_DecryptUpdate(ptr_ctx, p_dst, &len, p_src, src_len)) {
			break;
		}

		// Finalise the decryption. A positive return value indicates success,
		// anything else is a failure - the plaintext is not trustworthy.
		//
		if (EVP_DecryptFinal_ex(ptr_ctx, p_dst + len, &len) <= 0) { // same notes as above - you can't write beyond src_len
			break;
		}
		// Success
		// Increment counter
		//
		len = src_len;
		while (len >= 0) {
			ctr128_inc(p_ctr);
			len -= 16;
		}
		ret = SGX_SUCCESS;
	} while (0);

	//cleanup ctx, and return
	//
	if (ptr_ctx) {
		EVP_CIPHER_CTX_free(ptr_ctx);
	}
	return ret;
}


/* Global EID shared by multiple threads */
sgx_enclave_id_t global_eid = 0;

typedef struct _sgx_errlist_t {
    sgx_status_t err;
    const char *msg;
    const char *sug; /* Suggestion */
} sgx_errlist_t;

// Ocall functions
void print(const char *str)
{
    std::cout << str;
}

void printUint(const uint64_t *arr, size_t size)
{
    for (int i=0; i<size/2; i++) {
        char string_buf[BUFSIZ] = {'\0'};
        snprintf(string_buf, BUFSIZ, "%#x: %u\n", (unsigned int)arr[2*i], (unsigned int)arr[2*i+1]);
        print(string_buf);
    }
}

/* Error code returned by sgx_create_enclave */
static sgx_errlist_t sgx_errlist[] = {
    {
        SGX_ERROR_UNEXPECTED,
        "Unexpected error occurred.",
        NULL
    },
    {
        SGX_ERROR_INVALID_PARAMETER,
        "Invalid parameter.",
        NULL
    },
    {
        SGX_ERROR_OUT_OF_MEMORY,
        "Out of memory.",
        NULL
    },
    {
        SGX_ERROR_ENCLAVE_LOST,
        "Power transition occurred.",
        "Please refer to the sample \"PowerTransition\" for details."
    },
    {
        SGX_ERROR_INVALID_ENCLAVE,
        "Invalid enclave image.",
        NULL
    },
    {
        SGX_ERROR_INVALID_ENCLAVE_ID,
        "Invalid enclave identification.",
        NULL
    },
    {
        SGX_ERROR_INVALID_SIGNATURE,
        "Invalid enclave signature.",
        NULL
    },
    {
        SGX_ERROR_OUT_OF_EPC,
        "Out of EPC memory.",
        NULL
    },
    {
        SGX_ERROR_NO_DEVICE,
        "Invalid SGX device.",
        "Please make sure SGX module is enabled in the BIOS, and install SGX driver afterwards."
    },
    {
        SGX_ERROR_MEMORY_MAP_CONFLICT,
        "Memory map conflicted.",
        NULL
    },
    {
        SGX_ERROR_INVALID_METADATA,
        "Invalid enclave meta.",
        NULL
    },
    {
        SGX_ERROR_DEVICE_BUSY,
        "SGX device was busy.",
        NULL
    },
    {
        SGX_ERROR_INVALID_VERSION,
        "Enclave version was invalid.",
        NULL
    },
    {
        SGX_ERROR_INVALID_ATTRIBUTE,
        "Enclave was not authorized.",
        NULL
    },
    {
        SGX_ERROR_ENCLAVE_FILE_ACCESS,
        "Can't open enclave file.",
        NULL
    },
};

struct Data {
    uint64_t value1;
    uint64_t value2;

    bool operator==(const Data& d) const
    {
        return value1 == d.value1 && value2 == d.value2;
    }
};

void printHexByteArray(uint8_t *arr, size_t size) {
    for (int i = 0; i < size; ++i)
        std::cout << std::hex << std::setfill('0') << std::setw(2) << unsigned(arr[i]) << " ";
    std::cout << std::endl;
}

void printByteArray(const uint8_t *arr, size_t size) {
    for (int i = 0; i < size; ++i) {
        char string_buf[8192] = {'\0'};
        snprintf(string_buf, 8192, "%u ", unsigned(arr[i]));
        print(string_buf);
    }
    print("\n");
}

/* Check error conditions for loading enclave */
void print_error_message(sgx_status_t ret)
{
    size_t idx = 0;
    size_t ttl = sizeof sgx_errlist/sizeof sgx_errlist[0];

    for (idx = 0; idx < ttl; idx++) {
        if(ret == sgx_errlist[idx].err) {
            if(NULL != sgx_errlist[idx].sug)
                printf("Info: %s\n", sgx_errlist[idx].sug);
            printf("Error: %s\n", sgx_errlist[idx].msg);
            break;
        }
    }

    if (idx == ttl)
        printf("Error: Unexpected error occurred.\n");
}

/* Initialize the enclave:
 *   Call sgx_create_enclave to initialize an enclave instance
 */
int initialize_enclave(void)
{
    sgx_status_t ret = SGX_ERROR_UNEXPECTED;
    
    /* Call sgx_create_enclave to initialize an enclave instance */
    /* Debug Support: set 2nd parameter to 1 */
    ret = sgx_create_enclave(ENCLAVE_FILENAME, SGX_DEBUG_FLAG, NULL, NULL, &global_eid, NULL);
    if (ret != SGX_SUCCESS) {
        print_error_message(ret);
        return -1;
    }

    return 0;
}


int serverProcess(int setSize) {
    
    /* generate_data */
    std::vector<Data> set(setSize);
    std::random_device rd;
    std::mt19937 mt(rd());
    for (uint64_t i = 0; i < setSize; i++)  {
        set[i] = Data { mt(), mt() };
        // set[i] = Data { 0, mt() };
        // std::cout << "server data: " << set[i].value1 << " " << set[i].value2 << std::endl;
    }
    set[10] = Data { 10, 10 };
    set[11] = Data { 10, 10 };
    

    /* Initialize the enclave */
    if(initialize_enclave() < 0){
        printf("Enter a character before exit ...\n");
        getchar();
        return -1; 
    }


    /* upload server data to sgx */
    size_t server_data_size = setSize*2;
    uint64_t *server_data = (uint64_t*)malloc(sizeof(uint64_t)* server_data_size);
    for (int i = 0; i < setSize; i++)  {
        server_data[i*2] = set[i].value1;
        server_data[i*2+1] = set[i].value2;
    }

    sgx_status_t retval;
    sgx_status_t ret = upload_server_data(global_eid, &retval, server_data, server_data_size);
    if (ret != SGX_SUCCESS)
    {
        print_error_message(ret);
        print_error_message(retval);
        printf("upload_server_data failed.\n");
        sgx_destroy_enclave(global_eid);
        return -1;
    }
    free(server_data);


    /* listen socket */
    bool isClosed = false;
    int sock = socket(PF_INET, SOCK_STREAM, IPPROTO_TCP);

    sockaddr_in address;
    if (inet_pton(AF_INET, addr, &address.sin_addr) <= 0) 
    {
        std::cout << errno << " : " << "Invalid address. Address type not supported." << std::endl;
        return -1;
    }
    address.sin_family = AF_INET;
    address.sin_port = htons(port);

    if (bind(sock, (const sockaddr *)&address, sizeof(address)) < 0)
    {
        std::cout << errno << " : " << "Cannot bind the socket." << std::endl;
        return -1;
    }

    if (listen(sock, 10) < 0) 
    {
        std::cout << errno << " : " << "Error: Server can't listen the socket." << std::endl;
        return -1;
    }
    

    /* receive data from client */
    std::vector<uint8_t> client_data_u8_vec;
    std::thread acceptThread([&client_data_u8_vec, &isClosed, &sock](){
        int newSock;
        sockaddr_in newAddr;
        socklen_t newAddrLength = sizeof(sockaddr_in);

        if (!isClosed)
        {
            if ((newSock = accept(sock, (sockaddr *)&newAddr, &newAddrLength)) < 0)
            {
                std::cout << errno << " : " << "Accept failed." << std::endl;
            }
            
            if (!isClosed && newSock >= 0)
            {
                std::thread receive([&client_data_u8_vec, &newSock, &sock, &isClosed](){
                    uint8_t tempBuf[BUFFER_SIZE];
                    int messageLength;

                    while ((messageLength = recv(newSock, tempBuf, BUFFER_SIZE, 0)))
                    {
                        if (messageLength < 0)  {
                            std::cout << errno << " : " << "Recv." << std::endl;
                            break;
                        }
                        
                        for (int i=0; i<messageLength; i++ )
                        {
                            client_data_u8_vec.push_back(tempBuf[i]);
                        }
                    }
                    close(newSock);

                    /* shut down server and close socket */
                    shutdown(sock, SHUT_RDWR);
                    if (!isClosed) {
                        isClosed = true;
                        close(sock);
                    }
                });
                receive.join();
            }
        }
    });
    acceptThread.join();
    

    /* upload client data to sgx and do PSI*/
    size_t client_data_size = client_data_u8_vec.size() / 16;
    std::cout << "client data size: " << client_data_size << std::endl;
    uint8_t *result_buf = (uint8_t*)malloc(sizeof(uint8_t)* client_data_size);
    ret = upload_and_psi(global_eid, &retval, client_data_u8_vec.data(), client_data_u8_vec.size(), result_buf, client_data_size);
    if (ret != SGX_SUCCESS || retval != SGX_SUCCESS)
    {
        print_error_message(ret);
        std::cout << "ret: " << ret << std::endl;
        print_error_message(retval);
        std::cout << "retval: " << retval << std::endl;
        printf("upload_server_data failed.\n");
        sgx_destroy_enclave(global_eid);
        return -1;
    }
    

    /* send result to client */
    int send_sock = socket(PF_INET, SOCK_STREAM, IPPROTO_TCP);
    sockaddr_in send_address;

    // ホスト名からIPv4のIPアドレスを引く
    struct addrinfo hints, *res, *it;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_INET;
    hints.ai_socktype = SOCK_STREAM;
    int status;
    if ((status = getaddrinfo(addr, NULL, &hints, &res)) != 0) {
        std::cout << errno << " : " << "Invalid Address." << std::string(gai_strerror(status)) << std::endl;
        return -1;
    }

    for(it = res; it != NULL; it = it->ai_next)
    {
        if (it->ai_family == AF_INET) {
            memcpy((void*)(&send_address), (void*)it->ai_addr, sizeof(sockaddr_in));
            break;
        }
    }
    freeaddrinfo(res);
    
    send_address.sin_family = AF_INET;
    send_address.sin_port = htons(port2);

    if (connect(send_sock, (const sockaddr*)&send_address, sizeof(sockaddr_in)) < 0)
    {
        std::cout << errno << " : " << "Connection failed to the host2." << std::endl;
        return -1;
    }

    int sent;
    if ((sent = send(send_sock, result_buf, client_data_size, 0)) < 0) {
        std::cout << errno << " : " << "Failed to send." << std::endl;
        return -1;
    }
    close(send_sock);
    free(result_buf);


    /* finish enclave */
    sgx_destroy_enclave(global_eid);
    return 0;
}

int clientProcess(int setSize) {
    usleep(200000000);
    
    /* generate_data */
    std::vector<Data> set(setSize);
    std::random_device rd;
    std::mt19937 mt(rd());
    for (uint64_t i = 0; i < setSize; ++i) {
        set[i] = Data { mt(), mt() };
        // set[i] = Data { 0, i };
        // std::cout << "client data: " << set[i].value1 << " " << set[i].value2 << std::endl;
    }
    set[4] = Data { 10, 10 };
    
    start = std::chrono::system_clock::now();

    /* encryption with RSA */
    size_t input_len = setSize*16;
    uint8_t *input_buf = (uint8_t*)malloc(sizeof(uint8_t)* input_len);
    for (int i=0; i<setSize; i++) {
        uint64_t v1 = set[i].value1;
        std::memcpy(input_buf + i*16, &v1, sizeof(v1));
        uint64_t v2 = set[i].value2;
        std::memcpy(input_buf + i*16+8, &v2, sizeof(v2));
    }

    uint8_t counter_block[16] = {0};
    uint32_t ctr_inc_bits = SGXSSL_CTR_BITS;
    uint8_t *encrypted_buf = (uint8_t*)malloc(sizeof(uint8_t)* input_len);
    
    sgx_status_t ret = sgx_aes_ctr_encrypt(&SHARED_KEY, input_buf, input_len, counter_block, ctr_inc_bits, encrypted_buf);
    if (ret != SGX_SUCCESS)
    {
        print_error_message(ret);
        std::cout << "ret: " << ret << std::endl;
        printf("sgx_aes_ctr_encrypt failed.\n");
        return -1;
    }
    free(input_buf);
    

    /* send data to server */
    int sock = socket(PF_INET, SOCK_STREAM, IPPROTO_TCP);
    bool isClosed = false;
    sockaddr_in address;

    // ホスト名からIPv4のIPアドレスを引く
    struct addrinfo hints, *res, *it;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_INET;
    hints.ai_socktype = SOCK_STREAM;
    int status;
    if ((status = getaddrinfo(addr, NULL, &hints, &res)) != 0) {
        std::cout << errno << " : " << "Invalid Address." << std::string(gai_strerror(status)) << std::endl;
        return -1;
    }
    
    for(it = res; it != NULL; it = it->ai_next)
    {
        if (it->ai_family == AF_INET) {
            memcpy((void*)(&address), (void*)it->ai_addr, sizeof(sockaddr_in));
            break;
        }
    }
    freeaddrinfo(res);

    address.sin_family = AF_INET;
    address.sin_port = htons(port);
    
    if (connect(sock, (const sockaddr*)&address, sizeof(sockaddr_in)) < 0)
    {
        std::cout << errno << " : " << "Connection failed to the host1." << std::endl;
        return -1;
    }

    if (isClosed)
        return -1;
    
    int sent;
    if ((sent = send(sock, encrypted_buf, input_len, 0)) < 0) {
        std::cout << errno << " : " << "Failed to send." << std::endl;
        return -1;
    }

    isClosed = true;
    close(sock);
    free(encrypted_buf);
    

    /* receive results from server */
    /* listen socket */
    isClosed = false;
    int rev_sock = socket(PF_INET, SOCK_STREAM, IPPROTO_TCP);

    sockaddr_in rcv_address;
    if (inet_pton(AF_INET, addr, &rcv_address.sin_addr) <= 0) 
    {
        std::cout << errno << " : " << "Invalid address. Address type not supported." << std::endl;
        return -1;
    }
    rcv_address.sin_family = AF_INET;
    rcv_address.sin_port = htons(port2);

    if (bind(rev_sock, (const sockaddr *)&rcv_address, sizeof(rcv_address)) < 0)
    {
        std::cout << errno << " : " << "Cannot bind the socket." << std::endl;
        return -1;
    }

    if (listen(rev_sock, 10) < 0) 
    {
        std::cout << errno << " : " << "Error: Server can't listen the socket." << std::endl;
        return -1;
    }
    
     
    /* receive data from server */
    std::vector<uint8_t> result_u8_vec;
    std::thread acceptThread([&result_u8_vec, &isClosed, &rev_sock](){
        int newSock;
        sockaddr_in newAddr;
        socklen_t newAddrLength = sizeof(sockaddr_in);

        if (!isClosed)
        {
            if ((newSock = accept(rev_sock, (sockaddr *)&newAddr, &newAddrLength)) < 0)
            {
                std::cout << errno << " : " << "Accept failed." << std::endl;
            }
            
            if (!isClosed && newSock >= 0)
            {
                std::thread receive([&result_u8_vec, &newSock, &rev_sock, &isClosed](){
                    uint8_t tempBuf[BUFFER_SIZE];
                    int messageLength;

                    while ((messageLength = recv(newSock, tempBuf, BUFFER_SIZE, 0)))
                    {
                        if (messageLength < 0)  {
                            std::cout << errno << " : " << "Recv." << std::endl;
                            break;
                        }
                        
                        for (int i=0; i<messageLength; i++ )
                        {
                            result_u8_vec.push_back(tempBuf[i]);
                        }
                    }
                    close(newSock);

                    /* shut down server and close socket */
                    shutdown(rev_sock, SHUT_RDWR);
                    if (!isClosed) {
                        isClosed = true;
                        close(rev_sock);
                    }
                });
                receive.join();
            }
        }
    });
    acceptThread.join();
    

    /* decrypt results and show result indices */
    uint8_t new_counter_block[16] = {0};
    uint8_t *decrypted_buf = (uint8_t*)malloc(sizeof(uint8_t)* setSize);
    ret = sgx_aes_ctr_decrypt(&SHARED_KEY, result_u8_vec.data(), setSize, new_counter_block, ctr_inc_bits, decrypted_buf);
    if (ret != SGX_SUCCESS)
    {
        print_error_message(ret);
        std::cout << "ret: " << ret << std::endl;
        printf("sgx_aes_ctr_decrypt failed.\n");
        return -1;
    }
    std::cout << "success." << std::endl;

    end = std::chrono::system_clock::now();
    double elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(end-start).count();
    std::cout << "time: " << elapsed << "[ms]" << std::endl;

    std::cout << "set intersection result: ";
    for (int i=0; i<setSize; i++) {
        if (decrypted_buf[i] == 1) {
            std::cout << i << ", ";
        }
    }
    
    free(decrypted_buf);

    return 0;
}

/* Application entry */
int main(int argc, char *argv[])
{
    size_t setSize = 0;
    if (argc < 1) {
        std::cout << "Usage: ./app server_size client_size" << std::endl;
    }
    if (argc == 2) {
        setSize = atoi(argv[1]);
    }
    
    auto thrd = std::thread([&]()
    {
        int ret = serverProcess(setSize);
        if (ret < 0) {
            std::cout << "Force exit" << std::endl;
            exit(0);
        }
    });

    if (argc == 3) {
        setSize = atoi(argv[2]);
    }

    clientProcess(setSize);
    thrd.join();
}
