package com.yeonv.videoplayer

import android.net.Uri
import android.os.Bundle
import android.util.Log
import androidx.appcompat.app.AppCompatActivity
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.MimeTypes
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.PlayerView

class VideoActivity : AppCompatActivity() {
    companion object {
        const val VIDEO_PATH_EXTRA = "videoPath"
        const val SUBTITLE_URL_EXTRA = "subtitleUrl"
    }

    // 1. Declare Player and View variables
    private var player: ExoPlayer? = null
    private lateinit var playerView: PlayerView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_video)

        // Find the PlayerView from our layout
        playerView = findViewById(R.id.player_view)

        // Make the activity fullscreen
        hideSystemUI()
    }

    // 2. Initialize the player in onStart()
    override fun onStart() {
        super.onStart()
        initializePlayer()
    }

    // 3. Release the player in onStop()
    override fun onStop() {
        super.onStop()
        releasePlayer()
    }

    private fun initializePlayer() {
        // Get the video URL passed from our plugin
        val videoUrl = intent.getStringExtra(VIDEO_PATH_EXTRA)
        if (videoUrl == null) {
            finish()
            return
        }

        val subtitleUrl = intent.getStringExtra(SUBTITLE_URL_EXTRA)
        Log.d("VideoActivity", "Video URL: $videoUrl, Subtitle URL: $subtitleUrl")

        // Create an ExoPlayer instance
        player = ExoPlayer.Builder(this).build()

        // Attach the player to the view
        playerView.player = player

        // Build the MediaItem with optional subtitle
        val mediaItemBuilder = MediaItem.Builder().setUri(videoUrl)

        if (subtitleUrl != null) {
            val subtitleConfig = MediaItem.SubtitleConfiguration.Builder(Uri.parse(subtitleUrl))
                .setMimeType(MimeTypes.TEXT_VTT)
                .setLanguage("en")
                .setSelectionFlags(C.SELECTION_FLAG_DEFAULT)
                .build()
            mediaItemBuilder.setSubtitleConfigurations(listOf(subtitleConfig))
            Log.d("VideoActivity", "Subtitle configuration added")
        }

        val mediaItem = mediaItemBuilder.build()

        // Set the media item to be played
        player?.setMediaItem(mediaItem)

        // Prepare the player
        player?.prepare()

        // Start playback automatically when ready
        player?.playWhenReady = true
    }

    private fun releasePlayer() {
        player?.let {
            it.release() // This is a crucial step to free up resources.
            player = null
        }
    }

    private fun hideSystemUI() {
        WindowCompat.setDecorFitsSystemWindows(window, false)
        WindowInsetsControllerCompat(window, window.decorView).let { controller ->
            controller.hide(WindowInsetsCompat.Type.systemBars())
            controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        }
    }
}
