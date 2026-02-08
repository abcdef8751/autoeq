# autoeq
Finds peaking filters to EQ one frequency response to match another.

Usage: ```cargo r```

main.rs parameters:
  - input_file/output_file: paths for frequency response files provided by squig.link sites
  - iters: simulated annealing iterations
  - filter_count: number of filters to use in the EQ
  - optimisation_threads_count: amount of threads to spawn each iteration (best result picked)
  - optimisation_rounds: how many times the algorithm is run (each time uses the best result from the last round)
  - optimisation_threads_count: amount of threads to run the algorithm (best result picked)
  - min_gain/max_gain: gain limits in dB
  - min_q/max_q: quality factor limits
  - lower_freq/upper_freq: frequency limits in Hz

Resulting files are in APO format and are written to ./eq
The resulting frequency response is written to ./test.txt
