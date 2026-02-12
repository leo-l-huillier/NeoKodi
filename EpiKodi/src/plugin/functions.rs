/*


*/

use std::os::raw::c_char;

pub type GreetFunc = unsafe extern "C" fn(*const c_char) -> *mut c_char;
pub type GetArtistMetadataFunc = unsafe extern "C" fn(*const c_char) -> *mut c_char;
pub type GetFilmMetadataFunc = unsafe extern "C" fn(*const c_char) -> *mut c_char;

pub type NameFunc = unsafe extern "C" fn() -> *mut c_char;
pub type VersionFunc = unsafe extern "C" fn() -> *mut c_char;
pub type FreeStringFunc = unsafe extern "C" fn(*mut c_char);
pub type PluginTypeFunc = unsafe extern "C" fn() -> *mut c_char;