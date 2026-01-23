#! python3
import glob
import json
import matplotlib.pyplot as plt
import numpy as np
import os
import re
from collections import defaultdict

LEGEND_LOOKUP = {
    "0": "Unavailable",
    "1": "None",
    "2": "Lexing",
    "3": "Parsing",
    "6": "Semantic",
    "23": "Valid",
}

os.makedirs("plots", exist_ok=True)


def natural_sort_key(s):
    return [int(c) if c.isdigit() else c.lower() for c in re.split(r"(\d+)", s)]


def extract_absolute_counts(data):
    """Extract absolute correctness counts from data."""
    absolute_counts = defaultdict(float)
    cat_order = []

    for cs in data.get("client_stats", {}).values():
        if cs.get("executions", 0) == 0:
            continue

        s_absolute = (
            cs.get("user_stats", {})
            .get("correctness-absolute", {})
            .get("value", {})
            .get("String", "")
        )

        if s_absolute:
            for pair in s_absolute.split(", "):
                if ":" in pair:
                    cat, count_str = pair.split(": ", 1)
                    absolute_counts[cat] += float(count_str)
                    if cat not in cat_order:
                        cat_order.append(cat)

    return absolute_counts, cat_order


def plot_stackplot(ax, times, data_arrays, labels, title, ylabel):
    """Helper function to create stack plots."""
    ax.stackplot(times, *data_arrays, labels=labels)
    ax.set_xlabel("Time (seconds)")
    ax.set_ylabel(ylabel)
    ax.set_title(title)
    handles, labels_legend = ax.get_legend_handles_labels()
    ax.legend(handles[::-1], labels_legend[::-1])


for log in glob.glob("out/*/stats.json"):
    # Extract data
    times_list, executions_list, lines_data, cat_order = [], [], [], []

    with open(log, "r") as f:
        print(log)
        for line in f:
            data = json.loads(line)
            rt = data.get("run_time", {})
            run_time = rt.get("secs", 0) + rt.get("nanos", 0) / 1e9

            if run_time > 3600:
                break

            absolute_counts, new_cats = extract_absolute_counts(data)
            if absolute_counts:
                times_list.append(run_time)
                executions_list.append(data.get("executions", 0))
                lines_data.append(absolute_counts)
                cat_order = list(set(cat_order + new_cats))

    if not times_list:
        continue

    # Convert to numpy arrays and sort
    times = np.array(times_list)
    executions = np.array(executions_list)
    sorted_idx = np.argsort(times)
    times = times[sorted_idx]
    executions = executions[sorted_idx]
    lines_data = [lines_data[i] for i in sorted_idx]

    # Build 2D array: rows = time points, cols = categories
    n_points, n_cats = len(lines_data), len(cat_order)
    cat_to_idx = {cat: idx for idx, cat in enumerate(cat_order)}
    absolute_array = np.zeros((n_points, n_cats))

    for i, counts in enumerate(lines_data):
        for cat, count in counts.items():
            if cat in cat_to_idx:
                absolute_array[i, cat_to_idx[cat]] = count

    # Compute cumulative ratios
    totals = np.sum(absolute_array, axis=1, keepdims=True)
    totals = np.where(totals > 0, totals, 1.0)
    cat_values_array = absolute_array / totals

    # Compute current rates
    time_diffs = np.diff(times)
    valid_mask = time_diffs > 0
    rate_times = times[1:][valid_mask]

    absolute_diffs = np.diff(absolute_array, axis=0)[valid_mask]
    time_diffs_valid = time_diffs[valid_mask, np.newaxis]
    current_rates_array = np.divide(
        absolute_diffs,
        time_diffs_valid,
        out=np.zeros_like(absolute_diffs),
        where=time_diffs_valid > 0,
    )

    # Smooth rates using moving average
    smoothed_times = None
    smoothed_ratios_array = None

    if len(rate_times) > 0:
        n_rate_points = len(rate_times)
        window_size = max(3, min(int(n_rate_points * 0.05), n_rate_points))
        if window_size % 2 == 0:
            window_size += 1

        kernel = np.ones(window_size) / window_size
        initial_ratios = (
            absolute_array[0] / np.sum(absolute_array[0])
            if np.sum(absolute_array[0]) > 0
            else np.zeros(n_cats)
        )

        # Smooth rates
        smoothed_rates = np.array(
            [
                np.convolve(current_rates_array[:, i], kernel, mode="same")
                for i in range(n_cats)
            ]
        ).T

        # Normalize to ratios
        totals_smooth = np.sum(smoothed_rates, axis=1, keepdims=True)
        smoothed_ratios = np.where(
            totals_smooth > 0, smoothed_rates / totals_smooth, initial_ratios
        )
        # Prepend initial ratios to match smoothed_times
        smoothed_ratios_array = np.vstack([initial_ratios, smoothed_ratios])
        smoothed_times = np.concatenate([[times[0]], rate_times])

    # Prepare for plotting
    name = os.path.basename(os.path.dirname(log))
    sorted_cats = sorted(cat_order, key=natural_sort_key)
    labels = [LEGEND_LOOKUP.get(cat, cat) for cat in sorted_cats]
    cat_arrays = [cat_values_array[:, cat_to_idx[cat]] for cat in sorted_cats]

    # Plot cumulative ratios
    fig, ax = plt.subplots(figsize=(12, 6))
    plot_stackplot(
        ax,
        times,
        cat_arrays,
        labels,
        f"Weighted Correctness Ratios Over Time - {name}",
        "Correctness Ratio",
    )
    plt.savefig(os.path.join("plots", f"correctness-{name}.png"), dpi=300)
    plt.close()

    # Plot executions
    fig, ax = plt.subplots(figsize=(12, 6))
    ax.plot(times, executions)
    ax.set_xlabel("Time (seconds)")
    ax.set_ylabel("Total Executions")
    ax.set_title(f"Total Executions Over Time - {name}")
    plt.savefig(os.path.join("plots", f"executions-{name}.png"), dpi=300)
    plt.close()

    # Plot smoothed current ratios
    if smoothed_times is not None and smoothed_ratios_array is not None:
        smoothed_arrays = [
            smoothed_ratios_array[:, cat_to_idx[cat]] for cat in sorted_cats
        ]

        fig, ax = plt.subplots(figsize=(12, 6))
        plot_stackplot(
            ax,
            smoothed_times,
            smoothed_arrays,
            labels,
            f"Current Correctness Rate Over Time (Moving Average) - {name}",
            "Current Correctness Ratio (Smoothed)",
        )
        plt.savefig(os.path.join("plots", f"correctness-current-{name}.png"), dpi=300)
        plt.close()
