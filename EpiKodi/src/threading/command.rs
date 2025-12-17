
/*
This file defines commands and events for media playback control.
*/
use crate::media::data::MediaType;
use std::path::PathBuf;
use crate::media::data::MediaInfo;

pub enum Command {

    AddSource(PathBuf, MediaType), // path
    //ScanLibrary,        // allows the user to rescan the library

    GetMediaFromPath(PathBuf), // path
    GetAllMedia(),
    GetMediaFromType(MediaType), // media type
    GetMediaFromTag(String),

    Play(i64), // media id
    Pause(i64), // media id 
    Resume(i64), // media id
    Stop(i64), // media id
    Info(i64), // media id
    //ActuaizeMedia       //if a user want to change at what time he is in a media

    AddTag(String), // tag name
    AddTagToMedia(i64, i64), // media_id, tag_id
    GetTagId(String), // tag name
    //RemoveTag(String),
}

pub enum Event {
    Finished(i64),
    NowPlaying(i64),
    Data(String),
    Info(MediaInfo),
    IDList(Vec<i64>),
    MediaList(Vec<MediaInfo>),
}