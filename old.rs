mod peak;
use peak::{Filter, peak};
use rand::{prelude::*, rng};
use std::time::Instant;
use std::{fs, path, thread};
#[derive(Clone, Copy, Debug)]
struct Point {
    freq: f64,
    gain: f64,
}

fn lerp(Point { freq: x1, gain: y1 }: Point, Point { freq: x2, gain: y2 }: Point, x: f64) -> f64 {
    (y1 - y2) / (x1 - x2) * x + (x1 * y2 - x2 * y1) / (x1 - x2)
}

const lower_freq: f64 = 20.0;
const upper_freq: f64 = 18000.0;

const min_q: f64 = 0.1;

const max_q: f64 = 5.0;

const min_gain: f64 = -100.0;
const max_gain: f64 = 100.0;

const thread_count: usize = 6;
const iters: usize = 1000;
const filter_count: usize = 20;

const optimisation_rounds: usize = 1;

const input_file: &str = "./measurements/Galaxy Buds2 Pro (ANC on) R (CRIN).txt";
const output_file: &str = "./measurements/ThieAudio Monarch MKII L.txt";
fn parse_measurement_file(file_path: &str) -> Vec<Point> {
    let file = fs::read_to_string(file_path).unwrap();
    file.lines()
        .map(|x| {
            let binding = x.replace(",", " ");
            let split = binding.split_whitespace().collect::<Vec<_>>();
            let invalid_point = Point {
                freq: -1.0,
                gain: -1.0,
            };

            if split.len() < 2 {
                return invalid_point;
            }
            match [split[0].parse(), split[1].parse()] {
                [Ok(freq), Ok(gain)] => Point { freq, gain },
                _ => invalid_point,
            }
        })
        .filter(|x| x.freq > 0.0)
        .collect()
}
fn closest_points(data: &Vec<Point>, freq: f64) -> Vec<Point> {
    let mut sorted = data.clone();
    sorted.sort_by(|x, y| (x.freq - freq).abs().total_cmp(&(y.freq - freq).abs()));
    sorted
}
fn normalise(data: Vec<Point>) -> Vec<Point> {
    let norm_freq: f64 = 400.0;
    let norm_pt = closest_points(&data, norm_freq)[0];
    data.iter()
        .map(|x| Point {
            freq: x.freq,
            gain: x.gain - norm_pt.gain,
        })
        .collect()
}
fn error(filters: &Vec<Filter>, dif: &Vec<Point>) -> f64 {
    let mut res = 0f64;
    for point in dif {
        let mut s = 0f64;
        for filter in filters {
            s += peak(point.freq, filter);
        }
        res += (s - point.gain).abs();
    }
    res
}

fn bound(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}
fn optimise(
    starting_filters: &Vec<Filter>,
    iter_count: usize,
    deviation: &Vec<f64>,
    measurement_dif: &Vec<Point>,
) -> Vec<Filter> {
    let mut rng = rand::rng();
    let mut random = |maximum: f64| -> f64 { (rng.random::<f64>() * 2f64 - 1f64) * maximum };
    let mut filters = starting_filters.clone();
    let mut last_error = error(&filters, &measurement_dif);
    for iter in 0..iter_count {
        let temp = 1.0 - (iter as f64 + 1.0) / iter_count as f64;
        let mut index = 0usize;
        for filter in filters.clone() {
            let mutated_filter = Filter::new(
                bound(
                    2f64.powf((filter.center).log2() + random(deviation[0])),
                    lower_freq,
                    upper_freq,
                ),
                bound(filter.Q + random(deviation[1]), min_q, max_q),
                bound(filter.gain + random(deviation[2]), min_gain, max_gain),
            );
            for param in 0..=2usize {
                match param {
                    0 => filters[index].center = mutated_filter.center,
                    1 => filters[index].Q = mutated_filter.Q,
                    2 => filters[index].gain = mutated_filter.gain,
                    _ => (),
                };
                let new_error = error(&filters, &measurement_dif);
                let mut prob = ((-new_error + last_error) / temp).exp();
                if prob.is_nan() {
                    prob = 0f64;
                }
                if (last_error > new_error) || ((random(0.5f64) + 0.5f64) < prob) {
                    last_error = new_error;
                } else {
                    filters[index] = filter.clone();
                }
            }
            index += 1;
        }
    }
    filters
}
fn opt(
    starting_filters: Vec<Filter>,
    iter_count: usize,
    deviation: Vec<f64>,
    measurement_dif: Vec<Point>,
) -> Vec<Filter> {
    optimise(&starting_filters, iter_count, &deviation, &measurement_dif)
}

fn main() {
    let raw_input_data = parse_measurement_file(input_file);
    let raw_output_data = parse_measurement_file(output_file);
    let input_data = normalise(raw_input_data);
    let output_data = normalise(raw_output_data);
    let mut measurement_dif: Vec<Point> = Vec::new();
    let mut freq = lower_freq;
    while freq <= upper_freq {
        let closest_in_input = closest_points(&input_data, freq);
        let closest_in_output = closest_points(&output_data, freq);
        measurement_dif.push(Point {
            freq,
            gain: -lerp(closest_in_input[0], closest_in_input[1], freq)
                + lerp(closest_in_output[0], closest_in_output[1], freq),
        });
        freq *= if freq < 2000.0 { 1.1 } else { 1.05 };
        freq = freq.ceil();
    }
    println!("{:?}", measurement_dif);
    let mut deviation = vec![1.0f64, 1.0, 1.0];
    let mut best_filters = vec![Filter::new(2000f64, 1f64, 1f64); filter_count];
    let min_error = 0.2f64 * measurement_dif.len() as f64;
    for i in 0..optimisation_rounds {
        println!("round: {}/{}", i + 1, optimisation_rounds);
        let mut handles = vec![];
        for _ in 0..thread_count {
            let cloned = best_filters.clone();
            let clonned = deviation.clone();
            let clonnned = measurement_dif.clone();
            handles.push(thread::spawn(|| opt(cloned, iters, clonned, clonnned)));
        }
        for handle in handles {
            let filters = handle.join().unwrap();
            if error(&filters, &measurement_dif) < error(&best_filters, &measurement_dif) {
                best_filters = filters;
            }
        }
        if error(&best_filters, &measurement_dif) <= min_error {
            break;
        }
        deviation = deviation.iter().map(|x| x * 0.8).collect();
    }
    println!("{:#?}", best_filters);
    println!(
        "error: {}, min error: {}",
        error(&best_filters, &measurement_dif),
        min_error
    );
    best_filters.sort_by(|x, y| x.center.total_cmp(&y.center));
    let preamp = (-input_data
        .iter()
        .map(|x| best_filters.iter().fold(0f64, |a, f| a + peak(x.freq, f)))
        .max_by(|&x, y| x.total_cmp(y))
        .unwrap())
    .min(0f64);
    let eq_file_content = format!(
        "Preamp: {} dB\n{}",
        preamp,
        best_filters
            .iter()
            .enumerate()
            .map(|(i, x)| format!(
                "Filter {}: ON PK Fc {} Hz Gain {} dB Q {}",
                i + 1,
                x.center,
                x.gain,
                x.Q
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
    println!("{}", eq_file_content);
    fs::write(
        format!(
            "./eq/eq_{}_{}.txt",
            path::Path::new(output_file)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            path::Path::new(input_file)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
        ),
        eq_file_content,
    )
    .unwrap();
    fs::write(
        "./test.txt",
        input_data
            .iter()
            .map(|x| {
                format!(
                    "{} {}",
                    x.freq,
                    x.gain
                        + best_filters
                            .iter()
                            .fold(0f64, |a, filter| a + peak(x.freq, filter))
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
}
