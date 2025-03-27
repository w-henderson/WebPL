import os
import json

from numpy import median

results_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(__file__))), "bench", "results")
out_dir = os.path.join(os.path.dirname(__file__), "data")

def process(path):
    data = json.load(open(path))
    benchmark_names = list(map(lambda x: x["name"], data[list(data.keys())[0]]))

    timeCSV = [["Benchmark"]] + [[name] for name in benchmark_names]
    memoryCSV = [["Benchmark"]] + [[name] for name in benchmark_names]

    for solver in data.keys():
        timeCSV[0].append(solver)
        if len(data[solver][0]["memorySamples"]) > 0:
            memoryCSV[0].append(solver)
        for j, benchmark in enumerate(data[solver]):
            timeCSV[j + 1].append(median(benchmark["timeSamples"]))
            if len(benchmark["memorySamples"]) > 0:
                memoryCSV[j + 1].append(median(benchmark["memorySamples"]))

    return timeCSV, memoryCSV

if __name__ == "__main__":
    if not os.path.exists(out_dir):
        os.makedirs(out_dir)

    for file in filter(lambda x: x.endswith(".json"), os.listdir(results_dir)):
        timeCSV, memoryCSV = process(os.path.join(results_dir, file))

        with open(os.path.join(out_dir, file.replace(".json", "") + ".time.csv"), "w") as f:
            f.write("\n".join([",".join(map(str, row)) for row in timeCSV]).replace("_", "\\_"))

        with open(os.path.join(out_dir, file.replace(".json", "") + ".memory.csv"), "w") as f:
            f.write("\n".join([",".join(map(str, row)) for row in memoryCSV]).replace("_", "\\_"))