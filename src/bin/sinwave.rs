use std::env;
use std::mem::size_of;
use std::process::exit;
use std::fs::File;
use std::io::Write;
use std::f64::consts::PI;

type Sample = i16;
const HEADER_LEN: u32 = 44;
const SAMPLE_SIZE: u32 = size_of::<Sample>() as u32;

fn main() {
    let args: Vec<String> = env::args().collect();
    let argc = args.len();

    if argc != 2 {
        eprintln!("USAGE: cargo run --bin sinwave <output>");
        exit(-1);
    }

    let channel: u16 = 1;
    let fs_out: u32 = 16000;
    let len: u32 = 16000;
    let data_len: u32 = SAMPLE_SIZE * channel as u32 * len;
    let frequency: f64 = 1000.0;

    let file_size: u32 = HEADER_LEN + data_len - 8;
    let file_size = &file_size.to_le_bytes();

    let fmt_chnk = &(16 as u32).to_le_bytes();
    let fmt_id = &(1 as u16).to_le_bytes();
    let byte_per_sec: u32 = SAMPLE_SIZE * channel as u32 * fs_out;
    let byte_per_sample: u16 = SAMPLE_SIZE as u16 * channel;
    let bit_per_sample: u16 = SAMPLE_SIZE as u16 * 8;

    let filename = args[1].to_owned() + ".wav";
    let mut f = File::create(filename.as_str()).expect("ファイルの作成に失敗しました");

    f.write(b"RIFF").unwrap();
    f.write(file_size).unwrap();
    f.write(b"WAVEfmt ").unwrap();
    f.write(fmt_chnk).unwrap();
    f.write(fmt_id).unwrap();
    f.write(&channel.to_le_bytes()).unwrap();
    f.write(&fs_out.to_le_bytes()).unwrap();
    f.write(&byte_per_sec.to_le_bytes()).unwrap();
    f.write(&byte_per_sample.to_le_bytes()).unwrap();
    f.write(&bit_per_sample.to_le_bytes()).unwrap();
    f.write(b"data").unwrap();
    f.write(&data_len.to_le_bytes()).unwrap();

    println!("Output WAVE data is");
    println!("Channel = {} ch", channel);
    println!("Sample rate = {} Hz", fs_out);
    println!("Sample number = {}", data_len / channel as u32 / SAMPLE_SIZE);

    let mut t_out = 0;

    while t_out < len {
        let y: f64 = 0.5 * f64::sin(2.0 * PI * frequency / fs_out as f64 * t_out as f64);
        let output: Sample = (y * 32768.0) as i16;
        let output = &output.to_le_bytes();
        f.write(output).unwrap();
        t_out += 1;
    }
}
