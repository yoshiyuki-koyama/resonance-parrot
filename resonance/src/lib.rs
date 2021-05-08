mod error;
use error::*;

use std::convert::TryFrom;
use std::f64::consts::PI;
use std::rc::Rc;

use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;



const LOWEST_PITCH_IDX: isize = -48; // A0 to A8
const HIGHEST_PITCH_IDX: isize = 48;
pub const SPN_NUM: usize = 97; // HIGHEST_PITCH_IDX - LOWEST_PITCH_IDX + 1



//pub const SPN_LABEL: [&str; SPN_NUM] = [ //Scientific Pitch Notation
//    "A0","A♯/B♭0","B0","C1","C♯/D♭1","D1","D♯/E♭1","E1","F1","F♯/G♭1","G1","G♯/A♭1",
//    "A1","A♯/B♭1","B1","C2","C♯/D♭2","D2","D♯/E♭2","E2","F2","F♯/G♭2","G2","G♯/A♭2",
//    "A2","A♯/B♭2","B2","C3","C♯/D♭3","D3","D♯/E♭3","E3","F3","F♯/G♭3","G3","G♯/A♭3",
//    "A3","A♯/B♭3","B3","C4","C♯/D♭4","D4","D♯/E♭4","E4","F4","F♯/G♭4","G4","G♯/A♭4",
//    "A4","A♯/B♭4","B4","C5","C♯/D♭5","D5","D♯/E♭5","E5","F5","F♯/G♭5","G5","G♯/A♭5",
//    "A5","A♯/B♭5","B5","C6","C♯/D♭6","D6","D♯/E♭6","E6","F6","F♯/G♭6","G6","G♯/A♭6",
//    "A6","A♯/B♭6","B6","C7","C♯/D♭7","D7","D♯/E♭7","E7","F7","F♯/G♭7","G7","G♯/A♭7",
//    "A7","A♯/B♭7","B7","C8","C♯/D♭8","D8","D♯/E♭8","E8","F8","F♯/G♭8","G8","G♯/A♭8",
//    "A8",
//];

pub const SPN_LABEL: [&str; SPN_NUM] = [ //Scientific Pitch Notation
    "A0","A#/Bb0","B0","C1","C#/Db1","D1","D#/Eb1","E1","F1","F#/Gb1","G1","G#/Ab1",
    "A1","A#/Bb1","B1","C2","C#/Db2","D2","D#/Eb2","E2","F2","F#/Gb2","G2","G#/Ab2",
    "A2","A#/Bb2","B2","C3","C#/Db3","D3","D#/Eb3","E3","F3","F#/Gb3","G3","G#/Ab3",
    "A3","A#/Bb3","B3","C4","C#/Db4","D4","D#/Eb4","E4","F4","F#/Gb4","G4","G#/Ab4",
    "A4","A#/Bb4","B4","C5","C#/Db5","D5","D#/Eb5","E5","F5","F#/Gb5","G5","G#/Ab5",
    "A5","A#/Bb5","B5","C6","C#/Db6","D6","D#/Eb6","E6","F6","F#/Gb6","G6","G#/Ab6",
    "A6","A#/Bb6","B6","C7","C#/Db7","D7","D#/Eb7","E7","F7","F#/Gb7","G7","G#/Ab7",
    "A7","A#/Bb7","B7","C8","C#/Db8","D8","D#/Eb8","E8","F8","F#/Gb8","G8","G#/Ab8",
    "A8",
];

#[repr(usize)]
pub enum SpnIdx { // Scientific Pitch Notation
    A0,AsBf0,B0,C1,CsDf1,D1,DsEf1,E1,F1,FsGf1,G1,GsAf1,
    A1,AsBf1,B1,C2,CsDf2,D2,DsEf2,E2,F2,FsGf2,G2,GsAf2,
    A2,AsBf2,B2,C3,CsDf3,D3,DsEf3,E3,F3,FsGf3,G3,GsAf3,
    A3,AsBf3,B3,C4,CsDf4,D4,DsEf4,E4,F4,FsGf4,G4,GsAf4,
    A4,AsBf4,B4,C5,CsDf5,D5,DsEf5,E5,F5,FsGf5,G5,GsAf5,
    A5,AsBf5,B5,C6,CsDf6,D6,DsEf6,E6,F6,FsGf6,G6,GsAf6,
    A6,AsBf6,B6,C7,CsDf7,D7,DsEf7,E7,F7,FsGf7,G7,GsAf7,
    A7,AsBf7,B7,C8,CsDf8,D8,DsEf8,E8,F8,FsGf8,G8,GsAf8,
    A8,
}


#[cfg(test)]
mod tests {
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct SpringStatus {
    speed:f64,
    position:f64,
}

#[derive(Clone)]
#[derive(PartialEq)]
enum ResonanceRequestType {
    Calc,
    Exit,
}

pub struct ResonanceRequest {
    request_type: ResonanceRequestType,
    sound_data_arc: Option<Arc<Vec<Vec<f64>>>>,
}

pub struct ResonanceReport {
    ch_idx: usize,
    energy_spring_vec: Vec<Vec<f64>>,
}


#[allow(dead_code)]
#[derive(Clone)]
pub struct SplitResonance {
    spring_constant_vec: Vec<f64>,
    data_period: f64,
    ch_idx: usize,
    pub spring_sts_vec: Vec<SpringStatus>,  // channel<spring<SpringStatus>>
}

impl SplitResonance {
    pub fn new(spring_constant_vec: Vec<f64>, data_period: f64, ch_idx: usize) -> Result<SplitResonance>  {
        let mut spring_sts_vec: Vec<SpringStatus> = Vec::new();
        for _ in 0..spring_constant_vec.len(){
            let spring_sts = SpringStatus{
                speed:0.0,
                position:0.0,
            };
            spring_sts_vec.push(spring_sts);
        }

        Ok(SplitResonance {
            spring_constant_vec: spring_constant_vec,
            data_period : data_period,
            ch_idx: ch_idx,
            spring_sts_vec: spring_sts_vec,
        }) 
    }

    pub fn split_resonance(&mut self, sound_data_arc: Arc<Vec<Vec<f64>>>)  -> Result<ResonanceReport> {
        let mut energy_max = 0.0;
        let mut ret_energy_spring_vec: Vec<Vec<f64>> = Vec::with_capacity(self.spring_constant_vec.len());
        for (spring_idx, spring_constant) in self.spring_constant_vec.iter().enumerate() {
            let spring_sts = &mut self.spring_sts_vec[spring_idx];
            let mut ret_energy_vec :Vec<f64> = Vec::with_capacity(sound_data_arc.len());
            for data in sound_data_arc[self.ch_idx].iter() {
                spring_sts.speed = (*data - spring_constant * spring_sts.position - spring_sts.speed * 100.0)*self.data_period + spring_sts.speed;
                spring_sts.position = spring_sts.speed*self.data_period + spring_sts.position;
                let route_energy = (0.5*spring_sts.speed.powi(2) + 0.5*spring_constant*spring_sts.position.powi(2)).powf(0.5);
                ret_energy_vec.push(route_energy);
                if energy_max < route_energy {
                    energy_max = route_energy;
                }
            }
            ret_energy_spring_vec.push(ret_energy_vec);
        }

        Ok( ResonanceReport{
            ch_idx: self.ch_idx,
            energy_spring_vec: ret_energy_spring_vec,
        })
    }
}
#[allow(dead_code)]
#[derive(Clone)]
pub struct Resonance {
    ch_num:usize,
    thread_per_ch: usize,
    thread_vec: Rc<Vec<thread::JoinHandle<Result<()>>>>,
    to_resonance_sender_vec: Rc<Vec<Sender<ResonanceRequest>>>,
    from_resonance_receiver_vec: Rc<Vec<Receiver<ResonanceReport>>>,
}

fn resonance_thread_main( from_resonanance_sender: Sender<ResonanceReport>, to_resonance_receiver: Receiver<ResonanceRequest>,
    split_spring_vec: Vec<f64>, data_period: f64, ch_idx: usize) -> Result<()> {
    
    let mut split_resonance = SplitResonance::new(split_spring_vec, data_period, ch_idx)?;

    loop {
        let resonance_request = to_resonance_receiver.recv()?;
        match resonance_request.request_type {
            ResonanceRequestType::Calc => {
                if let Some(sound_data_arc) = resonance_request.sound_data_arc {
                    from_resonanance_sender.send(split_resonance.split_resonance(sound_data_arc)?)?;
                }
                else {
                    return Err(ResonanceError::new("Calc Resonance must be with Sound Data!"));
                }
            }
            ResonanceRequestType::Exit => {
                break;
            }
        }
    }
    Ok(())
}

fn resonance_thread( from_resonanance_sender: Sender<ResonanceReport>, to_resonance_receiver: Receiver<ResonanceRequest>,
    split_spring_vec: Vec<f64>, data_period: f64, ch_idx: usize) -> Result<()> {
    match resonance_thread_main( from_resonanance_sender, to_resonance_receiver, split_spring_vec, data_period, ch_idx) {
        Ok(_ret) => { /* Nothing to do */ }
        Err(err) => {
            println!("Error! resonance_thread!");
            return Err(err);
        }
    }
    Ok(())
}

impl Resonance {
    pub fn new(pitch_standard_frequency: f64, data_frequency: usize, ch_num:usize, thread_per_ch: usize) -> Result<Resonance>  {
        if thread_per_ch == 0 {
            return Err(ResonanceError::new("thread_per_ch must not be 0!"));
        }

        let mut to_resonance_sender_vec: Vec<Sender<ResonanceRequest>> = Vec::new();
        let mut from_resonance_receiver_vec: Vec<Receiver<ResonanceReport>> = Vec::new();
        let mut resonance_thread_instanse_vec: Vec<thread::JoinHandle<Result<()>>> = Vec::new();

        let mut spring_constant_vec: Vec<f64> = Vec::new();
        for i in LOWEST_PITCH_IDX..HIGHEST_PITCH_IDX+1 {
            let hz = pitch_standard_frequency*2.0_f64.powf(f64::from(i32::try_from(i)?)/12.0);
            spring_constant_vec.push((hz*2.0*PI).powi(2));
        }

        // split by thread
        let split_pitch_range;
        if SPN_NUM % thread_per_ch == 0 {
            split_pitch_range = SPN_NUM / thread_per_ch;
        }
        else{
            split_pitch_range =SPN_NUM / thread_per_ch + 1;
        }

        let data_period = 1.0/f64::from(u32::try_from(data_frequency)?);

        for ch_idx in 0..ch_num {
            for split_idx in 0..thread_per_ch {
                let split_spring_vec;
                if split_idx < thread_per_ch - 1 {
                    split_spring_vec = spring_constant_vec[split_pitch_range*split_idx..split_pitch_range*(split_idx+1)].to_vec();
                }
                else{ 
                    split_spring_vec = spring_constant_vec[split_pitch_range*split_idx..].to_vec();
                }

                let (to_resonance_sender, to_resonance_receiver) = channel::<ResonanceRequest>(); // data
                let (from_resonance_sender, from_resonance_receiver) = channel::<ResonanceReport>(); // spring<data<energy>>
                let resonance_thread_instanse = thread::spawn(move || 
                    resonance_thread(from_resonance_sender, to_resonance_receiver, split_spring_vec, data_period, ch_idx)
                );
                to_resonance_sender_vec.push(to_resonance_sender);
                from_resonance_receiver_vec.push(from_resonance_receiver);
                resonance_thread_instanse_vec.push(resonance_thread_instanse);
            }
        }
        Ok(Resonance {
            ch_num: ch_num,
            thread_per_ch: thread_per_ch,
            thread_vec: Rc::new(resonance_thread_instanse_vec),
            to_resonance_sender_vec: Rc::new(to_resonance_sender_vec),
            from_resonance_receiver_vec: Rc::new(from_resonance_receiver_vec)
        })
    }

    // Temporary Implementation
    pub fn resonance(&self, sound_data_arc: Arc<Vec<Vec<f64>>>) -> Result<Vec<Vec<Vec<f64>>>> { // channel<spring<data<energy>>>
      for sender in &*self.to_resonance_sender_vec {
        sender.send(ResonanceRequest {
                request_type: ResonanceRequestType::Calc,
                sound_data_arc: Some(sound_data_arc.clone()),
        })?;
      }
      
      let mut energy_spring_ch_vec:Vec<Vec<Vec<f64>>> = Vec::with_capacity(self.ch_num);
      for _ in 0..self.ch_num {
        energy_spring_ch_vec.push(Vec::new());
      }
      for receiver in &*self.from_resonance_receiver_vec {
        let mut resonance_report = receiver.recv()?;
        energy_spring_ch_vec[resonance_report.ch_idx].append(&mut resonance_report.energy_spring_vec);
      }
      Ok(energy_spring_ch_vec)
    }

    // manual exit
    pub fn exit(&mut self) -> Result<()> {
        let mut err_flg = false;
        for sender in &*self.to_resonance_sender_vec {
            match sender.send(ResonanceRequest {
                    request_type: ResonanceRequestType::Exit,
                    sound_data_arc: None,
            }) {
                Ok(_) => {
                    // Ok!
                }
                Err(_err) => {
                    println!("Send Exit Thread Request Error!");
                    err_flg = true;
                }
            }
        }

        if !self.thread_vec.is_empty() {
            print!("Split Resonance Thread Close....");
            if let Some(thread_vec) = Rc::get_mut(&mut self.thread_vec) {
                loop {
                    if let Some(thread) = thread_vec.pop() {
                        match thread.join() {
                            Ok(_ret) => {
                                // Ok!
                            }
                            Err(_err) => {
                                println!("Send Exit Thread Request Error!");
                                err_flg = true;
                            }
                        }
                    }
                    else {
                        if !err_flg {
                            println!("Ok!");
                        }
                        break;
                    }
                }
            }
            else {
                println!("Could not get thread_vec mutable!");
                err_flg = true;
            }
        }
        // Clear to_resonance_sender_vec (to ignore when Drop)
        if let Some(sender_vec) = Rc::get_mut(&mut self.to_resonance_sender_vec) {
            sender_vec.clear();
        }
        else {
            println!("Could not get sender_vec mutable!");
            err_flg = true;
        }
        if err_flg {
            return Err(ResonanceError::new("Could not get thread_vec mutable!"));
        }
        Ok(())
    }
}

// auto exit
impl Drop for Resonance {
    fn drop(&mut self) {
        for sender in &*self.to_resonance_sender_vec {
            match sender.send(ResonanceRequest {
                    request_type: ResonanceRequestType::Exit,
                    sound_data_arc: None,
            }) {
                Ok(_) => {
                    // Ok!
                }
                Err(_err) => {
                    println!("Error in Resonance Drop! Send Exit Thread Request Error!");
                }
            }
        }

        if !self.thread_vec.is_empty() {
            print!("Split Resonance Thread Close....");
            if let Some(thread_vec) = Rc::get_mut(&mut self.thread_vec) {
                loop {
                    if let Some(thread) = thread_vec.pop() {
                        match thread.join() {
                            Ok(_ret) => {
                                // Ok!
                            }
                            Err(_err) => {
                                println!("Error!");
                            }
                        }
                    }
                    else {
                        println!("Ok!");
                        break;
                    }
                }
            }
            else {
                println!("Error in Resonance Drop. Could not get thread_vec mutable!");
            }
        }
    }
}





