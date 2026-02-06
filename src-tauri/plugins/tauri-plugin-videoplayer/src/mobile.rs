use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PluginApi, PluginHandle},
  AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_videoplayer);

pub fn init<R: Runtime, C: DeserializeOwned>(
  _app: &AppHandle<R>,
  api: PluginApi<R, C>,
) -> crate::Result<Videoplayer<R>> {
  #[cfg(target_os = "android")]
  let handle = api.register_android_plugin("com.yeonv.videoplayer", "ExamplePlugin")?;
  #[cfg(target_os = "ios")]
  let handle = api.register_ios_plugin(init_plugin_videoplayer)?;
  Ok(Videoplayer(handle))
}

pub struct Videoplayer<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Videoplayer<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    self
      .0
      .run_mobile_plugin("ping", payload)
      .map_err(Into::into)
  }

  pub fn play_video(&self, payload: PlayVideoRequest) -> crate::Result<PlayVideoResponse> {
    self
      .0
      .run_mobile_plugin("playVideo", payload)
      .map_err(Into::into)
  }

  pub fn play_live_video(&self, payload: PlayLiveVideoRequest) -> crate::Result<PlayLiveVideoResponse> {
    self
      .0
      .run_mobile_plugin("playLiveVideo", payload)
      .map_err(Into::into)
  }

  pub fn force_focus(&self, payload: ForceFocusRequest) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("forceFocus", payload)
            .map_err(Into::into)
    }
}
