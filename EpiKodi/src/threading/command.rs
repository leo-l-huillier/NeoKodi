
/*
This file defines commands and events for media playback control.
*/
use crate::media::data::MediaType;
use std::path::PathBuf;
use crate::media::data::MediaInfo;

pub enum Command {
    ChangeLibraryPath(PathBuf),

    AddSource(PathBuf, MediaType), // path, media type
    RemoveSource(PathBuf, MediaType), // path, media type
    Reload(),
    /*
    TODO

    RemoveSource(PathBuf), // path
    //ScanLibrary, // allows the user to rescan the library todo: add options like full scan or quick scan, 
    ajouter un fichier de config, 
    scanner seulement la premiere fois et puis tous les x jours ou sur un bouton
    ajouter des logs
    */
            

    GetMediaFromPath(PathBuf), // path
    GetAllMedia(),
    GetMediaFromType(MediaType), // media type
    GetMediaFromTag(String), // tag name
    GetMediaFromPlaylist(i64), // playlist id
    UpdateMediaState(i64, i32, f64), // media id, status, time_stop
    /*
    TODO:
    Récupération automatique d'informations supplémentaires sur les médias (affiches, synopsis, note, casting) depuis des bases de données en ligne.
    */

    Play(i64), // media id
    Pause(i64), // media id 
    Resume(i64), // media id
    Stop(i64), // media id
    Info(i64), // media id
    /*
    TODO:
    Avance rapide / Retour rapide (±10 secondes)
    Contrôle du volume (0-100%)
    Barre de progression avec seek (clic pour aller à un moment précis)
    file d'attente de lecture (playlist dynamique)
    lecture aleatoire
    repeat
    */
    //ActuaizeMedia       //if a user want to change at what time he is in a media

    AddTag(String), // tag name
    GetTagId(String), // tag name
    AddTagToMedia(i64, i64), // media_id, tag_id
    /*
    TODO:

    */
    //RemoveTag(String),

    AddPlaylist(String), // playlist name
    GetPlaylistId(String), // playlist name
    AddMediaToPlaylist(i64, i64), // media_id, playlist_id
    //LoadM3U(String),
    RemoveMediaFromPlaylist(i64, i64), // media_id, playlist_id
    DeletePlaylist(i64), // playlist_id
    GetAllPlaylists(), // returns Vec<(playlist_id, playlist_name)>
    /*
    TODO:
    delete playlist
    */

    GetArtistMetadataFromPlugin(String), // artist name

}

pub enum Event {
    Finished(i64),
    NowPlaying(i64),
    Data(String),
    Info(MediaInfo),
    IDList(Vec<i64>),
    MediaList(Vec<MediaInfo>),
    ArtistInfoReceived(String),
    M3UList(Vec<crate::iptv::parser::TVChannel>),
    PlaylistList(Vec<(i64, String)>),
}