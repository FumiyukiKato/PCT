import os
import json
from Crypto.Cipher import AES
from Crypto.Random import get_random_bytes
import hashlib
import random
import datetime
import collections as cl

def gen_rand_geohash(length):
    geohash = [''] * 32
    base32 = '0123456789bcdefghjkmnpqrstuvwxyz'
    for i in range(length):
        r = random.randint(0, 31)
        geohash[i] = base32[r]
        
    return ''.join(geohash)

def gen_rand_timestamp():
    start = datetime.datetime(2020, 6, 1)
    end = datetime.datetime(2020, 6, 30)
    dt = random.random() * (end - start) + start
    return str(int(dt.timestamp()))

def generateMergeByteData(timestamp, geohash):
    timestamp = timestamp.encode()
    geohash = geohash.encode()
    return timestamp + geohash

def main():
    current_id = 0
    query_size = 5
    client_size = 3
    json_data = cl.OrderedDict()
    total_data_list = []
    for i in range(client_size):
        data_list = []
        for j in range(query_size):
            timestamp = gen_rand_timestamp()
            geohash = gen_rand_geohash(10)
            mergedData = generateMergeByteData(timestamp, geohash)
            data_list.append(mergedData)
        
        byte = b''.join(data_list)
        value = { "geodata": byte.hex(), "query_size": query_size, "query_id": current_id }
        total_data_list.append(value)
        current_id = current_id + 1

    json_data["data"] = total_data_list
    json_data["client_size"] = client_size

    filename = './data/client-query-qs-%d-cs-%s.json' % (query_size, client_size)
    with open(filename, 'w') as f:
        json.dump(json_data, f, indent=4)

main()