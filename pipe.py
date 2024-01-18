#!/usr/bin/env python3

import random
import time
import numpy as np
import sys

NUM_SIGNALS = 5
RATE = 50


def header(num_signals) -> list[str]:
    return ["Time"] + [f"signal{i}" for i in range(1, num_signals + 1)]


def gen_data(num_signals) -> list[float]:
    period = 1 / RATE

    locs = np.array([(random.random() - 0.5) * 40 for _ in range(0, num_signals)])
    scales = np.array([random.random() * 5/3 for _ in range(0, num_signals)])

    start = time.time()

    while True:
        yield [time.time() - start] + np.random.normal(loc=locs, scale=scales).tolist()
        time.sleep(period)


def main():
    flush = not sys.stdout.isatty()

    # print(",".join(header(NUM_SIGNALS+1)), flush=flush)
    # print(",".join([str(d) for d in next(gen_data(NUM_SIGNALS-1))]), flush=flush)
    # print("Random stuff")

    # Correct data
    print(",".join(header(NUM_SIGNALS)), flush=flush)
    for data in gen_data(NUM_SIGNALS):
        print(",".join([str(d) for d in data]), flush=flush)



if __name__ == "__main__":
    main()