import os
import json
from Crypto.Cipher import AES
from Crypto.Random import get_random_bytes
import hashlib
import random
import datetime
import collections as cl

# それっぽいデータをうまく生成したい感じはある
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
    return int(dt.timestamp())

def main():
    data_size = 5
    json_data = cl.OrderedDict()
    total_data_list = []
    for i in range(data_size):
        timestamp = gen_rand_timestamp()
        geohash = gen_rand_geohash(10).encode().hex()
        
        value = { "geohash": geohash, "unixepoch": timestamp }
        total_data_list.append(value)

    json_data["vec"] = total_data_list

    filename = './data/generated-external-data-%d.json' % data_size
    with open(filename, 'w') as f:
        json.dump(json_data, f, indent=None)

main()