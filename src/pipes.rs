use alsa::pcm::*;
use alsa::{Direction, Error};

pub struct Pipe {
    pub num_ch: u16,
    pub fft_size: u16,
    pub input: Vec<f32>,
    // pub input_l: Vec<f32>,
    // pub input_r: Vec<f32>,
    pub output: Vec<f32>,

    pub pcm: PCM,
    // io: IO<f32>,
}

impl Pipe {
    pub fn fill_input_buffer(&mut self) -> Result<(), Error> {
        let io = self.pcm.io_f32()?;

        loop {
            if io.readi(&mut self.input)? == self.input.len() / self.num_ch as usize {
                // println!("breaking");
                break;
            }
        }

        Ok(())
    }

    pub fn get_highest_output_index(&self) -> Result<usize, Error> {
        let highest_index: usize = self
            .output
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(index, _)| index)
            .unwrap_or(0);

        Ok(highest_index)
    }
}

pub fn get_alsa_pcm(device: &str, sample_rate: u32, num_ch: u16) -> Result<PCM, Error> {
    let pcm = PCM::new(device, Direction::Capture, false)?;
    {
        let hwp = HwParams::any(&pcm)?;
        hwp.set_channels(num_ch.into())?;
        hwp.set_rate(sample_rate, alsa::ValueOr::Nearest)?;
        hwp.set_format(Format::float())?;
        hwp.set_access(Access::RWInterleaved)?;
        // hwp.set_buffer_size(4500)?;
        pcm.hw_params(&hwp)?;
    }

    pcm.start()?;
    Ok(pcm)
}
