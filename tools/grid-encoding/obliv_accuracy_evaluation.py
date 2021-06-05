from subprocess import PIPE
import subprocess
import random
import json

start_time = 1597881600
theta_l_lng = 0.0000215
theta_l_lat = 0.0000165
DEBUG=False
seed = 0
random.seed(seed)


grid_l_lng = 0.0000215
grid_l_lat = 0.0000165
print('parameter', theta_l_lng, theta_l_lat)

total_result = {'tp': 0, 'fp': 0, 'tn': 0, 'fn': 0}
for i in range(100):
    r = random.randrange(20000)
    time = start_time+(r*60)

    ## Encoding with [SP20] both server data and client data and calculate matching results
    proc = subprocess.run(f'cargo run --release --  --theta-l-lng {grid_l_lng} --theta-l-lat {grid_l_lat} --theta-l-lng-max 72.60 --theta-l-lng-min 79.50 --theta-l-lat-max 44.80 --theta-l-lat-min 40.50 --time {time} --target pct --client-input-dir ../trajectory/data/client --output-file obliv-result-tmp.bin --input-file ../trajectory/data/server/NY-DensityEPR-1-0-1000.csv', shell=True, stdout=PIPE, stderr=PIPE, text=True, cwd=r"/home/fumiyuki/workspace/PCT/tools/grid-encoding")
    if DEBUG:
        print(proc.stderr)

    ## calculate accurate query
    proc = subprocess.run(f'cargo run --release -- -i ../trajectory/data/client -m obliv -o obliv_acc_results-tmp.bin --theta-t {time} --theta-l-lng {theta_l_lng} --theta-l-lat {theta_l_lat}', shell=True, stdout=PIPE, stderr=PIPE, text=True, cwd=r"/home/fumiyuki/workspace/PCT/tools/accurate_analysis")
    if DEBUG:
        print(proc.stderr)

    ## results analysis to aggregate results
    proc = subprocess.run(f'cargo run --release -- -u result-analysis --accurate-result-file ./obliv_acc_results-tmp.bin --pct-result-file ../grid-encoding/obliv-result-tmp.bin', shell=True, stdout=PIPE, stderr=PIPE, text=True, cwd=r"/home/fumiyuki/workspace/PCT/tools/accurate_analysis")
    if DEBUG:
        print(proc.stderr)

    result = json.loads(proc.stdout)
    print('TP: ', result['tp'], ', TN: ', result['tn'], ', FP: ', result['fp'], ', FN: ', result['fn'])
    total_result['tp'] += result['tp']
    total_result['fp'] += result['fp']
    total_result['fn'] += result['fn']
    total_result['tn'] += result['tn']

print('Total results: TP: ', total_result['tp'], ', TN: ', total_result['tn'], ', FP: ', total_result['fp'], ', FN: ', total_result['fn'])
