from selenium import webdriver
from selenium.webdriver.chrome.options import Options
import base64
import os

chrome_options = Options()
chrome_options.add_argument("--headless")

ENGINES = [
    ("WebPL", "/webpl.html"),
    ("WebPL (GC)", "/webpl-gc.html"),
    ("SWI-Prolog", "/swipl.html"),
    ("Trealla Prolog", "/trealla.html"),
    ("Tau Prolog", "/tau.html"),
]

SUITE = os.path.join(os.path.dirname(__file__), "../suite")

def run(path, program, query = "top."):
    driver = webdriver.Chrome(options=chrome_options)
    program = base64.urlsafe_b64encode(program.encode()).decode()
    query = base64.urlsafe_b64encode(query.encode()).decode()
    url = f"http://localhost{path}?program={program}&query={query}"
    driver.get(url)

    while "Running..." in driver.page_source: pass

    return int(driver.find_element("id", "result").text)

if __name__ == "__main__":
    results = [["benchmark"] + [engine for (engine, _) in ENGINES]]

    for benchmark in sorted(os.listdir(SUITE)):
        benchmark_name = os.path.splitext(benchmark)[0]
        result = [benchmark_name]
        with open(os.path.join(SUITE, benchmark)) as file:
            program = file.read()

        for (engine, path) in ENGINES:
            time = run(path, program)
            print(f"{benchmark_name} {engine} {time}")
            result.append(time)

        results.append(result)

    with open("results.csv", "w") as file:
        file.write("\n".join([",".join(map(str, row)) for row in results]))