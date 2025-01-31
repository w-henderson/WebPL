import os
import json

from numpy import mean

results_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(__file__))), "bench", "results")
out_dir = os.path.join(os.path.dirname(__file__), "data")

for file in filter(lambda x: x.endswith(".json"), os.listdir(results_dir)):
    data = json.load(open(os.path.join(results_dir, file)))
    benchmark_names = list(map(lambda x: x["name"], data[list(data.keys())[0]]))

    timeCSV = [["benchmark"]] + [[name] for name in benchmark_names]
    memoryCSV = [["benchmark"]] + [[name] for name in benchmark_names]

    for solver in data.keys():
        timeCSV[0].append(solver)
        if len(data[solver][0]["memorySamples"]) > 0:
            memoryCSV[0].append(solver)
        for j, benchmark in enumerate(data[solver]):
            timeCSV[j + 1].append(mean(benchmark["timeSamples"]))
            if len(benchmark["memorySamples"]) > 0:
                memoryCSV[j + 1].append(mean(benchmark["memorySamples"]))

    if not os.path.exists(out_dir):
        os.makedirs(out_dir)

    with open(os.path.join(out_dir, file.replace(".json", "") + ".time.csv"), "w") as f:
        f.write("\n".join([",".join(map(str, row)) for row in timeCSV]).replace("_", "\\_"))

    with open(os.path.join(out_dir, file.replace(".json", "") + ".memory.csv"), "w") as f:
        f.write("\n".join([",".join(map(str, row)) for row in memoryCSV]).replace("_", "\\_"))