package com.yeonv.videoplayer

import android.app.Activity
import android.content.Intent
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

@InvokeArg
class PingArgs {
  var value: String? = null
}

@InvokeArg
class SubtitleTrackArg {
  lateinit var url: String
  var language: String = "en"
  var label: String = "Subtitles"
  var mimeType: String = "text/vtt"
}

@InvokeArg
class PlayVideoArgs {
  lateinit var path: String
  var subtitleUrl: String? = null
  var subtitles: Array<SubtitleTrackArg>? = null
  var startPosition: Long? = null
  var title: String? = null
}

@InvokeArg
class PlayLiveVideoArgs {
  lateinit var url: String
  var title: String? = null
}

@InvokeArg
class ForceFocusArgs {
  lateinit var mainActivityClassName: String
}

@TauriPlugin
class ExamplePlugin(private val activity: Activity): Plugin(activity) {
  private val implementation = Example()

  companion object {
    var instance: ExamplePlugin? = null
    // Pending invoke stored here so VideoActivity can resolve it when finishing
    var pendingInvoke: Invoke? = null
    var pendingLiveInvoke: Invoke? = null
  }

  override fun load(webView: android.webkit.WebView) {
    super.load(webView)
    instance = this
  }

  fun emitEvent(eventName: String, data: JSObject) {
    trigger(eventName, data)
  }

  /**
   * Called by VideoActivity when it finishes to resolve the pending playVideo invoke.
   * Position data is already emitted via positionUpdate events - resolve with no data
   * to stay compatible with the Rust side expecting unit return.
   */
  fun resolvePlayVideo(lastPosition: Long, duration: Long) {
    // Emit final position as event (JS side saves it)
    val data = JSObject()
    data.put("position", lastPosition)
    data.put("duration", duration)
    data.put("final", true)
    trigger("positionUpdate", data)

    // Resolve with position data (Rust expects PlayVideoResponse)
    val result = JSObject()
    result.put("lastPosition", lastPosition)
    result.put("duration", duration)
    pendingInvoke?.resolve(result)
    pendingInvoke = null
  }

  @Command
  fun ping(invoke: Invoke) {
      val args = invoke.parseArgs(PingArgs::class.java)
      val ret = JSObject()
      ret.put("value", implementation.pong(args.value ?: "default value :("))
      invoke.resolve(ret)
  }

  @Command
  fun playVideo(invoke: Invoke) {
    val args = invoke.parseArgs(PlayVideoArgs::class.java)

    val intent = Intent(activity, VideoActivity::class.java).apply {
        putExtra(VideoActivity.VIDEO_PATH_EXTRA, args.path)
        args.subtitleUrl?.let { putExtra(VideoActivity.SUBTITLE_URL_EXTRA, it) }
        args.title?.let { putExtra(VideoActivity.TITLE_EXTRA, it) }
        args.startPosition?.let { putExtra(VideoActivity.START_POSITION_EXTRA, it) }

        // Pass subtitle tracks as parallel arrays
        args.subtitles?.let { subs ->
            putExtra(VideoActivity.SUBTITLE_URLS_EXTRA, subs.map { it.url }.toTypedArray())
            putExtra(VideoActivity.SUBTITLE_LANGUAGES_EXTRA, subs.map { it.language }.toTypedArray())
            putExtra(VideoActivity.SUBTITLE_LABELS_EXTRA, subs.map { it.label }.toTypedArray())
            putExtra(VideoActivity.SUBTITLE_MIMETYPES_EXTRA, subs.map { it.mimeType }.toTypedArray())
        }
    }

    // Store invoke in companion - VideoActivity will call resolvePlayVideo() when it finishes
    pendingInvoke = invoke

    activity.startActivity(intent)
  }

  fun resolveLivePlayback() {
    val result = JSObject()
    result.put("finished", true)
    pendingLiveInvoke?.resolve(result)
    pendingLiveInvoke = null
  }

  @Command
  fun playLiveVideo(invoke: Invoke) {
    val args = invoke.parseArgs(PlayLiveVideoArgs::class.java)

    val intent = Intent(activity, LivePlayerActivity::class.java).apply {
      putExtra(LivePlayerActivity.STREAM_URL_EXTRA, args.url)
      args.title?.let { putExtra(LivePlayerActivity.TITLE_EXTRA, it) }
    }

    pendingLiveInvoke = invoke
    activity.startActivity(intent)
  }

  @Command
  fun forceFocus(invoke: Invoke) {
    try {
      val args = invoke.parseArgs(ForceFocusArgs::class.java)
      val mainActivityClass = Class.forName(args.mainActivityClassName)
      val intent = Intent(activity, mainActivityClass).apply {
        addFlags(Intent.FLAG_ACTIVITY_REORDER_TO_FRONT)
      }
      activity.startActivity(intent)
      Log.d("VideoplayerPlugin", "Forcing focus to main activity: ${args.mainActivityClassName}")
      invoke.resolve()
    } catch (e: Exception) {
      invoke.reject("Failed to force focus: ${e.message}")
    }
  }
}
