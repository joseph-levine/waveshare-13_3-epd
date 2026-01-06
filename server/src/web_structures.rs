use actix_multipart::form::json::Json;
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::MultipartForm;
use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Debug, Deserialize_repr, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum DevicePixelRatio {
    One = 1,
    Two = 2,
    Three = 3,
}

impl Into<u8> for DevicePixelRatio {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for DevicePixelRatio {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(DevicePixelRatio::One),
            2 => Ok(DevicePixelRatio::Two),
            3 => Ok(DevicePixelRatio::Three),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Deserialize_repr, Copy, Clone)]
#[repr(u8)]
pub enum ValidHour {
    Morning = 5,
    Noon = 12,
    Night = 18,
}
impl Into<u8> for ValidHour {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryInto<ValidHour> for u8 {
    type Error = ();

    fn try_into(self) -> Result<ValidHour, Self::Error> {
        match self {
            5 => Ok(ValidHour::Morning),
            12 => Ok(ValidHour::Noon),
            18 => Ok(ValidHour::Night),
            _ => Err(()),
        }
    }
}
#[derive(Debug, Deserialize_repr, Copy, Clone)]
#[repr(u8)]
pub enum ValidDay {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Sunday = 7,
}

impl Into<u8> for ValidDay {
    fn into(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Deserialize)]
pub struct UploadJsonForm {
    pub show_now: bool,
}

#[derive(Debug, MultipartForm)]
pub struct UploadMultipartForm {
    #[multipart()]
    pub file: TempFile,
    pub json: Json<UploadJsonForm>,
}

#[derive(Debug, Deserialize)]
pub struct AuthData {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct DevicePixelRatioQuery {
    pub d: DevicePixelRatio,
}
