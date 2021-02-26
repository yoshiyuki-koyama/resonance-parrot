mod error;
use error::*;

use std::convert::TryFrom;
use std::f64::consts::PI;


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

#[allow(dead_code)]
#[derive(Clone)]
pub struct ResonanceSpectrum {
    spring_constant_vec: Vec<f64>,
    pub spring_sts_ch_vec: Vec<Vec<SpringStatus>>,  // channel<spring<SpringStatus>>
    pub energy_spring_ch_vec:Vec<Vec<Vec<f64>>>,  // channel<spring<data<energy>>>
    data_period: f64,
}

impl ResonanceSpectrum {
    pub fn new(pitch_standard_frequency: f64, ch_num:usize, data_frequency: usize) -> Result<ResonanceSpectrum>  {
        let mut spring_constant_vec: Vec<f64> = Vec::new();
        for i in LOWEST_PITCH_IDX..HIGHEST_PITCH_IDX+1 {
            let hz = pitch_standard_frequency*2.0_f64.powf(f64::from(i32::try_from(i)?)/12.0);
            spring_constant_vec.push((hz*2.0*PI).powi(2));
        }
        
        let mut spring_sts_ch_vec: Vec<Vec<SpringStatus>> = Vec::new();
        let mut energy_spring_ch_vec: Vec<Vec<Vec<f64>>> = Vec::new();
        for _ in 0..ch_num {
            let mut spring_sts_vec: Vec<SpringStatus> = Vec::new();
            let mut energy_spring_vec: Vec<Vec<f64>> = Vec::new();
            for _ in 0..spring_constant_vec.len(){
                let spring_sts = SpringStatus{
                    speed:0.0,
                    position:0.0,
                };
                spring_sts_vec.push(spring_sts);
                energy_spring_vec.push(vec!(0.0));
            }
            spring_sts_ch_vec.push(spring_sts_vec);
            energy_spring_ch_vec.push(energy_spring_vec);
        }
        Ok(ResonanceSpectrum {
            spring_constant_vec: spring_constant_vec,
            data_period : 1.0/f64::from(u32::try_from(data_frequency)?),
            spring_sts_ch_vec: spring_sts_ch_vec,
            energy_spring_ch_vec: energy_spring_ch_vec,
        }) 
    }

    pub fn resonance(&mut self, ch_idx:usize , sound_data_slice: &[f64])  -> Result<Vec<Vec<f64>>> {
        let mut energy_max = 0.0;
        let spring_sts_vec: &mut Vec<SpringStatus> = &mut self.spring_sts_ch_vec[ch_idx];
        let energy_spring_vec: &mut Vec<Vec<f64>> = &mut self.energy_spring_ch_vec[ch_idx];
        let mut ret_energy_spring_vec: Vec<Vec<f64>> = Vec::with_capacity(self.spring_constant_vec.len());
        for (spring_idx, spring_constant) in self.spring_constant_vec.iter().enumerate() {
            let spring_sts = &mut spring_sts_vec[spring_idx];
            let enegy_vec = &mut energy_spring_vec[spring_idx];
            let mut ret_energy_vec :Vec<f64> = Vec::with_capacity(sound_data_slice.len());
            for data in sound_data_slice.iter() {
                spring_sts.speed = (*data - spring_constant * spring_sts.position - spring_sts.speed * 100.0)*self.data_period + spring_sts.speed;
                spring_sts.position = spring_sts.speed*self.data_period + spring_sts.position;
                let route_energy = (0.5*spring_sts.speed.powi(2) + 0.5*spring_constant*spring_sts.position.powi(2)).powf(0.5);
                enegy_vec.push(route_energy);
                ret_energy_vec.push(route_energy);
                if energy_max < route_energy {
                    energy_max = route_energy;
                }
            }
            ret_energy_spring_vec.push(ret_energy_vec);
        }
        //println!("{}",energy_max);
        Ok(ret_energy_spring_vec)
    }
}