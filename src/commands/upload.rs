use std::{
    fs,
    io::{Error, ErrorKind, Read},
    path::Path,
    process::Command,
};

use arboard::Clipboard;
use ffmpeg_sidecar::{command::FfmpegCommand, ffprobe::ffprobe_sidecar_path};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    commands::login::login, config::{self, HTTP_CLIENT}, spinner::get_spinner, token::get_token
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct UploadUrlResponse {
    video_id: String,
    video_upload_url: String,
    thumbnail_upload_url: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct InsertUrlResponse {
    video_url: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct UploadRequestPayload {
    video_size: u64,
    thumbnail_size: u64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct InsertPayload {
    video_id: String,
    file_name: String,
    duration: u64,
    size: u64,
}

fn get_upload_url(
    token: &str,
    video_size: u64,
    thumbnail_size: u64,
) -> Result<UploadUrlResponse, Error> {
    let payload = UploadRequestPayload {
        video_size: video_size,
        thumbnail_size: thumbnail_size,
    };

    let response = HTTP_CLIENT
        .post(format!("{}/api/upload", *config::API_ENDPOINT))
        .header("token", token)
        .json(&serde_json::json!(payload))
        .send()
        .map_err(|e| {
            if e.is_timeout() {
                Error::new(ErrorKind::TimedOut, "Request timed out")
            } else if e.is_connect() {
                Error::new(ErrorKind::NotConnected, "Cannot connect to server")
            } else {
                Error::new(ErrorKind::Other, format!("Error: {}", e))
            }
        })?;

    match response.status() {
        StatusCode::OK => {
            let raw_body = response.text().unwrap_or_default();
            serde_json::from_str::<UploadUrlResponse>(&raw_body)
                .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to wrap JSON: {}", e)))
        }
        e => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Failed requesting upload url: {}", e),
            ));
        }
    }
}

fn get_thumbnail(file_path: &str) -> Result<Vec<u8>, Error> {
    let mut thumbnail = FfmpegCommand::new()
        .input(file_path)
        .arg("-vf")
        .arg("thumbnail,scale=480:-1")
        .frames(1)
        .format("image2pipe")
        .codec_video("mjpeg")
        .pipe_stdout()
        .spawn()?;

    let mut jpeg_bytes = Vec::new();
    thumbnail
        .take_stdout()
        .unwrap()
        .read_to_end(&mut jpeg_bytes)?;

    Ok(jpeg_bytes)
}

fn put_file(token: &str, signed_url: &str, body: Vec<u8>) -> Result<(), Error> {
    let response = HTTP_CLIENT
        .put(signed_url)
        .header("token", token)
        .body(body)
        .send()
        .map_err(|e| {
            if e.is_timeout() {
                Error::new(ErrorKind::TimedOut, "Request timed out")
            } else if e.is_connect() {
                Error::new(ErrorKind::NotConnected, "Cannot connect to server")
            } else {
                Error::new(ErrorKind::Other, format!("Error: {}", e))
            }
        })?;

    match response.status() {
        StatusCode::OK => Ok(()),
        e => Err(Error::new(
            ErrorKind::Other,
            format!("Error uploading file: {}", e),
        )),
    }
}

pub fn upload(file_path: &str) -> Result<String, Error> {
    let token_spinner = get_spinner("Resolving token...".to_string());
    let token = match get_token() {
        Ok(token) => token,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => return Err(Error::new(ErrorKind::NotFound, "Token not found, try logging in first with login command")),
            other => return Err(Error::new(other, e.to_string()))
        }
    };
    token_spinner();

    let metadata_spinner = get_spinner("Resolving video metadata...".to_string());
    let thumbnail = get_thumbnail(file_path)?;
    let video_metadata = fs::metadata(file_path)?;
    let upload_url = get_upload_url(&token, video_metadata.len(), thumbnail.len() as u64)?;
    let mut video = Vec::new();
    fs::File::open(&file_path)?.read_to_end(&mut video)?;
    metadata_spinner();

    let upload_spinner = get_spinner("Uploading files...".to_string());
    put_file(&token, &upload_url.video_upload_url, video)?;
    put_file(&token, &upload_url.thumbnail_upload_url, thumbnail)?;

    let file_name = format!("{}", Path::new(&file_path).file_name().unwrap().display());
    let duration_raw = Command::new(ffprobe_sidecar_path().unwrap())
        .args(["-i", &file_path])
        .args(["-show_entries", "format=duration"])
        .args(["-v", "error"])
        .args(["-of", "default=noprint_wrappers=1:nokey=1"])
        .output()?;
    let duration = String::from_utf8_lossy(&duration_raw.stdout)
        .split(".")
        .next()
        .unwrap()
        .parse::<u64>()
        .map_err(|e| Error::new(ErrorKind::Other, e))?;
    let insert_payload = InsertPayload {
        video_id: upload_url.video_id,
        file_name: file_name,
        duration: duration,
        size: video_metadata.len(),
    };

    let insert_response = HTTP_CLIENT
        .post(format!("{}/api/insert", *config::API_ENDPOINT))
        .header("token", &token)
        .json(&serde_json::json!(insert_payload))
        .send()
        .map_err(|e| {
            if e.is_timeout() {
                Error::new(ErrorKind::TimedOut, "Request timed out")
            } else if e.is_connect() {
                Error::new(ErrorKind::NotConnected, "Cannot connect to server")
            } else {
                Error::new(ErrorKind::Other, format!("Error: {}", e))
            }
        })?;
    upload_spinner();

    match insert_response.status() {
        StatusCode::OK => {
            let raw_body = insert_response.text().unwrap_or_default();
            let body = serde_json::from_str::<InsertUrlResponse>(&raw_body)
                .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to wrap JSON: {}", e)))?;
            let mut clipboard = Clipboard::new().unwrap();
            clipboard.set_text(&body.video_url).unwrap();
            println!("Link copied to clipboard");
            println!("{}", &body.video_url);
        }
        e => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Error inserting video: {}", e),
            ));
        }
    }

    Ok("".to_string())
}
