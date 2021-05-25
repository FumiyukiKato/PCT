import argparse
from skmob.models.epr import DensityEPR, SpatialEPR
import skmob
import geopandas as gpd
import pandas as pd
import numpy as np
from tqdm import tqdm
import os


parser = argparse.ArgumentParser(description='Generate individual human mobility data by extended DensityEPR (see scimob https://github.com/scikit-mobility/scikit-mobility .)')
parser.add_argument('--target', type=str, help="target for dataset, server | client")
parser.add_argument('--size', type=int, help="data size")
parser.add_argument('--dir', type=str, help="directory name")
args = parser.parse_args()


## const
# load a spatial tesellation on which to perform the simulation
url = skmob.utils.constants.NY_COUNTIES_2011
tessellation = gpd.read_file(url)
# starting and end times of the simulation
start_time = pd.to_datetime('2020/08/13 00:00:00')
true_start_time = pd.to_datetime('2020/08/20 00:00:00')
end_time = pd.to_datetime('2020/09/03 00:00:00')


## override if necessary
class ConstantTimeDensityEPR(DensityEPR):
    def __init__(self):
        super().__init__()
        
     
def apply_minute(tdf, minutes=10):
    unix_minutes = minutes * 60
    return tdf['time'].apply(lambda x: x - (x % unix_minutes))

def gauusian_bridge(tdf, uid, random_state, minutes):
    unix_minutes = minutes * 60
    tmp_dict = {"uid": [], "time": [], "lat": [], "lng": []}
    table_size = len(tdf)
    for i in range(-1, table_size):
        if (i == table_size - 1):
            time_diff = int(end_time.timestamp()) - tdf['time'].iloc[i]
            curr_time = tdf['time'].iloc[i]
            i -= 10
        elif (i == -1):
            time_diff = tdf['time'].iloc[0] - int(true_start_time.timestamp())
            curr_time = int(true_start_time.timestamp())
            i += 10
        else:
            time_diff = tdf['time'].iloc[i+1] - tdf['time'].iloc[i]
            curr_time = tdf['time'].iloc[i]
    
        time_times = time_diff // unix_minutes
        if time_times == 0:
            continue

        lat_diff = (tdf['lat'].iloc[i+1] - tdf['lat'].iloc[i]) / time_times
        lng_diff = (tdf['lng'].iloc[i+1] - tdf['lng'].iloc[i]) / time_times
        
        curr_lat = tdf['lat'].iloc[i]
        curr_lng = tdf['lng'].iloc[i]
        
        random_walk = random_state.random(time_times)
        lat_noise = random_state.normal(lat_diff, np.abs(lat_diff/10), time_times+1)
        curr_lat += lat_noise[-1]
        lng_noise = random_state.normal(lng_diff, np.abs(lng_diff/10), time_times+1)
        curr_lng += lng_noise[-1]
        
        for j in range(time_times):
            tmp_dict["uid"].append(uid)
            tmp_dict["time"].append(curr_time)
            tmp_dict["lat"].append(curr_lat)
            tmp_dict["lng"].append(curr_lng)
            
            curr_time += unix_minutes
            if random_walk[j] < 0.5:
                curr_lat += lat_noise[j]
                curr_lng += lng_noise[j]

    df = pd.DataFrame.from_dict(tmp_dict)
    return df

def for_all_user(tdf, random_state, minutes=10):
    df_list = []
    for uid in tdf.groupby('uid').groups.keys():
        df_list.append(gauusian_bridge(tdf[tdf['uid'] == uid], uid, random_state, minutes))
    return pd.concat(df_list, axis=0)

def generate_server_data(agents=100, seed=1, minutes=10, dir="data"):
    state = np.random.RandomState(seed)
    start_locations = list(state.choice(list(range(0,62)), agents, True))
    depr = ConstantTimeDensityEPR()
    
    # start the simulation
    tdf = depr.generate(start_time, end_time, tessellation, relevance_column='population', n_agents=agents, random_state=seed, show_progress=True, starting_locations=start_locations)
    
    tdf = tdf[tdf['datetime'] >= true_start_time]
    tdf['time'] = tdf['datetime'].apply(lambda x: int(x.timestamp()))
    tdf['time'] = apply_minute(tdf, minutes=minutes)
    tdf = for_all_user(tdf, state, minutes)
    tdf[['time', 'lat', 'lng']].to_csv(f'{dir}/NY-DensityEPR-{minutes}-{seed}-{agents}.csv', index=False)

def generate_client_data(client_size, seed=1, minutes=10, dir="data"):
    agents = 1
    state = np.random.RandomState(seed)
    
    for i in tqdm(range(client_size)):
        gen_seed = state.randint(100000000)
        start_locations = list(state.choice(list(range(0,62)), agents, True))
        depr = ConstantTimeDensityEPR()
        # start the simulation
        tdf = depr.generate(start_time, end_time, tessellation, relevance_column='population', n_agents=agents, random_state=gen_seed, show_progress=True, starting_locations=start_locations)
    
        tdf = tdf[tdf['datetime'] >= true_start_time]
        tdf['time'] = tdf['datetime'].apply(lambda x: int(x.timestamp()))
        tdf['time'] = apply_minute(tdf, minutes=minutes)
        tdf = for_all_user(tdf, state, minutes)
        tdf[['time', 'lat', 'lng']].to_csv(f'{dir}/client-{i}-NY-DensityEPR-{minutes}-{seed}-{agents}.csv', index=False)

        
if __name__ == '__main__':
    minutes = 10
    seed = 1
    os.makedirs(args.dir, exist_ok=True)
    if args.target == "server":
        generate_server_data(agents=args.size, seed=seed, minutes=minutes, dir=args.dir)
    elif args.target == "client":
        generate_client_data(client_size=args.size, seed=seed, minutes=minutes, dir=args.dir)
    else:
        assert(False, "invalid parameter target")

    print("ok.")
