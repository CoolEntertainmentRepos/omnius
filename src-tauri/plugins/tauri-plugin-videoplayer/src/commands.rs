use tauri::{AppHandle, command, Runtime};

use crate::models::*;
use crate::Result;
use crate::VideoplayerExt;

#[command]
pub(crate) async fn ping<R: Runtime>(
    app: AppHandle<R>,
    payload: PingRequest,
) -> Result<PingResponse> {
    app.videoplayer().ping(payload)
}

#[command]
pub(crate) async fn play_video<R: Runtime>(
    app: AppHandle<R>,
    payload: PlayVideoRequest,
) -> Result<PlayVideoResponse> {
    app.videoplayer().play_video(payload)
}

#[command]
pub(crate) async fn play_live_video<R: Runtime>(
    app: AppHandle<R>,
    payload: PlayLiveVideoRequest,
) -> Result<PlayLiveVideoResponse> {
    app.videoplayer().play_live_video(payload)
}

#[command]
pub(crate) async fn force_focus<R: Runtime>(
    app: AppHandle<R>,
    payload: ForceFocusRequest,
) -> Result<()> {
    app.videoplayer().force_focus(payload)
}
