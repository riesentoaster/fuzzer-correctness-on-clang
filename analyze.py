#! python3
import argparse
from concurrent.futures import ThreadPoolExecutor
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
    "7": "Lambda",
    "18": "Inline Assembly",
    "23": "Valid",
}


def natural_sort_key(s):
    return [int(c) if c.isdigit() else c.lower() for c in re.split(r"(\d+)", s)]


def extract_absolute_counts(data):
    """Extract absolute correctness counts from data (summed across all clients)."""
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


def extract_per_client_counts(data):
    """Extract absolute correctness counts per client from data."""
    client_counts = {}
    cat_order = []

    for client_id, cs in data.get("client_stats", {}).items():
        if cs.get("executions", 0) == 0:
            continue

        s_absolute = (
            cs.get("user_stats", {})
            .get("correctness-absolute", {})
            .get("value", {})
            .get("String", "")
        )

        if s_absolute:
            client_counts[client_id] = defaultdict(float)
            for pair in s_absolute.split(", "):
                if ":" in pair:
                    cat, count_str = pair.split(": ", 1)
                    client_counts[client_id][cat] = float(count_str)
                    if cat not in cat_order:
                        cat_order.append(cat)

    return client_counts, cat_order


def interpolate_array(target_times, times, values):
    """Interpolate values at target_times using numpy's interp (vectorized, much faster)."""
    if len(times) == 0:
        return np.zeros(len(target_times))
    if len(times) == 1:
        return np.full(len(target_times), values[0])

    # Use numpy's interp for fast vectorized interpolation
    return np.interp(target_times, times, values)


def plot_stackplot(ax, times, data_arrays, labels, title, ylabel):
    """Helper function to create stack plots."""
    ax.stackplot(times, *data_arrays, labels=labels)
    ax.set_xlabel("Time (seconds)")
    ax.set_ylabel(ylabel)
    ax.set_title(title)
    handles, labels_legend = ax.get_legend_handles_labels()
    ax.legend(handles[::-1], labels_legend[::-1])


def max_coverage_over_clients(data):
    """At this snapshot, get max edges covered (absolute count) over all clients."""
    best = 0
    for cs in data.get("client_stats", {}).values():
        ratio = (
            cs.get("user_stats", {})
            .get("edges", {})
            .get("value", {})
            .get("Ratio", [0, 1])
        )
        if len(ratio) >= 1:
            best = max(best, int(ratio[0]))
    return best


def process_log(log):
    # Extract data
    times_list, executions_list, corpus_list, coverage_list, lines_data, cat_order = (
        [],
        [],
        [],
        [],
        [],
        [],
    )
    # Per-client data: client_id -> list of (time, category_counts dict)
    client_data = defaultdict(list)

    with open(log, "r") as f:
        print(f"Processing {log}")
        lines = f.readlines()
        error_lines = []
        for i, line in enumerate(lines):
            try:
                data = json.loads(line)
            except json.JSONDecodeError:
                if i < len(lines) - 1:  # ignore errors on the last line
                    error_lines.append(i + 1)
                continue

            rt = data.get("run_time", {})
            run_time = rt.get("secs", 0) + rt.get("nanos", 0) / 1e9

            # Apply time limit if specified
            if args.limit is not None and run_time > args.limit:
                break

            absolute_counts, new_cats = extract_absolute_counts(data)
            if absolute_counts:
                times_list.append(run_time)
                executions_list.append(data.get("executions", 0))
                corpus_list.append(data.get("corpus", 0))
                coverage_list.append(max_coverage_over_clients(data))
                lines_data.append(absolute_counts)
                cat_order = list(set(cat_order + new_cats))

            # Extract per-client data for interpolation
            client_counts, _ = extract_per_client_counts(data)
            for client_id, counts in client_counts.items():
                if counts:
                    client_data[client_id].append((run_time, counts))
                    # Update cat_order from per-client data too
                    for cat in counts.keys():
                        if cat not in cat_order:
                            cat_order.append(cat)
        if error_lines:
            print(
                f"Got {len(error_lines)} errors for file {log} at lines {', '.join(map(str, error_lines))}"
            )

    if not times_list:
        return

    # Convert to numpy arrays and sort
    times = np.array(times_list)
    executions = np.array(executions_list)
    corpus = np.array(corpus_list)
    coverage = np.array(coverage_list)
    sorted_idx = np.argsort(times)
    times = times[sorted_idx]
    executions = executions[sorted_idx]
    corpus = corpus[sorted_idx]
    coverage = coverage[sorted_idx]
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

    # Compute current rates using interval-based interpolation
    smoothed_times = None
    smoothed_ratios_array = None

    if len(times) > 0 and len(client_data) > 0:
        # Generate interval time points
        min_time = float(times[0])
        max_time = float(times[-1])
        interval_times = np.arange(min_time, max_time + args.interval, args.interval)

        # Pre-compute numpy arrays for each client's data (sorted by time)
        # Structure: client_arrays[client_id][cat] = (times_array, values_array)
        client_arrays = {}
        for client_id, client_times_data in client_data.items():
            if not client_times_data:
                continue
            # Sort by time
            client_times_data.sort(key=lambda x: x[0])
            client_times = np.array([t for t, _ in client_times_data])

            # Pre-compute arrays for each category
            client_arrays[client_id] = {}
            for cat in cat_order:
                client_values = np.array(
                    [counts.get(cat, 0.0) for _, counts in client_times_data]
                )
                client_arrays[client_id][cat] = (client_times, client_values)

        # Vectorized interpolation: for each category, sum interpolated values across all clients
        interpolated_absolute_array = np.zeros((len(interval_times), n_cats))

        for cat_idx, cat in enumerate(cat_order):
            # Interpolate for all interval times at once for each client, then sum
            for client_id in client_arrays:
                if cat in client_arrays[client_id]:
                    client_times, client_values = client_arrays[client_id][cat]
                    # Vectorized interpolation for all target times at once
                    interpolated_values = interpolate_array(
                        interval_times, client_times, client_values
                    )
                    interpolated_absolute_array[:, cat_idx] += interpolated_values

        # Calculate rates from differences between consecutive intervals
        if len(interval_times) > 1:
            time_diffs = np.diff(interval_times)
            valid_mask = time_diffs > 0
            rate_times = interval_times[1:][valid_mask]

            absolute_diffs = np.diff(interpolated_absolute_array, axis=0)[valid_mask]
            time_diffs_valid = time_diffs[valid_mask, np.newaxis]
            current_rates_array = np.divide(
                absolute_diffs,
                time_diffs_valid,
                out=np.zeros_like(absolute_diffs),
                where=time_diffs_valid > 0,
            )

            # Smooth rates using moving average
            if len(rate_times) > 0:
                n_rate_points = len(rate_times)
                window_size = max(3, min(int(n_rate_points * 0.05), n_rate_points))
                if window_size % 2 == 0:
                    window_size += 1

                kernel = np.ones(window_size) / window_size
                initial_ratios = (
                    interpolated_absolute_array[0]
                    / np.sum(interpolated_absolute_array[0])
                    if np.sum(interpolated_absolute_array[0]) > 0
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
                smoothed_times = np.concatenate([[interval_times[0]], rate_times])

    # Prepare for plotting
    name = os.path.basename(os.path.dirname(log))
    sorted_cats = sorted(cat_order, key=natural_sort_key)
    labels = [LEGEND_LOOKUP.get(cat, cat) for cat in sorted_cats]
    cat_arrays = [cat_values_array[:, cat_to_idx[cat]] for cat in sorted_cats]

    # Plot cumulative ratios
    fig, ax = plt.subplots(figsize=(12, 12))
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

    # Print final cumulative ratios at the last available time (limit if specified, otherwise total)
    final_cum_time = float(times[-1])
    final_cum_ratios = [cat_values_array[-1, cat_to_idx[cat]] for cat in sorted_cats]
    print(f"Final cumulative correctness ratios at t={final_cum_time:.2f}s for {name}:")
    for label, ratio in zip(labels, final_cum_ratios):
        print(f"  {label}: {ratio:.4f}")

    # Store for LaTeX cumulative table
    cum = {"name": name, "labels": labels, "ratios": final_cum_ratios}

    # Plot executions
    fig, ax = plt.subplots(figsize=(12, 12))
    ax.plot(times, executions)
    ax.set_xlabel("Time (seconds)")
    ax.set_ylabel("Total Executions")
    ax.set_title(f"Total Executions Over Time - {name}")
    plt.savefig(os.path.join("plots", f"executions-{name}.png"), dpi=300)
    plt.close()

    # Plot overall coverage (max over clients) vs time — absolute edges
    fig, ax = plt.subplots(figsize=(12, 12))
    ax.plot(times, coverage)
    ax.set_xlabel("Time (seconds)")
    ax.set_ylabel("Edges covered (max over clients)")
    ax.set_title(f"Coverage Over Time - {name}")
    plt.savefig(os.path.join("plots", f"coverage-{name}.png"), dpi=300)
    plt.close()

    # Log-log plot for coverage
    fig, ax = plt.subplots(figsize=(12, 12))
    # Avoid log(0): use at least 1 for y
    coverage_log = np.maximum(coverage, 1)
    min_time = max(times.min(), 1e-6) if len(times) > 0 else 1e-6
    ax.loglog(np.maximum(times, min_time), coverage_log)
    ax.set_xlabel("Time (seconds)")
    ax.set_ylabel("Edges covered (max over clients)")
    ax.set_title(f"Coverage Over Time (log-log) - {name}")
    plt.savefig(os.path.join("plots", f"coverage-loglog-{name}.png"), dpi=300)
    plt.close()

    # Plot corpus size (log scale on x-axis only)
    fig, ax = plt.subplots(figsize=(12, 12))
    ax.semilogx(times, corpus)
    # Anchor to minimum time value (avoid 0 for log scale)
    min_time = max(times.min(), 1e-6) if len(times) > 0 else 1e-6
    ax.set_xlim(left=min_time)
    ax.set_ylim(bottom=0)
    ax.set_xlabel("Time (seconds)")
    ax.set_ylabel("Corpus Size")
    ax.set_title(f"Corpus Size Over Time - {name}")
    plt.savefig(os.path.join("plots", f"corpus-{name}.png"), dpi=300)
    plt.close()

    # Plot smoothed current ratios
    if smoothed_times is not None and smoothed_ratios_array is not None:
        smoothed_arrays = [
            smoothed_ratios_array[:, cat_to_idx[cat]] for cat in sorted_cats
        ]

        # Print final current (smoothed) ratios at the last available time
        final_curr_time = float(smoothed_times[-1])
        final_curr_ratios = [
            smoothed_ratios_array[-1, cat_to_idx[cat]] for cat in sorted_cats
        ]
        print(
            f"Final current (smoothed) correctness ratios at t={final_curr_time:.2f}s for {name}:"
        )
        for label, ratio in zip(labels, final_curr_ratios):
            print(f"  {label}: {ratio:.4f}")

        # Store for LaTeX current table
        current = {"name": name, "labels": labels, "ratios": final_curr_ratios}

        fig, ax = plt.subplots(figsize=(12, 12))
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

    return cum, current


# After processing all logs, output LaTeX tables with the final ratios
def process_cumulative_results(cumulative_results):
    # Collect all category labels across runs
    all_labels_cum = sorted(
        {label for res in cumulative_results for label in res["labels"]},
        key=natural_sort_key,
    )

    print("\nCumulative correctness ratios (LaTeX table):")
    col_spec = "l" + "c" * len(all_labels_cum)
    print(f"\\begin{{tabular}}{{{col_spec}}}")
    header_cells = ["Run"] + list(all_labels_cum)
    print(" & ".join(header_cells) + " \\\\")
    print("\\hline")
    for res in cumulative_results:
        label_to_ratio = dict(zip(res["labels"], res["ratios"]))
        row = [res["name"]]
        for label in all_labels_cum:
            ratio = label_to_ratio.get(label, 0.0)
            row.append(f"{ratio:.4f}")
        print(" & ".join(row) + " \\\\")
    print("\\end{tabular}")


def process_current_results(current_results):
    # Collect all category labels across runs
    all_labels_cur = sorted(
        {label for res in current_results for label in res["labels"]},
        key=natural_sort_key,
    )

    print("\nCurrent (smoothed) correctness ratios (LaTeX table):")
    col_spec = "l" + "c" * len(all_labels_cur)
    print(f"\\begin{{tabular}}{{{col_spec}}}")
    header_cells = ["Run"] + list(all_labels_cur)
    print(" & ".join(header_cells) + " \\\\")
    print("\\hline")
    for res in current_results:
        label_to_ratio = dict(zip(res["labels"], res["ratios"]))
        row = [res["name"]]
        for label in all_labels_cur:
            ratio = label_to_ratio.get(label, 0.0)
            row.append(f"{ratio:.4f}")
        print(" & ".join(row) + " \\\\")
    print("\\end{tabular}")


if __name__ == "__main__":
    os.makedirs("plots", exist_ok=True)

    parser = argparse.ArgumentParser(description="Analyze fuzzer correctness data")
    parser.add_argument(
        "--limit",
        type=float,
        default=None,
        help="Only parse and plot times up to this limit in seconds from the start of the campaign",
    )
    parser.add_argument(
        "--interval",
        type=float,
        default=10.0,
        help="Time interval in seconds for calculating current rates (default: 10.0)",
    )
    parser.add_argument(
        "--parallelize",
        action="store_true",
        help="Parallelize the processing of logs",
    )
    args = parser.parse_args()
    cumulative_results = []
    current_results = []
    with ThreadPoolExecutor(
        max_workers=os.cpu_count() if args.parallelize else 1
    ) as executor:
        res = executor.map(process_log, glob.glob("out/*/stats.json"))
        for cum, current in res:
            cumulative_results.append(cum)
            current_results.append(current)

    if cumulative_results:
        process_cumulative_results(cumulative_results)
    if current_results:
        process_current_results(current_results)
