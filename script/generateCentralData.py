import os
import json
from Crypto.Cipher import AES
from Crypto.Random import get_random_bytes
import hashlib
import random
import datetime
import collections as cl

ACCURACY = 24

# それっぽいデータをうまく生成したい感じはある
def gen_rand_geohash(length):
    geohash = [''] * length
    base32 = '0123456789bcdefghjkmnpqrstuvwxyz'
    for i in range(length):
        r = random.randint(0, 31)
        geohash[i] = base32[r]
        
    return ''.join(geohash)

def gen_rand_timestamp():
    start = datetime.datetime(2020, 6, 16)
    end = datetime.datetime(2020, 6, 30)
    dt = random.random() * (end - start) + start
    return int(dt.timestamp())

# 少しそれっぽいデータを生成する
def gen_soreppoi_geohash(length):
    geohash = [''] * (length-1)
    # だいたい日本くらいのところ
    geohash[0] = 'xn'
    # 人間は偏在しているということで適当にBase16に削る
    # 12^8 = 4億くらい（日本のPOIってそんなもんじゃない？勘）
    # 感染者がいった場所だから100万POIくらいに絞りたい？
    # 8^8 = 1600万くらいにした
    base32 = '0123456789bcdefghjkmnpqrstuvwxyz'
    for i in range(length - 2):
        r = random.randint(0, ACCURACY-1)
        geohash[i+1] = base32[r]
    return ''.join(geohash)

def main():
    data_size = 10000000
    json_data = cl.OrderedDict()
    total_data_list = []
    for i in range(data_size):
        if (i+1) % 10 == 0:
            print("\r" + "generate process (%d/%d)" % (i+1, data_size), end="")
        timestamp = gen_rand_timestamp()
        geohash = gen_soreppoi_geohash(10).encode().hex()
        
        value = { "geohash": geohash, "unixepoch": timestamp }
        total_data_list.append(value)

    json_data["vec"] = total_data_list
    print("\n" + "done!")
    now_timestamp = datetime.datetime.now().strftime("%Y%m%d%H%M%S")
    filename = './data/soreppoi_central%s/generated-central-data-%d-%s.json' % (ACCURACY, data_size, now_timestamp)
    with open(filename, 'w') as f:
        json.dump(json_data, f, indent=None)

main()