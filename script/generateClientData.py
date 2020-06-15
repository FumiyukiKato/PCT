import os
import json
from Crypto.Cipher import AES
from Crypto.Random import get_random_bytes
import hashlib
import random
import datetime
import collections as cl

def gen_rand_geohash(length):
    geohash = [''] * length
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

# 少しそれっぽいデータを生成する
def gen_soreppoi_geohash(length):
    geohash = [''] * (length-1)
    # だいたい日本くらいのところ
    geohash[0] = 'xn'
    # 人間は偏在しているということで適当にBase16に削る
    # 12^8 = 4億くらい（日本のPOIってそんなもんじゃない？勘）
    small_base32 = '012345fghjkm'
    for i in range(length - 2):
        r = random.randint(0, 11)
        geohash[i+1] = small_base32[r]
    
    return ''.join(geohash)

# それっぽいクエリを生成する
def gen_soreppoi_query(query_size):
    soreppoi_query = []
    soreppoi_geohash = gen_soreppoi_geohash(10)
    start = datetime.datetime(2020, 6, 16)
    end = datetime.datetime(2020, 6, 30)
    # 2 weekをquery_sizeで分割する 2week = 2*7*24*60 = 20160 minutes query_size=1000なら20分間隔
    interval = (end - start) / query_size

    for i in range(query_size):
        dt = start + interval * i
        # だいたい100分に一回くらいgeohashをランダムに移動する感じで良さそう
        if random.random() < 0.2:
            soreppoi_geohash = gen_soreppoi_geohash(10)
        soreppoi_query.append(generateMergeByteData(str(int(dt.timestamp())), soreppoi_geohash))
    
    return soreppoi_query


def main():
    current_id = 0
    query_size = 4500
    client_size = 2000
    json_data = cl.OrderedDict()
    total_data_list = []
    for i in range(client_size):
        print("\r" + "generate process (%d/%d)" % (i+1, client_size), end="")
        data_list = gen_soreppoi_query(query_size)
        byte = b''.join(data_list)
        value = { "geodata": byte.hex(), "query_size": query_size, "query_id": current_id }
        total_data_list.append(value)
        current_id = current_id + 1

    json_data["data"] = total_data_list
    json_data["client_size"] = client_size
    
    print("\n" + "done!")
    now_timestamp = datetime.datetime.now().strftime("%Y%m%d%H%M%S")
    filename = './data/soreppoi_query/generated-client-query-qs-%d-cs-%s-%s.json' % (query_size, client_size, now_timestamp)
    with open(filename, 'w') as f:
        json.dump(json_data, f, indent=None)

main()