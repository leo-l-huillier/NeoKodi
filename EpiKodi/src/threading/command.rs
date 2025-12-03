

pub enum Command {
    Play(u32),
    Pause(u32),
    Resume(u32),
    Stop(u32),
    Info(u32),
    MediaScan(u32),
}

pub enum Event {
    Finished(u32),
    NowPlaying(u32),
    Data(String),
}