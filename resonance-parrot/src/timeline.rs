use std::time;

use std::convert::TryFrom;

use std::sync::mpsc::{Sender, Receiver};

use super::error::*;
use super::{ThreadID, AppEvent};

#[derive(Clone)]
#[derive(PartialEq)]
pub enum TimelineReportType {
    Periodical,
    Response,
    //ChangedByRequest,
    ChangedBySelf,
}

#[derive(Clone)]
#[derive(PartialEq)]
pub enum PlayStatus {
    Play,
    Pause,
    Stop,
}

#[derive(Clone)]
pub struct TimelineBase {
    pub len: usize,
    pub frequency: usize,
    pub event_divisor: usize
}

pub struct TimelineReport {
    pub report_type: TimelineReportType,
    pub timeline: TimelineStatus,
}

#[derive(Clone)]
pub struct TimelineStatus {
    pub play_status: PlayStatus,
    pub time_counter: usize,
    pub base:TimelineBase
}

impl TimelineStatus {
    fn new(request:TimelineRequest) -> RpResult<TimelineStatus> {
        if request.op_base.is_none() {
            return Err(ResonanceParrotError::new("All TimeLine Valueables needs to be set the first time!"));
        }
        Ok(TimelineStatus {
            play_status: PlayStatus::Stop, 
            time_counter:0,
            base: request.op_base.unwrap(),
        }) 
    }

    fn stop(&mut self) {
        self.play_status = PlayStatus::Stop;
        self.time_counter = 0;
    }

    fn update(&mut self, request:TimelineRequest) -> RpResult<()> {
        match request.request_type {
            TimelineRequestType::PlayOrPause => {
                match self.play_status {
                    PlayStatus::Play => { self.play_status = PlayStatus::Pause; },
                    PlayStatus::Pause => { self.play_status = PlayStatus::Play; },
                    PlayStatus::Stop => { self.play_status = PlayStatus::Play; },
                }
            }
            TimelineRequestType::Stop => {
                self.play_status = PlayStatus::Stop;
                self.time_counter = 0;
            }
            TimelineRequestType::Next => {
                self.play_status = PlayStatus::Stop;
                self.time_counter = 0;
            }
            TimelineRequestType::Prev => {
                self.play_status = PlayStatus::Stop;
                self.time_counter = 0;
            }
            TimelineRequestType::Point => {
                if let Some(time_counter) = request.op_time_counter {
                    self.time_counter = time_counter;
                }
            }
            _ => {
                return Err(ResonanceParrotError::new("TimelineRequestType is Wrong!"));
            }
        }
        Ok(())
    }

}

#[derive(Clone)]
#[derive(PartialEq)]
#[allow(dead_code)]
pub enum TimelineRequestType {
    Open,
    Close,
    Status,
    PlayOrPause,
    Stop,
    Next,
    Prev,
    Point,
}

pub struct TimelineRequest {
    request_type: TimelineRequestType,
    op_time_counter: Option<usize>,
    op_base:Option<TimelineBase>
}

#[allow(dead_code)]
impl TimelineRequest {
    pub fn open(len: usize, frequency: usize, event_divisor: usize) -> TimelineRequest {
        TimelineRequest {
            request_type: TimelineRequestType::Open,
            op_time_counter: None,
            op_base:Some(TimelineBase {
                len: len,
                frequency: frequency,
                event_divisor: event_divisor
            })
        }
    }
    pub fn close() -> TimelineRequest {
        TimelineRequest {
            request_type: TimelineRequestType::Close,
            op_time_counter: None,
            op_base:None
        }
    }
    pub fn play_or_pause() -> TimelineRequest {
        TimelineRequest {
            request_type: TimelineRequestType::PlayOrPause,
            op_time_counter: None,
            op_base:None
        }
    }
    pub fn stop() -> TimelineRequest {
        TimelineRequest {
            request_type: TimelineRequestType::Stop,
            op_time_counter: None,
            op_base:None
        }
    }
    pub fn point(counter: usize) -> TimelineRequest {
        TimelineRequest {
            request_type: TimelineRequestType::Point,
            op_time_counter: Some(counter),
            op_base:None
        }
    }
    pub fn status() -> TimelineRequest {
        TimelineRequest {
            request_type: TimelineRequestType::Status,
            op_time_counter: None,
            op_base:None
        }
    }
}


#[allow(dead_code)]
struct FrequencySlice {
    base_instant: time::Instant,
    base_count: usize,
}

#[allow(dead_code)]
impl FrequencySlice {
    fn new(time_counter: usize) -> FrequencySlice {
        FrequencySlice {
            base_instant:time::Instant::now(),
            base_count: time_counter
        }
    }

    fn next_dur(& mut self, timeline: &TimelineStatus) -> RpResult<Option<time::Duration>> {
        let sleep_duration : Option<time::Duration>;
        let freq = timeline.base.frequency;
    
        if timeline.play_status == PlayStatus::Play {
            if timeline.time_counter < self.base_count {
                return Err(ResonanceParrotError::new("TimeCounter Error!"));
            }
            let counter_diff = timeline.time_counter - self.base_count;
            let seconds =  counter_diff / freq;
            let nano_seconds = 1000000000 * (counter_diff % freq)/ freq;
            let duration = time::Duration::new(u64::try_from(seconds)?, u32::try_from(nano_seconds)?);
            sleep_duration = duration.checked_sub(self.base_instant.elapsed());
        }
        else {
            let nano_seconds = 1000000000 / freq;
            sleep_duration = Some(time::Duration::new(0, u32::try_from(nano_seconds)?));
        }
        Ok(sleep_duration)
    }
}

struct TimeLine {
    timeline: TimelineStatus,
    freq_slice: FrequencySlice,
    event_id: usize,
    event_sender: Sender<AppEvent>,
    to_timeline_receiver: Receiver<TimelineRequest>,
    from_timeline_sender: Sender<TimelineReport>
}

#[allow(dead_code)]
impl TimeLine {
    fn new(request: TimelineRequest,event_sender: Sender<AppEvent>, from_timeline_sender: Sender<TimelineReport>, to_timeline_receiver: Receiver<TimelineRequest>) -> RpResult<TimeLine> {
        Ok(TimeLine {
            timeline: TimelineStatus::new(request)?,
            freq_slice:  FrequencySlice::new(0),
            event_id: 0,
            event_sender: event_sender,
            to_timeline_receiver:to_timeline_receiver,
            from_timeline_sender: from_timeline_sender
        })
    }

    fn main(&mut self) -> RpResult<()> {
        loop {
            let op_dur = self.freq_slice.next_dur(&self.timeline)?;
            let res_request;
            if let Some(dur) = op_dur {
                res_request = self.to_timeline_receiver.recv_timeout(dur);
            }
            else {
                // No wait
                res_request = self.to_timeline_receiver.recv_timeout(time::Duration::new(0,0));
            }
            match res_request {
                Ok(request) => {
                    match request.request_type {
                        TimelineRequestType::Open => {  self.timeline = TimelineStatus::new(request)?; }
                        TimelineRequestType::Close => { break; }
                        TimelineRequestType::Status => {
                            self.send(TimelineReportType::Response)?;
                        }
                        _ => {  self.timeline.update(request)?; }
                    }
                    self.send(TimelineReportType::ChangedBySelf)?;
                    self.freq_slice = FrequencySlice::new(self.timeline.time_counter);
                }
                Err(_) => {
                    if self.timeline.play_status == PlayStatus::Play {
                        self.timeline.time_counter += 1;
                        //if self.timeline.time_counter % (self.timeline.base.event_divisor * 100) == 0{
                        //    println!("time_counter {}", self.timeline.time_counter);
                        //}
                        if self.timeline.time_counter % self.timeline.base.event_divisor == 0{
                            self.send(TimelineReportType::Periodical)?;
                        }
                        if self.timeline.time_counter > self.timeline.base.len - 1 {
                            self.timeline.stop();
                            self.send(TimelineReportType::ChangedBySelf)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn send(&mut self, report_type: TimelineReportType) -> RpResult<()> {
        let timeline_report = TimelineReport {
            report_type: report_type,
            timeline: self.timeline.clone()
        };
        self.from_timeline_sender.send(timeline_report)?;
        self.event_sender.send(AppEvent{thread_id:ThreadID::TimeCounter, event_id:self.event_id})?;
        self.event_id += 1;
        Ok(())
    }
}

pub fn timeline_thread_main(event_sender: Sender<AppEvent>, from_timeline_sender: Sender<TimelineReport>, to_timeline_receiver: Receiver<TimelineRequest>) -> RpResult<()> {
    let request = to_timeline_receiver.recv()?;
    let mut timeline = TimeLine::new(request, event_sender, from_timeline_sender, to_timeline_receiver)?;
    timeline.main()?;
    Ok(())
}

pub fn timeline_thread(event_sender: Sender<AppEvent>, from_timeline_sender: Sender<TimelineReport>, to_timeline_receiver: Receiver<TimelineRequest>) -> RpResult<()> {
    match timeline_thread_main(event_sender, from_timeline_sender, to_timeline_receiver) {
        Ok(_ret) => { /* Nothing to do */ }
        Err(err) => {
            println!("Error! timeline_thread!");
            return Err(err);
        }
    }
    Ok(())
}