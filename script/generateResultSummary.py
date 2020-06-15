import os
import json
import datetime
import re
from enum import Enum

class SummaryType(Enum):
    NORMAL = 0

def parse_from_result_files(dir_name):
    file_names = [ os.path.join(dir_name, file_name) for file_name in os.listdir(path=dir_name) ]
    results = []
    for file_name in file_names:
        result = {}
        with open(file_name) as f:
            for line in f:
                if not line:
                    break
                if any(line.startswith(x) for x in ["++++", "Basic data", "Clocker data", "  ", "\n", "---"]):
                    continue
                else:
                    name = list(filter(lambda x: line.startswith(x), [
                        "data structure type", 
                        "central data file", 
                        "query data file", 
                        "threashould", 
                        "response data type",
                        "Read Query Data",
                        "ECALL get_result",
                        "ECALL init_enclave",
                        "Distribute central data",
                        "ECALL upload_query_data",
                        "ECALL private_contact_trace",
                        "Read Central Data"
                    ]))[0]
                    i = line.find(": ")
                    result[name] = line[i+2:-1]
        results.append(result)
    return results

def build_results_summary(type, results, args):
    if type == SummaryType.NORMAL:
        make_normal_summary(results, args)

def make_normal_summary(results, args):
    normal_summary_data = []
    for result in results:
        data_structure_type = result["data structure type"]
        query_size, client_size = extract_from_query_file(result["query data file"])
        central_data_size = extract_from_central_file(result["central data file"])
        threashould = extract_int(result["threashould"])
        contact_trace_time = extract_secounds(result["ECALL private_contact_trace"])
        distribute_time = extract_secounds(result["Distribute central data"])
        upload_to_sgx_time = extract_secounds(result["ECALL upload_query_data"])
        data = {
            "data_structure_type": data_structure_type,
            "query_size": query_size,
            "client_size": client_size,
            "central_data_size": central_data_size,
            "threashould": threashould,
            "contact_trace_time": contact_trace_time,
            "distribute_time": distribute_time,
            "upload_to_sgx_time": upload_to_sgx_time
        }
        
        if "data_structure_type" in args and args.get("data_structure_type") != data_structure_type:
            continue
        if "query_size" in args and args.get("query_size") != query_size:
            continue
        if "central_data_size" in args and args.get("central_data_size") != central_data_size:
            continue
        if "client_size" in args and args.get("client_size") != client_size:
            continue
        if "threashould" in args and args.get("threashould") != threashould:
            continue
        if "contact_trace_time" in args and args.get("contact_trace_time") > contact_trace_time:
            continue
        normal_summary_data.append(data)
    
    sorted_data = sorted(normal_summary_data, key=lambda x: x["contact_trace_time"])
    if "sort_key" in args:
        sorted_data = sorted(normal_summary_data, key=lambda x: x[args.get("sort_key")])

    print("+++++++++++++++++++++++++++++++++++++++++++++++++++++++")
    print("Summary")
    print("+++++++++++++++++++++++++++++++++++++++++++++++++++++++")
    for data in sorted_data:
        print(" contact_trace_time : %s" % data["contact_trace_time"])
        print(" threashould        : %s" % data["threashould"])
        print(" query_size         : %s" % data["query_size"])
        print(" client_size        : %s" % data["client_size"])
        print(" central_data_size  : %s" % data["central_data_size"])
        print("-------------------------------------------------------")

def extract_from_query_file(file_name):
    matches = re.findall("generated-client-query-qs-(\d+)-cs-(\d+)", file_name)
    return int(matches[0][0]), int(matches[0][1])

def extract_from_central_file(file_name):
    matches = re.findall("generated-central-data-(\d+)-", file_name)
    return int(matches[0])

def extract_int(int_string):
    return int(int_string)

def extract_secounds(seconds_string):
    matches = re.findall("(\d.+) seconds", seconds_string)
    return float(matches[0])

"""
filter
args = {
    "sort_key": "central_data_size",
    "central_data_size": 1000000,
    "threashould": 1000000,
}
"""
def main():
    summary_type = SummaryType.NORMAL
    args = {
    }
    results =parse_from_result_files("data/result")
    build_results_summary(summary_type, results, args)

main()