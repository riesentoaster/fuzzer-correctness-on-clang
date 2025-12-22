import glob
import json
import matplotlib.pyplot as plt
import os
import re
from collections import defaultdict

# Legend lookup dictionary: maps category keys to display labels
LEGEND_LOOKUP = {
    "0": "Unavailable",
    "1": "None",
    "2": "Lexing",
    "3": "Parsing",
    "6": "Semantic",
    "23": "Valid",
}

if not os.path.exists("plots"):
    os.makedirs("plots")


def natural_sort_key(s):
    return [int(c) if c.isdigit() else c.lower() for c in re.split(r"(\d+)", s)]


for log in glob.glob("out/*-2/stats.json"):
    times = []
    executions = []
    cat_values = defaultdict(list)
    cat_order = []
    lines_data = []

    with open(log, "r") as f:
        print(log)
        for line in f.readlines():
            data = json.loads(line)
            rt = data.get("run_time", {})
            run_time = rt.get("secs", 0) + rt.get("nanos", 0) / 1e9
            if run_time > 3600:
                break
            total_execs = data.get("executions", 0)

            weighted = defaultdict(float)
            total_weight = 0.0
            for cs in data.get("client_stats", {}).values():
                execs = cs.get("executions", 0)
                if execs == 0:
                    continue
                s = (
                    cs.get("user_stats", {})
                    .get("correctness", {})
                    .get("value", {})
                    .get("String", "")
                )
                if s:
                    for cat, prop in (p.split(": ") for p in s.split(", ") if ":" in p):
                        val = float(prop) * execs
                        weighted[cat] += val
                        total_weight += val
                        if cat not in cat_order:
                            cat_order.append(cat)

            if total_weight > 0:
                times.append(run_time)
                executions.append(total_execs)
                lines_data.append(weighted)

    if not times:
        continue

    sorted_indices = sorted(range(len(times)), key=lambda i: times[i])
    times = [times[i] for i in sorted_indices]
    executions = [executions[i] for i in sorted_indices]
    lines_data = [lines_data[i] for i in sorted_indices]

    for cat in cat_order:
        cat_values[cat] = []
        for weighted in lines_data:
            total = sum(weighted.values())
            cat_values[cat].append(weighted.get(cat, 0.0) / total if total > 0 else 0.0)

    name = os.path.basename(os.path.dirname(log))
    sorted_cats = sorted(cat_order, key=natural_sort_key)
    reversed_cats = sorted_cats  # list(reversed(sorted_cats))
    labels = [LEGEND_LOOKUP.get(cat, cat) for cat in reversed_cats]

    fig, ax = plt.subplots(figsize=(12, 6))
    ax.stackplot(times, *[cat_values[cat] for cat in reversed_cats], labels=labels)
    ax.set_xlabel("Time (seconds)")
    ax.set_ylabel("Correctness Ratio")
    ax.set_title(f"Weighted Correctness Ratios Over Time - {name}")
    handles, labels_legend = ax.get_legend_handles_labels()
    ax.legend(handles[::-1], labels_legend[::-1])
    plt.savefig(f"plots/correctness-{name}.png")
    plt.close()

    fig, ax = plt.subplots(figsize=(12, 6))
    ax.plot(times, executions)
    ax.set_xlabel("Time (seconds)")
    ax.set_ylabel("Total Executions")
    ax.set_title(f"Total Executions Over Time - {name}")
    plt.savefig(f"plots/executions-{name}.png")
    plt.close()
