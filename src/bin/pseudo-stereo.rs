use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem::size_of;
use std::process::exit;

type Sample = i16;
const MEM_SIZE: usize = 16000;
const HEADER_LEN: u32 = 44;
const SAMPLE_SIZE: u32 = size_of::<Sample>() as u32;

fn main() {
    let args: Vec<String> = env::args().collect();
    let argc = args.len();

    if argc != 3 {
        eprintln!("USAGE: cargo run --bin pseudo-stereo <input> <output>");
        exit(-1);
    }

    let mut f1 = File::open(args[1].to_owned()).expect("ファイルの読み込みに失敗しました");
    let mut f2 = File::create(args[2].to_owned()).expect("ファイルの作成に失敗しました");

    let mut file_size = [0; 4];
    let mut fmt_chnk = [0; 4];
    let mut fmt_id = [0; 2];
    let mut ch_input = [0; 2];
    let mut sampling_rate = [0; 4];
    let mut byte_per_sec = [0; 4];
    let mut byte_per_block = [0; 2];
    let mut bit_per_sample = [0; 2];
    let mut data_len = [0; 4];

    f1.seek(SeekFrom::Start(4)).unwrap();
    f1.read_exact(&mut file_size).unwrap();
    f1.seek(SeekFrom::Current(8)).unwrap();
    f1.read_exact(&mut fmt_chnk).unwrap();
    f1.read_exact(&mut fmt_id).unwrap();
    f1.read_exact(&mut ch_input).unwrap();
    f1.read_exact(&mut sampling_rate).unwrap();
    f1.read_exact(&mut byte_per_sec).unwrap();
    f1.read_exact(&mut byte_per_block).unwrap();
    f1.read_exact(&mut bit_per_sample).unwrap();
    f1.seek(SeekFrom::Current(4)).unwrap();
    f1.read_exact(&mut data_len).unwrap();

    let ch_input = u16::from_le_bytes(ch_input);
    assert_eq!(ch_input, 1);
    let fs = u32::from_le_bytes(sampling_rate);
    let sample_size = u16::from_le_bytes(bit_per_sample);
    assert_eq!(sample_size as u32 / 8, SAMPLE_SIZE);
    let data_len = u32::from_le_bytes(data_len);
    let sample_num = data_len / SAMPLE_SIZE / ch_input as u32;

    println!("Input Wave data is");
    println!("Channel = {} ch", ch_input);
    println!("Sample number = {}", sample_num);

    let ch_output: u16 = 2;
    let fs_out: u32 = fs;
    let data_len: u32 = SAMPLE_SIZE * ch_output as u32 * sample_num;

    let byte_per_sec: u32 = SAMPLE_SIZE * ch_output as u32 * fs_out;
    let byte_per_sample: u16 = SAMPLE_SIZE as u16 * ch_output;
    let bit_per_sample: u16 = SAMPLE_SIZE as u16 * 8;

    let d = fs_out / 200;

    let mut s: [f64; MEM_SIZE] = [0.0; MEM_SIZE];
    let mut y_l: [f64; MEM_SIZE] = [0.0; MEM_SIZE];
    let mut y_r: [f64; MEM_SIZE] = [0.0; MEM_SIZE];

    f2.write(b"RIFF").unwrap();
    f2.write(&file_size).unwrap();
    f2.write(b"WAVEfmt ").unwrap();
    f2.write(&fmt_chnk).unwrap();
    f2.write(&fmt_id).unwrap();
    f2.write(&ch_output.to_le_bytes()).unwrap();
    f2.write(&fs_out.to_le_bytes()).unwrap();
    f2.write(&byte_per_sec.to_le_bytes()).unwrap();
    f2.write(&byte_per_sample.to_le_bytes()).unwrap();
    f2.write(&bit_per_sample.to_le_bytes()).unwrap();
    f2.write(b"data").unwrap();
    f2.write(&data_len.to_le_bytes()).unwrap();

    println!("Output WAVE data is");
    println!("Channel = {} ch", ch_output);
    println!("Sample rate = {} Hz", fs_out);
    println!(
        "Sample number = {}",
        data_len / ch_output as u32 / SAMPLE_SIZE
    );

    let mut t = 0;
    let mut t_out = 0;

    f1.seek(SeekFrom::Start(HEADER_LEN as u64)).unwrap();
    loop {
        let mut input = [0; 2];
        if let Err(_) = f1.read_exact(&mut input) {
            if t_out >= data_len {
                break;
            }
        }
        let input = Sample::from_le_bytes(input);
        s[t] = input as f64 / 32768.0;

        y_l[t] = s[t] + s[(t + MEM_SIZE - d as usize) % MEM_SIZE];
        y_r[t] = s[t] - s[(t + MEM_SIZE - d as usize) % MEM_SIZE];
        let output_l = (y_l[t] * 32768.0) as Sample;
        let output_r = (y_r[t] * 32768.0) as Sample;
        f2.write(&output_l.to_le_bytes()).unwrap();
        f2.write(&output_r.to_le_bytes()).unwrap();
        t = (t + 1) % MEM_SIZE;
        t_out += 1;
    }
}
