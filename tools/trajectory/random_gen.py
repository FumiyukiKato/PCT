import pandas as pd
import numpy as np
from tqdm import tqdm

seed = 0

# starting and end times of the simulation
start_time = int(pd.to_datetime('2020/08/20 00:00:00').timestamp())
end_time = int(pd.to_datetime('2020/08/20 00:30:00').timestamp())

MIN_lng = -74.001
MAX_lng = -74.00
DIFF_lng = MAX_lng - MIN_lng

MIN_lat = 40.0
MAX_lat = 40.001
DIFF_lat = MAX_lat - MIN_lat

num_of_data = (end_time - start_time) // 60

num_of_client = 1000
num_of_server_data = 100000

def generate_one_client_random_data():
    data = {'time': [], 'lng': [], 'lat': []}
    
    curr_time = start_time
    lng_list = MIN_lng + np.random.rand(num_of_data)*DIFF_lng
    lat_list = MIN_lat + np.random.rand(num_of_data)*DIFF_lat
    
    for i in range(num_of_data):
        data['time'].append(curr_time)
        data['lng'].append(lng_list[i])
        data['lat'].append(lat_list[i])
        curr_time += 60
    return pd.DataFrame.from_dict(data)
    
def generate_server_random_data():
    df_list = []
    for i in tqdm(range(num_of_server_data)):
        df_list.append(generate_one_client_random_data())
    return pd.concat(df_list, axis=0)


np.random.seed(seed)

for client_id in tqdm(range(num_of_client)):
    client_df = generate_one_client_random_data()
    client_df.to_csv(f'random/client-{client_id}-random.csv', index=False, header=False)

server_df = generate_server_random_data()
server_df.to_csv(f'random/server-14000-random.csv', index=False, header=False)
