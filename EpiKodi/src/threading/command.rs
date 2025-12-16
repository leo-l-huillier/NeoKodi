
/*
This file defines commands and events for media playback control.
*/

pub enum Command {
    Play(i64),
    Pause(i64),
    Resume(i64),
    Stop(i64),
    Info(i64),
}

pub enum Event {
    Finished(i64),
    NowPlaying(i64),
    Data(String),
}