package com.yeonv.videoplayer

import android.net.Uri
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.util.Log
import android.view.KeyEvent
import android.view.View
import android.widget.ImageButton
import android.widget.ProgressBar
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.PlayerView
import app.tauri.plugin.JSObject

class LivePlayerActivity : AppCompatActivity() {
    companion object {
        const val STREAM_URL_EXTRA = "streamUrl"
        const val TITLE_EXTRA = "title"
        private const val TAG = "LivePlayerActivity"
        private const val CONTROLS_HIDE_DELAY = 4000L
    }

    private var player: ExoPlayer? = null
    private lateinit var playerView: PlayerView
    private lateinit var controlsOverlay: View
    private lateinit var btnBack: ImageButton
    private lateinit var txtTitle: TextView
    private lateinit var bufferingSpinner: ProgressBar
    private val handler = Handler(Looper.getMainLooper())

    private var controlsVisible = true
    private var hideControlsRunnable: Runnable? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        window.addFlags(android.view.WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        setContentView(R.layout.activity_live_player)

        playerView = findViewById(R.id.live_player_view)
        controlsOverlay = findViewById(R.id.live_controls_overlay)
        btnBack = findViewById(R.id.live_btn_back)
        txtTitle = findViewById(R.id.live_txt_title)
        bufferingSpinner = findViewById(R.id.live_buffering_spinner)

        val title = intent.getStringExtra(TITLE_EXTRA) ?: ""
        txtTitle.text = title

        btnBack.setOnClickListener { finishPlayer() }

        hideSystemUI()
        scheduleHideControls()
    }

    override fun onStart() {
        super.onStart()
        initializePlayer()
    }

    override fun onStop() {
        super.onStop()
        releasePlayer()
    }

    private fun initializePlayer() {
        val streamUrl = intent.getStringExtra(STREAM_URL_EXTRA)
        if (streamUrl == null) { finish(); return }

        Log.d(TAG, "Stream URL: $streamUrl")

        player = ExoPlayer.Builder(this).build()
        playerView.player = player

        val mediaItem = MediaItem.Builder().setUri(Uri.parse(streamUrl)).build()

        player?.apply {
            setMediaItem(mediaItem)

            addListener(object : Player.Listener {
                override fun onPlaybackStateChanged(state: Int) {
                    bufferingSpinner.visibility =
                        if (state == Player.STATE_BUFFERING) View.VISIBLE else View.GONE

                    if (state == Player.STATE_ENDED) finishPlayer()
                }

                override fun onIsPlayingChanged(isPlaying: Boolean) {
                    if (isPlaying) scheduleHideControls() else showControls()
                }

                override fun onPlayerError(error: androidx.media3.common.PlaybackException) {
                    Log.e(TAG, "Player error: ${error.message}")
                    val data = JSObject()
                    data.put("error", error.message ?: "Unknown error")
                    ExamplePlugin.instance?.emitEvent("livePlayerError", data)
                }
            })

            prepare()
            playWhenReady = true
        }
    }

    // ---- Controls ----

    private fun showControls() {
        if (!controlsVisible) {
            controlsVisible = true
            controlsOverlay.visibility = View.VISIBLE
            controlsOverlay.alpha = 0f
            controlsOverlay.animate().alpha(1f).setDuration(200).start()
        }
        scheduleHideControls()
    }

    private fun hideControls() {
        if (controlsVisible && player?.isPlaying == true) {
            controlsVisible = false
            controlsOverlay.animate().alpha(0f).setDuration(300).withEndAction {
                controlsOverlay.visibility = View.GONE
            }.start()
        }
    }

    private fun scheduleHideControls() {
        hideControlsRunnable?.let { handler.removeCallbacks(it) }
        hideControlsRunnable = Runnable { hideControls() }
        handler.postDelayed(hideControlsRunnable!!, CONTROLS_HIDE_DELAY)
    }

    // ---- D-Pad ----

    override fun dispatchKeyEvent(event: KeyEvent): Boolean {
        if (event.action != KeyEvent.ACTION_DOWN) return super.dispatchKeyEvent(event)

        when (event.keyCode) {
            KeyEvent.KEYCODE_BACK -> {
                finishPlayer()
                return true
            }
            KeyEvent.KEYCODE_DPAD_CENTER, KeyEvent.KEYCODE_ENTER -> {
                player?.let { if (it.isPlaying) it.pause() else it.play() }
                showControls()
                return true
            }
            KeyEvent.KEYCODE_MEDIA_PLAY_PAUSE -> {
                player?.let { if (it.isPlaying) it.pause() else it.play() }
                return true
            }
            KeyEvent.KEYCODE_MEDIA_PLAY -> { player?.play(); return true }
            KeyEvent.KEYCODE_MEDIA_PAUSE -> { player?.pause(); return true }
            else -> {
                if (!controlsVisible) {
                    showControls()
                    return true
                }
            }
        }

        return super.dispatchKeyEvent(event)
    }

    // ---- Lifecycle ----

    private fun finishPlayer() {
        ExamplePlugin.instance?.resolveLivePlayback()
        finish()
    }

    @Deprecated("Deprecated in Java")
    override fun onBackPressed() {
        finishPlayer()
    }

    private fun releasePlayer() {
        hideControlsRunnable?.let { handler.removeCallbacks(it) }
        player?.release()
        player = null
    }

    private fun hideSystemUI() {
        WindowCompat.setDecorFitsSystemWindows(window, false)
        WindowInsetsControllerCompat(window, window.decorView).let { controller ->
            controller.hide(WindowInsetsCompat.Type.systemBars())
            controller.systemBarsBehavior =
                WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        }
    }
}
