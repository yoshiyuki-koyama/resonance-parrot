use std::option::Option;
use std::path::Path;
use std::path::PathBuf;
use std::convert::TryFrom;
use std::io::prelude::*;
use std::io::BufReader;

use std::fs::File;

mod error;
use error::*;

#[cfg(test)]
mod tests {
    use super::WavFile;
    #[test]
    fn test_get_wav_file() {
        let mut wav_file = WavFile::new();
        let res_open = wav_file.open(std::path::Path::new(r"./test.wav"));
        match res_open {
            Ok(_) => {},
            Err(err) => println!("Error!!! {}",err)
        }
        let res_wav_audio = wav_file.get_wav_audio();
        if let Ok(wav_audio) = res_wav_audio {
            // Check
            println!("ch = {}, sampling_rate = {}, bits_per_sample = {}",wav_audio.fmt.channel, wav_audio.fmt.sampling_rate, wav_audio.fmt.bits);
            println!("data length = {}",wav_audio.data.len());
            print!("data = ");
            for (i,data) in wav_audio.data.iter().enumerate() {
                print!("{:x} ", data);
                if i >= 3 {
                    print!("\n");
                    break;
                }
            }

            // New File and Write
            let mut new_file = WavFile::new();
            new_file.update_wav_audio(&wav_audio).unwrap();
            for subchunk in wav_file.sub_chunks {
                if subchunk.name != [b'f',b'm',b't',b' '] && subchunk.name != [b'f',b'm',b't',b' '] {
                    new_file.update_sub_chunk(subchunk).unwrap();
                }
            }
            new_file.save_as(std::path::Path::new(r"./new.wav")).unwrap();

        }
        else if let Err(err) = res_wav_audio {
            println!("Error!!! {}",err)
        }

    }
}


#[allow(dead_code)]
#[derive(Clone)]
pub struct Fmt {
    pub id: usize,
    pub channel: usize,
    pub sampling_rate: usize,
    pub bits: usize
}

#[allow(dead_code)]
pub struct WavAudio {
    pub fmt: Fmt,
    pub data: Vec<u8>
}

// Without "fmt " chunk abd  "data" chunk
#[allow(dead_code)]
#[derive(Clone)]
pub struct SubChunk {
    pub name: [u8;4],
    pub body_size: usize,
    pub data: Vec<u8>
}

impl SubChunk {
    pub fn new() -> SubChunk {
        SubChunk {name:[0,0,0,0], body_size:0, data:Vec::new()}
    }
}

pub struct WavFile {
    pub file_path: PathBuf,
    pub size: usize,
    pub sub_chunks: Vec<SubChunk>
}

impl WavFile {
    pub fn new() -> WavFile {
        WavFile {file_path:PathBuf::new(), size:0, sub_chunks:Vec::new()}
    }

    pub fn open(&mut self, file_path: &Path) -> Result<()> {
        // -- Check Parameter --
        if !file_path.is_file() {
            return Err(WavFileError::new(&format!("Path is Not File! Path:{}", file_path.display())));
        }
        if let Some(ext) = file_path.extension() {
            if (ext != "wav") && (ext != "WAV") {
                return Err(WavFileError::new(&format!("Path Extension is Not .wav!! File:{}", file_path.display())));
            }
        }
        else {
            return Err(WavFileError::new(&format!("Path is Not File! Path:{}", file_path.display())));
        }

        // -- Read Whole File --
        let target_file = File::open(file_path)?;
        let mut buf = Vec::new();
        let file_size = BufReader::new(&target_file).read_to_end(&mut buf)?;
        
        // -- Get WavFile Construction --
        // "RIFF"
        if buf[0x00..0x04] != [b'R',b'I',b'F',b'F'] {
            return Err(WavFileError::new(&format!("Not compatible wav format! \"RIFF\" File:{}", file_path.display())));
        }
        // RIFF Size
        let riff_size = usize::try_from(u32::from_le_bytes(<[u8;4]>::try_from(&buf[0x04..0x08])?))?;
        if riff_size != file_size - 8 {
            return Err(WavFileError::new(&format!("Not compatible wav format! RIFF Size File:{}", file_path.display())));
        }
        // "WAVE"
        if buf[0x08..0x0c] != [b'W',b'A',b'V',b'E'] {
            return Err(WavFileError::new(&format!("Not compatible wav format! \"WAVE\" File:{}", file_path.display())));
        }
        
        let sub_chunks_vec = self.get_sub_chunks(buf[0x0c..].to_vec(), file_size - 12)?;

        self.file_path = file_path.to_path_buf();
        self.size = file_size;
        self.sub_chunks = sub_chunks_vec;
        Ok(())
    }

    pub fn save(&mut self) -> Result<()> {
        let file_path = self.file_path.clone();
        self.save_as(&file_path)?;
        Ok(())
    }

    pub fn save_as(&mut self, file_path: &Path) -> Result<()> {
        // -- Check Parameter --
        if let Some(ext) = file_path.extension() {
            if (ext != "wav") && (ext != "WAV") {
                return Err(WavFileError::new(&format!("Path Extension is Not .wav!! File:{}", file_path.display())));
            }
        }
        else {
            return Err(WavFileError::new(&format!("Path is Not File! Path:{}", file_path.display())));
        }
        let mut buf: Vec<u8> = Vec::new();
        buf.append(&mut [b'R',b'I',b'F',b'F'].to_vec());
        let mut riff_size:usize = 4;
        for sub_chunk in &self.sub_chunks {
            riff_size += sub_chunk.body_size + 8;
        }
        buf.append(&mut riff_size.to_le_bytes()[0..4].to_vec());
        buf.append(&mut [b'W',b'A',b'V',b'E'].to_vec());
        for sub_chunk in &mut self.sub_chunks {
            buf.append(&mut sub_chunk.name.to_vec());
            buf.append(&mut sub_chunk.body_size.to_le_bytes()[0..4].to_vec());
            buf.append(&mut sub_chunk.data.to_vec());
        }
        let mut target_file = File::create(file_path)?;
        target_file.write_all(&buf)?;

        // Update Self Infomation
        self.file_path = file_path.to_path_buf();
        self.size = riff_size + 8;
        Ok(())
    }

    fn get_sub_chunks(&self, buf: Vec<u8>, chunks_size: usize) -> Result<Vec<SubChunk>> {
        let mut sub_chunks_vec:Vec<SubChunk> = Vec::new(); 
        let mut chunk_head_addr: usize = 0x00;
        while chunks_size - chunk_head_addr >= 8 {
            let chunk_head_buf = &buf[chunk_head_addr..];
            let chunk_body_size = usize::try_from(u32::from_le_bytes(<[u8;4]>::try_from(&chunk_head_buf[0x04..0x08])?))?;
            if chunk_head_buf.len() < chunk_body_size + 8 {
                return Err(WavFileError::new("Chunk Size is wrong!"));
            }
            let sub_chunk = SubChunk {
                name: [ chunk_head_buf[0x00],
                        chunk_head_buf[0x01],
                        chunk_head_buf[0x02],
                        chunk_head_buf[0x03]],
                body_size: chunk_body_size,
                data: chunk_head_buf[8..(chunk_body_size + 8)].to_vec()
            };
            sub_chunks_vec.push(sub_chunk);
            chunk_head_addr += 8 + chunk_body_size;
        }
        return Ok(sub_chunks_vec);
    }

    pub fn update_sub_chunk(&mut self, new_chunk:SubChunk) -> Result<()> {
        let mut op_chunk_idx: Option<usize> = None;
        for (i,existing_chunk) in self.sub_chunks.iter().enumerate() {
            if existing_chunk.name == new_chunk.name{
                op_chunk_idx = Some(i);
                break;
            }
        }
        if let Some(idx) = op_chunk_idx {
            self.sub_chunks[idx] = new_chunk;
        }
        else {
            self.sub_chunks.push(new_chunk);
        }
        Ok(())
    }


    pub fn get_wav_audio(&self) -> Result<WavAudio> {
        let mut op_fmt: Option<Fmt> = None; 
        let mut op_data: Option<Vec<u8>> = None;
        for sub_chunk in &self.sub_chunks {
            match sub_chunk.name {
                [b'f',b'm',b't',b' '] => {
                    if op_fmt.is_none() {
                        op_fmt = Some(self.get_fmt(&sub_chunk.data)?);
                    }
                    else {
                        return Err(WavFileError::new("There are two or more fmt chunks!"));
                    }
                }
                [b'd',b'a',b't',b'a'] => {
                    if op_data.is_none() {
                        op_data = Some(sub_chunk.data.clone());
                    }
                    else {
                        return Err(WavFileError::new("There are two or more data chunks!"));
                    }
                }
                _ => {}
            }
        }
        if op_fmt.is_none() {
            return Err(WavFileError::new("There are no fmt chunks!"));
        }
        if op_data.is_none() {
            return Err(WavFileError::new("There are no data chunks!"));
        }
        Ok(WavAudio {
            fmt: op_fmt.unwrap(),
            data: op_data.unwrap()
        })
    }

    pub fn update_wav_audio(&mut self, ref_wav_audio: &WavAudio) -> Result<()> {
        let fmt_buf: Vec<u8> = self.set_fmt(&ref_wav_audio.fmt)?;
        let data_buf: Vec<u8> = ref_wav_audio.data.clone();
        let mut op_fmt_chunk_idx: Option<usize> = None;
        let mut op_data_chunk_idx: Option<usize> = None;
        for (i,sub_chunk) in self.sub_chunks.iter().enumerate() {
            match sub_chunk.name {
                [b'f',b'm',b't',b' '] => {
                    if op_fmt_chunk_idx.is_none() {
                        op_fmt_chunk_idx = Some(i);
                    }
                    else {
                        return Err(WavFileError::new("There are two or more fmt chunks!"));
                    }
                }
                [b'd',b'a',b't',b'a'] => {
                    if op_data_chunk_idx.is_none() {
                        op_data_chunk_idx = Some(i);
                    }
                    else {
                        return Err(WavFileError::new("There are two or more data chunks!"));
                    }
                }
                _ => {}
            }
        }
        if let Some(idx) = op_fmt_chunk_idx {
            self.sub_chunks[idx].body_size = fmt_buf.len();
            self.sub_chunks[idx].data = fmt_buf;
        }
        else{
            // Create New Fmt Chunk
            
            let mut sub_chunk = SubChunk::new();
            sub_chunk.name = [b'f',b'm',b't',b' '];
            sub_chunk.body_size = fmt_buf.len();
            sub_chunk.data = fmt_buf;
            self.sub_chunks.push(sub_chunk);
        }
        if let Some(idx) = op_data_chunk_idx {
            self.sub_chunks[idx].body_size = data_buf.len();
            self.sub_chunks[idx].data = data_buf;
        }
        else {
            // Create New Data Chunk
            let mut sub_chunk = SubChunk::new();
            sub_chunk.name = [b'd',b'a',b't',b'a'];
            sub_chunk.body_size = data_buf.len();
            sub_chunk.data = data_buf;
            self.sub_chunks.push(sub_chunk);        
        }
        Ok(())
    }
    
    fn get_fmt(&self, ref_chunk_body: &Vec<u8>) -> Result<Fmt> {
        // fmt chunk
        // format id
        let format_id = usize::from(u16::from_le_bytes(<[u8;2]>::try_from(&ref_chunk_body[0x00..0x02])?));
        if format_id != 1 && format_id != 3 {
            return Err(WavFileError::new("Not compatible wav format!  fmt Chunk Size"));
        }
        // channel
        let channel = usize::from(u16::from_le_bytes(<[u8;2]>::try_from(&ref_chunk_body[0x02..0x04])?));
        // Sampling Rate
        let sampling_rate = usize::try_from(u32::from_le_bytes(<[u8;4]>::try_from(&ref_chunk_body[0x04..0x08])?))?;
        // Byte Per Sec
        let bytes_per_sec = usize::try_from(u32::from_le_bytes(<[u8;4]>::try_from(&ref_chunk_body[0x08..0x0c])?))?;
        // Block Size
        let block_size = usize::from(u16::from_le_bytes(<[u8;2]>::try_from(&ref_chunk_body[0x0c..0x0e])?));
        // Bit Rate
        let bits = usize::from(u16::from_le_bytes(<[u8;2]>::try_from(&ref_chunk_body[0x0e..0x10])?));

        // Check Byte Per Sec.
        if bytes_per_sec != channel * sampling_rate * (bits / 8)  {
            return Err(WavFileError::new("Not match parameter! Byte Per Sec"));
        }
        // Check Block Size.
        if block_size != channel * bits / 8 {
            return Err(WavFileError::new("Not match parameter! Block Size"));
        }
        Ok(Fmt {
            id:format_id,
            channel:channel,
            sampling_rate:sampling_rate,
            bits:bits
        })
    }

    fn set_fmt(&self, ref_fmt:&Fmt) -> Result<Vec<u8>> {
        // fmt chunk
        
        let mut chunk_body: Vec<u8> = Vec::new();
        
        // format id
        chunk_body.append(&mut ref_fmt.id.to_le_bytes()[0..2].to_vec());
        // channel
        chunk_body.append(&mut ref_fmt.channel.to_le_bytes()[0..2].to_vec());
        // Sampling Rate
        chunk_body.append(&mut ref_fmt.sampling_rate.to_le_bytes()[0..4].to_vec());
        // Byte Per Sec
        chunk_body.append(&mut (ref_fmt.channel * ref_fmt.sampling_rate * (ref_fmt.bits / 8)).to_le_bytes()[0..4].to_vec());
        // Block Size
        chunk_body.append(&mut (ref_fmt.channel * ref_fmt.bits / 8).to_le_bytes()[0..2].to_vec());
        // Bit Rate
        chunk_body.append(&mut ref_fmt.bits.to_le_bytes()[0..2].to_vec());
        Ok(chunk_body)
    }
}

fn fmt_check(ref_fmt: &Fmt) -> Result<()> {
    if ref_fmt.channel < 1 || ref_fmt.channel > 2 {
        return Err(WavFileError::new("Irregal Format! Not Supported Channel."));
    }
    if ref_fmt.bits == 0 || ref_fmt.bits > 64 {
        return Err(WavFileError::new("Irregal Format! Not Supported Bit Rate."));
    }
    match ref_fmt.sampling_rate {
        8000  => {},
        16000 => {},
        44100 => {},
        48000 => {},
        _ => {return Err(WavFileError::new("Irregal Format! Not Supported Sampling Rate."));}
    }
    Ok(())
}

fn bytes_to_f64wave(id:usize, bytes: &[u8]) -> Result<f64> {
    let len = bytes.len();
    let mut buffer:[u8; 4] = [0; 4];
    match id {
        1 => {
            if len == 1 {   //unsigned 8bit
                buffer[3] = bytes[0] - 128;
                Ok(f64::from(i32::from_le_bytes(buffer))/(f64::from(i32::MAX)+1.0))
            }
            else if len == 2 || len == 3 { //signed 16bit,24bit
                for i in 0..len {
                    buffer[4-len+i] = bytes[i];
                }
                Ok(f64::from(i32::from_le_bytes(buffer))/(f64::from(i32::MAX)+1.0))
            }
            else {
                Err(WavFileError::new("Size is Too Large!"))
            }
        },
        3 => {
            if len == 4 {   //32bit float
                Ok(f64::from(f32::from_le_bytes(<[u8; 4]>::try_from(bytes)?)))
            }
            else {
                Err(WavFileError::new("Size is Too Small or Large!"))
            }
        },
        _ => {
            Err(WavFileError::new("Format is Irregal!"))
        }
    }
}

fn f64wave_to_bytes(id:usize, f64_val: f64, len:usize) -> Result<Vec<u8>> {
    match id {
        1 => {
            let i32_val :i32;
            if f64_val < -1.0 {
                i32_val = i32::MIN;
            }
            else if 1.0 < f64_val {
                i32_val = i32::MAX;
            }
            else {
                i32_val = (f64_val * (f64::from(i32::MAX)+1.0) ) as i32;
            }

            let mut buffer:[u8; 4] = i32_val.to_le_bytes();
            // TODO: Max Value Check for each length
            if len == 1 {   //unsigned 8bit
                buffer[3] += 128;
                Ok(vec!(buffer[3]))
            }
            else if len == 2 || len == 3 { //signed 16bit,24bit
                Ok(buffer[4-len..4].to_vec())
            }
            else {
                Err(WavFileError::new("Size is Too Small or Large!"))
            }
        },
        3 => {
            if len == 4 {   //32bit float
                Ok((f64_val as f32).to_le_bytes().to_vec())
            }
            else {
                Err(WavFileError::new("Size is Too Small or Large!"))
            }
        }
        _ => {
            Err(WavFileError::new("Format is Irregal!"))
        }
    }
}

pub fn to_channel_vec(ref_wav_audio: &WavAudio) -> Result<Vec<Vec<f64>>> {
    fmt_check(&ref_wav_audio.fmt)?;
    let mut ch_vec = Vec::new();
    
    let size = ref_wav_audio.fmt.bits / 8;
    let step = ref_wav_audio.fmt.channel * size;
    for _ in 0..ref_wav_audio.fmt.channel {
        ch_vec.push(Vec::new());
    }
    for (pos,_dummy) in ref_wav_audio.data.iter().enumerate().step_by(step) {
        //println!("{}",pos);
        for ch in 0..ref_wav_audio.fmt.channel {
            let stt = pos + ch*size;
            ch_vec[ch].push(bytes_to_f64wave(ref_wav_audio.fmt.id, &ref_wav_audio.data[stt..stt+size])?);
        }
    }
    Ok(ch_vec)
}

pub fn to_wav_audio(ref_ch_vec: &Vec<Vec<f64>>, ref_fmt: &Fmt) -> Result<WavAudio> {
    fmt_check(ref_fmt)?;
    let mut data = Vec::new();

    for ch in ref_ch_vec {
        if ref_ch_vec[0].len() != ch.len() {
            return Err(WavFileError::new("Irregal Data Vector! Data vector length are not same."));
        }
    }
    for (i,_) in ref_ch_vec[0].iter().enumerate() {
        for ch in 0..ref_fmt.channel {
            let mut l_bytes = f64wave_to_bytes(ref_fmt.id, ref_ch_vec[ch][i], ref_fmt.bits/8)?;
            data.append(&mut l_bytes);
        }
    }
    Ok(WavAudio{fmt: (*ref_fmt).clone(), data: data})
}


