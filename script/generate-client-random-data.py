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
query_size = 1440
client_size = 10
TH_BIT_LENGTH = 48
# method = "gp"
method = "th"

BASE8 = {
    '000': '0',
    '001': '1',
    '010': '2',
    '011': '3',
    '100': '4',
    '101': '5',
    '110': '6',
    '111': '7',
} 
def base8Encoding(binary):
    return ''.join([ BASE8.get("".join(binary[i:i+3])) for i in range(0, len(binary), 3)])

def gen_from_uniform_distribution(length):
    geohash = [''] * (length)
    base32 = '0123456789bcdefghjkmnpqrstuvwxyz'
    for i in range(length):
        r = random.randint(0, ACCURACY-1)
        geohash[i] = base32[r]
    
    return ''.join(geohash)

def encode(geohash, start_unixepoch, unixepoch):
    diff = unixepoch - start_unixepoch
    return (geohash + str(diff // DISTRIBUTED).zfill(4))

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
        binary = zeroPadding(bin(r1)[2:], max_len) + zeroPadding(bin(r2)[2:], max_len) + zeroPadding(bin(r3)[2:], max_len)        
        trajectory_data.append(base8Encoding(binary))
    return trajectory_data

def zeroPadding(binaryStr, maxlength):
    lengthOfbinary = len(binaryStr)
    if lengthOfbinary >= maxlength:
        return binaryStr[(lengthOfbinary - maxlength):]
    else:
        return ''.join(['0']*(maxlength - lengthOfbinary)) + binaryStr


def main():
    current_id = 0
    json_data = cl.OrderedDict()
    total_data_list = []
    for i in range(client_size):
        print("\r" + "generate process (%d/%d)" % (i+1, client_size), end="")
        data_list = []
        if method == "gp":
            data_list = gen_trajectory_by_gp(query_size)
        elif method == "th":
            data_list = get_trajectory_by_th(query_size)
        else:
            print("Error: method is nothing.")
            exit(1)
        value = { "geodata": data_list, "query_size": query_size, "query_id": current_id }
        total_data_list.append(value)        
        current_id += 1

    json_data["data"] = total_data_list
    json_data["client_size"] = client_size
    
    print("\n" + "done!")
    now_timestamp = datetime.datetime.now().strftime("%Y%m%d%H%M%S")
    filename = './data/random-uniform/client/generated-%s-%d-%s-%s.json' % (method, query_size, client_size, now_timestamp)
    with open(filename, 'w') as f:
        json.dump(json_data, f, indent=None)


main()