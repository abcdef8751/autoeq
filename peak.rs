fn sin(x: f64) -> f64 {
    x.sin()
}
fn cos(x: f64) -> f64 {
    x.cos()
}

#[derive(Clone, Debug)]
pub struct Filter {
    pub center: f64,
    pub Q: f64,
    pub gain: f64,
    pub sampleRate: f64,
}
impl Filter {
    pub fn new(center: f64, Q: f64, gain: f64) -> Self {
        Filter {
            sampleRate: 44100.0,
            center,
            Q,
            gain,
        }
    }
}

pub fn peak(
    freq: f64,
    &Filter {
        Q,
        center,
        gain,
        sampleRate,
    }: &Filter,
) -> f64 {
    let pi = std::f64::consts::PI;
    let A = 10f64.powf(gain / 40.0);
    let omega0 = 2.0 * pi * center / sampleRate;
    let omega = 2.0 * pi * freq / sampleRate;
    let alpha = sin(omega0) / (2.0 * Q);

    let B0 = 1.0 + alpha * A;
    let B1 = -2.0 * cos(omega0);
    let B2 = 1.0 - alpha * A;
    let A0 = 1.0 + alpha / A;
    let A1 = -2.0 * cos(omega0);
    let A2 = 1.0 - alpha / A;

    let cosOmega = cos(omega);
    let sinOmega = sin(omega);
    let cos2Omega = cos(2.0 * omega);

    let sin2Omega = sin(2.0 * omega);

    let numReal = B0 + B1 * cosOmega + B2 * cos2Omega;
    let numImag = -B1 * sinOmega - B2 * sin2Omega;
    let denReal = A0 + A1 * cosOmega + A2 * cos2Omega;
    let denImag = -A1 * sinOmega - A2 * sin2Omega;

    let numMagnitude = (numReal * numReal + numImag * numImag).sqrt();
    let denMagnitude = (denReal * denReal + denImag * denImag).sqrt();

    let H = numMagnitude / denMagnitude;
    return 20.0 * H.log10();
}
