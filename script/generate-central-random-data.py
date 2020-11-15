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
DISTRIBUTED = 60 # 86400 / DISTRIBUTED = number of bins
TH_BIT_LENGTH = 48
query_size = 1440
client_size = 200000
SIZE = 10000
# method = "gp"
method = "th"

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

def gen_trajectory_by_gp(query_size):
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

def get_trajectory_by_th(query_size):
    trajectory_data = []
    max_len = int(TH_BIT_LENGTH / 3)
    max_int = 2**(max_len)
    for i in range(query_size):
        r1 = random.randint(0, max_int)
        r2 = random.randint(0, max_int)
        r3 = random.randint(0, max_int)
        trajectory_data.append(zeroPadding(bin(r1)[2:], max_len) + zeroPadding(bin(r2)[2:], max_len) + zeroPadding(bin(r3)[2:], max_len))
    return trajectory_data

def zeroPadding(binaryStr, maxlength):
    lengthOfbinary = len(binaryStr)
    if lengthOfbinary >= maxlength:
        return binaryStr[(lengthOfbinary - maxlength):]
    else:
        return ''.join(['0']*(maxlength - lengthOfbinary)) + binaryStr

def main():
    json_data = cl.OrderedDict()
    total_data_list = []
    finish = False
    count = 0
    for _ in range(client_size):
        if finish:
            break
        data_list = []
        if method == "gp":
            data_list = gen_trajectory_by_gp(query_size)
        elif method == "th":
            data_list = get_trajectory_by_th(query_size)
        else:
            print("Error: method is nothing.")
            exit(1)
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
    filename = './data/random-uniform/central/generated-%s-%d-%s.json' % (method, SIZE, now_timestamp)
    with open(filename, 'w') as f:
        json.dump(json_data, f, indent=None)

main()