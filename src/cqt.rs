use alsa::nix::Error;
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct Complex {
    pub real: f32,
    pub imag: f32,
}

impl Default for Complex {
    fn default() -> Self {
        Self { real: 0.0, imag: 0.0 }
    }
}

#[derive(Debug, Clone)]
pub struct TimeKernel {
    pub signal: Vec<Complex>,
    pub len: u16,
    pub start: u16,
}

fn hamming_window(N: f32, n: f32) -> f32 {
    // hamming(len)= 0.46-0.54*cos(2*pi*(0:len-1)’/len)
    static PI_2: f32 = 2.0 * PI;
    0.46 - 0.54 * (PI_2 * n / N).cos()
}

pub fn init_time_domain_kernel(fs: u32, fft_size: u16, f_min: f32, bins_per_octave: u16, n_bins: u16) -> Result<Vec<TimeKernel>, Error> {
    let q: f32 = 1.0 / (f32::powf(2.0, 1.0 / bins_per_octave as f32) - 1.0);
    let alpha: f32 = f32::powf(2.0, 1.0 / bins_per_octave as f32) - 1.0;

    let mut time_domain_kernels: Vec<TimeKernel> = vec![{ TimeKernel {
        signal: vec![Complex::default(); fft_size.into()],
        len: fft_size,
        start: 0,
    }}; n_bins.into()];

    // for (int k = 0; k < n_bins; k++) {
    for k in 0..n_bins {
        let fk: f32 = f_min * f32::powf(2.0, k as f32 / bins_per_octave as f32);
        let mut len = (q * fs as f32 / (fk + 0.0 / alpha)).ceil() as u16;
        let start;

        if len > fft_size {
            len = fft_size;
        }

        // Center win len
        if len % 2 == 1 {
            start = ((fft_size as f32 / 2.0 - len as f32 / 2.0).ceil() - 1.0) as u16;
        } else {
            start = ((fft_size as f32 / 2.0 - len as f32 / 2.0).ceil()) as u16;
        }

        let mut s: i16 = 0 - len as i16 / 2;

        let end = start + len;
        let time_kernel = &mut time_domain_kernels[k as usize].signal;
        for i in 0..fft_size {
            if i <= start || i >= end {
                continue;
            }

            let W: f32 = 2.0 * PI * s as f32  * fk / fs as f32;
            let win: f32 = hamming_window(len as f32, i as f32);

            time_kernel[i as usize] = { Complex {
                real: win * W.cos() / len as f32,
                imag: win * W.sin() / len as f32
            }};
            s += 1;
        }

        time_domain_kernels[k as usize].start = start;
        time_domain_kernels[k as usize].len = len;
    }
    Ok(time_domain_kernels)
}


pub fn calc_cqt(input: &[f32], time_kernels: &[TimeKernel], n_bins: u16) -> Result<Vec<f32>, Box<Error>> {
    let mut output: Vec<f32> = vec![0f32; n_bins as usize];
    // assert_eq!(output.len() as u16, n_bins);

    // loop each cqt bin
    for k in 0..n_bins as usize {
        let time_kernel = &time_kernels[k];
        let mut sum: Complex = Complex::default();

        // loop sample through start until end 
        for n in time_kernel.start as usize..(time_kernel.start + time_kernel.len) as usize {
            // multiply sample with kernel 
            sum.real += input[n] * time_kernel.signal[n].real;
            sum.imag -= input[n] * time_kernel.signal[n].imag;
        }
        output[k] = (sum.real.powf(2.0) + sum.imag.powf(2.0)).sqrt();
    }
    Ok(output)
}
