use std::thread;
use std::sync::mpsc::channel;
use std::sync::Arc;

extern crate wavfile;
use wavfile::*;

extern crate resonance;
use resonance::*;

mod error;
use error::*;

mod timeline;
use timeline::*;

mod terminal_diplay;
use terminal_diplay::*;

mod keyhit_input;
use keyhit_input::*;

#[cfg(test)]
mod tests {
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Track {
    pub file_path: std::path::PathBuf,
    pub format_id: usize,
    pub channel: usize,
    pub sampling_rate: usize,
    pub bits: usize,
    pub ch_vec: Vec<Vec<f64>>,
}



#[allow(dead_code)]
#[derive(Clone)]
pub struct DisplaySpectrum {
    pub span: usize,
    pub labels:&'static[&'static str],
    pub ch_vec: Vec<Vec<Vec<f64>>>, // channel<freq<data<energy>>>
}

#[derive(Clone)]
#[derive(PartialEq)]
pub enum ThreadID {
    TimeCounter,
    Display,
    KeyHit,
}

pub struct AppEvent {
    pub thread_id: ThreadID,
    pub event_id: usize,
}

#[allow(dead_code)]
fn set_gain(ref_data_vec:&mut Vec<i64>, db:&f64) {
    let gain = 10_f64.powf(db / 10_f64);
    for ref_data in ref_data_vec {
        *ref_data = ((*ref_data as f64) * gain) as i64;
    }
}

fn wav_to_track( wav_path: &std::path::Path) -> Result<Track> {
    let mut base_file = WavFile::new();
    base_file.open(wav_path)?;
    let base_wav_audio = base_file.get_wav_audio()?;
    Ok(Track{
        file_path: wav_path.to_path_buf(),
        format_id : base_wav_audio.fmt.id,
        channel : base_wav_audio.fmt.channel,
        sampling_rate : base_wav_audio.fmt.sampling_rate,
        bits : base_wav_audio.fmt.bits,
        ch_vec : to_channel_vec(&base_wav_audio)?
    })
}

fn resonance_parrot() -> Result<()> {
    let base_track = wav_to_track(std::path::Path::new(r"./test.wav"))?;

    //  Time Measurement
    //let mut resonance_spectrum = ResonanceSpectrum::new(440.0, base_track.ch_vec.len(), base_track.sampling_rate)?;
    //let now = time::Instant::now();
    //for (ch_idx, ch) in base_track.ch_vec.iter().enumerate() {
    //    resonance_spectrum.resonance(ch_idx, &ch[0..442])?;
    //}
    //println!("Dur: {}",now.elapsed().as_millis());

    let (event_sender, event_receiver) = channel::<AppEvent>();

    let time_event = event_sender.clone();
    let (to_timeline_sender, to_timeline_receiver) = channel::<TimelineRequest>();
    let (from_timeline_sender, from_timeline_receiver) = channel::<TimelineReport>();
    let timeline_thread_instanse = thread::spawn(move || 
        timeline_thread(time_event, from_timeline_sender, to_timeline_receiver)
    );

    let (to_display_sender, to_display_receiver) = channel::<DisplayRequest>();
    let display_thread_instanse = thread::spawn(move || 
        display_thread(to_display_receiver)
    );

    let key_event = event_sender.clone();
    let (to_key_sender, to_key_receiver) = channel::<KeyHitRequest>();
    let (from_key_sender, from_key_receiver) = channel::<char>();
    let keyhit_thread_instanse = thread::spawn(move || 
        keyhit_thread(key_event, from_key_sender, to_key_receiver)
    );

    to_display_sender.send(DisplayRequest::open(base_track.file_path.to_string_lossy().to_string(),base_track.sampling_rate,base_track.bits,base_track.ch_vec.len())?)?;
    to_timeline_sender.send(TimelineRequest::open(base_track.ch_vec[0].len(), base_track.sampling_rate, base_track.sampling_rate/100))?;
    let mut resonance_spectrum = ResonanceSpectrum::new(440.0, base_track.ch_vec.len(), base_track.sampling_rate, 6)?;

    loop {
        let event = event_receiver.recv()?;
        match event.thread_id {
            ThreadID::TimeCounter => {
                let timeline_report = from_timeline_receiver.recv()?;
                
                // Temporary Process
                let mut sound_vec:Vec<Vec<f64>> = Vec::with_capacity(base_track.ch_vec.len());
                let mut resonance_vec:Vec<Vec<Vec<f64>>> = Vec::with_capacity(base_track.ch_vec.len());
                for (ch_idx, ch) in base_track.ch_vec.iter().enumerate() {
                    let data_stt = timeline_report.timeline.time_counter;
                    let data_end;
                    if timeline_report.timeline.time_counter + timeline_report.timeline.base.event_divisor < ch.len() {
                        data_end = timeline_report.timeline.time_counter + timeline_report.timeline.base.event_divisor;
                    }
                    else{
                        data_end = ch.len()
                    }
                    resonance_vec.push(resonance_spectrum.resonance(ch_idx, &ch[data_stt..data_end])?);
                    sound_vec.push(ch[data_stt..data_end].to_vec());
                }
                let sound_arc = Arc::new(sound_vec);
                let spectrum_arc = Arc::new(resonance_vec);
                to_display_sender.send(DisplayRequest::update_value(timeline_report.timeline.time_counter, sound_arc, spectrum_arc))?;
            },
            ThreadID::KeyHit => {
                let input_char = from_key_receiver.recv()?;
                if input_char == '\x1B' || input_char == 'q' || input_char == 'Q' {
                    break;
                }
                if input_char == 'w' || input_char == 'W' {
                    to_timeline_sender.send(TimelineRequest::play_or_pause())?;
                }
                if input_char == 's' || input_char == 'S' {
                    to_timeline_sender.send(TimelineRequest::stop())?;
                }
                if input_char == 'd' || input_char == 'D' {
                    // Fast Forword
                }
                if input_char == 'a' || input_char == 'A' {
                    // Rewind
                }
                if input_char == 'o' || input_char == 'O' {
                    // File Open
                }
                if input_char == 'm' || input_char == 'M' {
                    // Menu
                }
                if input_char == 'e' || input_char == 'E' {
                    // Shift Range High
                    to_display_sender.send(DisplayRequest::change_rel_range(12))?;
                }
                if input_char == 'c' || input_char == 'C' {
                    // Shift Range Low
                    to_display_sender.send(DisplayRequest::change_rel_range(-12))?;
                }
                to_key_sender.send(KeyHitRequest::Continue)?;
            },
            _ => { /*None*/ }
        }
    }
    print!("\n\n\n");
    print!("Display Thread Close....");
    to_display_sender.send(DisplayRequest::exit())?;
    match  display_thread_instanse.join() {
        Ok(_ret) => {
            println!("Ok!");
        }
        Err(_err) => {
            println!("Error!");
        }
    }
    print!("Timeline Thread Close....");
    to_timeline_sender.send(TimelineRequest::close())?;
    match  timeline_thread_instanse.join() {
        Ok(_ret) => {
            println!("Ok!");
        }
        Err(_err) => {
            println!("Error!");
        }
    }
    print!("Keyhit Thread Close....");
    to_key_sender.send(KeyHitRequest::Exit)?;
    match  keyhit_thread_instanse.join() {
        Ok(_ret) => {
            println!("Ok!");
        }
        Err(_err) => {
            println!("Error!");
        }
    }

    let wav_audio_fmt = Fmt {
        id:base_track.format_id,
        channel: base_track.channel,
        sampling_rate: base_track.sampling_rate,
        bits: base_track.bits,
    };
    let wav_audio = to_wav_audio(&base_track.ch_vec, &wav_audio_fmt)?;
    let mut new_file = WavFile::new();
    new_file.update_wav_audio(&wav_audio)?;
    new_file.save_as(std::path::Path::new(r"./new.wav"))?;
    Ok(())
}

fn main() {
    match resonance_parrot() {
        Ok(_) => {},
        Err(err) => println!("Error!!! {}",err)
    }
}
