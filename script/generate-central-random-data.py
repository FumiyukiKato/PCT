import os
import json
from Crypto.Cipher import AES
from Crypto.Random import get_random_bytes
import hashlib
import random
import datetime
import collections as cl

ACCURACY = 32
GEOHASH_LEN = 10
DISTRIBUTED = 60

def gen_from_uniform_distribution(length):
    geohash = [''] * (length)
    base32 = '0123456789bcdefghjkmnpqrstuvwxyz'
    for i in range(length):
        r = random.randint(0, ACCURACY-1)
        geohash[i] = base32[r]
    
    return ''.join(geohash)

def encode(geohash, start_unixepoch, unixepoch):
    diff = unixepoch - start_unixepoch
    return geohash + str(diff // DISTRIBUTED).zfill(4)

def gen_trajectory(query_size):
    trajectory_data = []
    start = datetime.datetime(2020, 6, 29)
    start_int = int(start.timestamp())
    end = datetime.datetime(2020, 6, 30)
    interval = (end - start) / query_size

    for i in range(query_size):
        dt = start + interval * i
        geohash = gen_from_uniform_distribution(GEOHASH_LEN)
        trajectory_data.append(encode(geohash, start_int, int(dt.timestamp())))
    
    return trajectory_data

def main():
    query_size = 1440
    client_size = 200000
    SIZE = 100000000
    
    json_data = cl.OrderedDict()
    total_data_list = []
    finish = False
    count = 0
    for _ in range(client_size):
        if finish:
            break
        data_list = gen_trajectory(query_size)
        for data in data_list:
            if count == SIZE:
                finish = True
                break
            if count % 100 == 0:
                print("\r" + "generate process (%d/%d)" % (count, SIZE), end="")
            total_data_list.append(data)
            count += 1

    json_data["data"] = total_data_list
    
    print("\n" + "done!")
    now_timestamp = datetime.datetime.now().strftime("%Y%m%d%H%M%S")
    filename = './data/random-uniform/central/generated-%d-%s.json' % (SIZE, now_timestamp)
    with open(filename, 'w') as f:
        json.dump(json_data, f, indent=None)

main()