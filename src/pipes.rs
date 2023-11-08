use std::fs::File;
use std::error::Error;
use std::io::{Read, BufReader};

use alsa::pcm::*;
use alsa::{Direction};
use byteorder::{ReadBytesExt, NativeEndian};

pub enum Source {
    Alsa,
    MpdFifo,
}

pub struct Pipe {
    pub num_ch: u16,
    pub fft_size: u16,
    pub input: Vec<f32>,
    // pub input_l: Vec<f32>,
    // pub input_r: Vec<f32>,
    pub output: Vec<f32>,
    pub input_pipe: Box<dyn InputPipe>,
}

pub trait InputPipe {
    fn pipe_read_internal(&mut self, input: &mut Vec<f32>) -> Result<(), &'static str>;
}

pub struct AlsaPipe {
    pub pcm: PCM,
}

impl InputPipe for AlsaPipe {
    fn pipe_read_internal(&mut self, input: &mut Vec<f32>) -> Result<(), &'static str> {
        let io = self.pcm.io_f32().unwrap();

        loop {
            if io.readi(input).unwrap() == input.len() as usize {
                break;
            }
        }

        Ok(())
    }
}

pub struct MPDFifoPipe {
    pub reader: BufReader<File>,
}

impl MPDFifoPipe {
    pub fn new(path: &str) -> MPDFifoPipe {
        MPDFifoPipe {
            reader: BufReader::new(File::open(path).unwrap()),
        }
    }
}

impl InputPipe for MPDFifoPipe {
    fn pipe_read_internal(&mut self, input: &mut Vec<f32>) -> Result<(), &'static str> {
        match self.reader.read_f32_into::<NativeEndian>(&mut input[..]) {
            Ok(_) => Ok(()),
            Err(_) => Err("Error reading fifo file.")
        }
    }
}

impl Pipe {
    pub fn fill_input_buffer(&mut self) {
        self.input_pipe.pipe_read_internal(&mut self.input).unwrap();
    }

    pub fn get_highest_output_index(&self) -> usize {
        let highest_index: usize = self
            .output
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(index, _)| index)
            .unwrap_or(0);

        highest_index
    }
}

pub fn get_alsa_pcm(device: &str, sample_rate: u32, num_ch: u16) -> Result<PCM, alsa::Error> {
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
