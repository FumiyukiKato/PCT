import pandas as pd
import numpy as np
from tqdm import tqdm

seed = 0
np.random.seed(seed)

CHARS = [
    '0',
    '1',
    '2',
    '3',
    '4',
    '5',
    '6',
    '7',
    '8',
    '9',
    'a',
    'b',
    'c',
    'd',
    'e',
    'f'
]

DATA_SIZE = 20160
SERVER_DATA_SIZE = 1000
BYTE_SIZE = 14

def generate_random_hash():
    
    hash_value = np.random.choice(CHARS, [BYTE_SIZE, DATA_SIZE*SERVER_DATA_SIZE], replace=True)
    return ''.join(hash_value)

def generate_one_client_random_data():
    data = {'value': []}
    hash_value = np.random.choice(CHARS, [DATA_SIZE*SERVER_DATA_SIZE, BYTE_SIZE], replace=True)
    
    for h in hash_value:
        data['value'].append(''.join(h))

    return pd.DataFrame.from_dict(data)

random_df = generate_one_client_random_data()
random_df.to_csv("random/server-14000-random.csv", index=False, header=False)