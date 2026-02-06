package com.yeonv.videoplayer

import android.app.Activity
import android.content.Intent
import android.content.SharedPreferences
import android.net.Uri
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.util.Log
import android.view.KeyEvent
import android.view.View
import android.view.animation.DecelerateInterpolator
import android.widget.Button
import android.widget.ImageButton
import android.widget.LinearLayout
import android.widget.ProgressBar
import android.widget.SeekBar
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.MimeTypes
import androidx.media3.common.PlaybackParameters
import androidx.media3.common.Player
import androidx.media3.common.TrackSelectionOverride
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.PlayerView
import app.tauri.plugin.JSObject

class VideoActivity : AppCompatActivity() {
    companion object {
        const val VIDEO_PATH_EXTRA = "videoPath"
        const val SUBTITLE_URL_EXTRA = "subtitleUrl"
        const val TITLE_EXTRA = "title"
        const val START_POSITION_EXTRA = "startPosition"
        const val SUBTITLE_URLS_EXTRA = "subtitleUrls"
        const val SUBTITLE_LANGUAGES_EXTRA = "subtitleLanguages"
        const val SUBTITLE_LABELS_EXTRA = "subtitleLabels"
        const val SUBTITLE_MIMETYPES_EXTRA = "subtitleMimeTypes"

        private const val TAG = "VideoActivity"
        private const val POSITION_UPDATE_INTERVAL_MS = 1000L
        private const val PREFS_NAME = "omnius_player_prefs"
        private const val CONTROLS_HIDE_DELAY = 5000L
        private const val SEEK_INCREMENT_MS = 10000L
    }

    // Player
    private var player: ExoPlayer? = null
    private lateinit var playerView: PlayerView
    private var lastPosition: Long = 0
    private var videoDuration: Long = 0
    private val handler = Handler(Looper.getMainLooper())
    private var positionUpdateRunnable: Runnable? = null
    private lateinit var prefs: SharedPreferences

    // Controls
    private lateinit var controlsOverlay: View
    private lateinit var btnBack: ImageButton
    private lateinit var txtTitle: TextView
    private lateinit var btnPlayPause: ImageButton
    private lateinit var btnRewind: ImageButton
    private lateinit var btnForward: ImageButton
    private lateinit var seekBar: SeekBar
    private lateinit var txtCurrentTime: TextView
    private lateinit var txtDuration: TextView
    private lateinit var btnSpeed: Button
    private lateinit var btnSubtitle: Button
    private lateinit var bufferingSpinner: ProgressBar

    // Panels
    private lateinit var subtitlePanel: LinearLayout
    private lateinit var subtitleList: LinearLayout
    private lateinit var btnSubtitleOff: Button
    private lateinit var speedPanel: LinearLayout
    private lateinit var speedList: LinearLayout

    // State
    private var controlsVisible = true
    private var hideControlsRunnable: Runnable? = null
    private var seekBarTracking = false

    // Speed
    private val speedOptions = floatArrayOf(0.5f, 0.75f, 1.0f, 1.25f, 1.5f, 2.0f)
    private val speedLabels = arrayOf("0.5x", "0.75x", "1x", "1.25x", "1.5x", "2x")
    private var currentSpeedIndex = 2

    // Subtitle labels for picker
    private var subtitleLabelsArray: Array<String>? = null
    private var currentSubtitleIndex = 0

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_video)

        prefs = getSharedPreferences(PREFS_NAME, MODE_PRIVATE)

        // Bind views
        playerView = findViewById(R.id.player_view)
        controlsOverlay = findViewById(R.id.controls_overlay)
        btnBack = findViewById(R.id.btn_back)
        txtTitle = findViewById(R.id.txt_title)
        btnPlayPause = findViewById(R.id.btn_play_pause)
        btnRewind = findViewById(R.id.btn_rewind)
        btnForward = findViewById(R.id.btn_forward)
        seekBar = findViewById(R.id.seek_bar)
        txtCurrentTime = findViewById(R.id.txt_current_time)
        txtDuration = findViewById(R.id.txt_duration)
        btnSpeed = findViewById(R.id.btn_speed)
        btnSubtitle = findViewById(R.id.btn_subtitle)
        bufferingSpinner = findViewById(R.id.buffering_spinner)
        subtitlePanel = findViewById(R.id.subtitle_panel)
        subtitleList = findViewById(R.id.subtitle_list)
        btnSubtitleOff = findViewById(R.id.btn_subtitle_off)
        speedPanel = findViewById(R.id.speed_panel)
        speedList = findViewById(R.id.speed_list)

        // Set title
        val title = intent.getStringExtra(TITLE_EXTRA) ?: ""
        txtTitle.text = title

        // Setup button listeners
        btnBack.setOnClickListener { finishWithResult() }
        btnPlayPause.setOnClickListener { togglePlayPause() }
        btnRewind.setOnClickListener { seekRelative(-SEEK_INCREMENT_MS) }
        btnForward.setOnClickListener { seekRelative(SEEK_INCREMENT_MS) }
        btnSpeed.setOnClickListener { toggleSpeedPanel() }
        btnSubtitle.setOnClickListener { toggleSubtitlePanel() }
        btnSubtitleOff.setOnClickListener { disableSubtitles(); closeSubtitlePanel() }

        // Setup seek bar
        seekBar.setOnSeekBarChangeListener(object : SeekBar.OnSeekBarChangeListener {
            override fun onProgressChanged(sb: SeekBar?, progress: Int, fromUser: Boolean) {
                if (fromUser) {
                    txtCurrentTime.text = formatTime(progress.toLong())
                }
            }
            override fun onStartTrackingTouch(sb: SeekBar?) { seekBarTracking = true }
            override fun onStopTrackingTouch(sb: SeekBar?) {
                seekBarTracking = false
                sb?.let { player?.seekTo(it.progress.toLong()) }
            }
        })

        // Build speed picker
        buildSpeedPicker()

        hideSystemUI()
        scheduleHideControls()
    }

    override fun onStart() {
        super.onStart()
        initializePlayer()
    }

    override fun onStop() {
        super.onStop()
        savePosition()
        releasePlayer()
    }

    private fun initializePlayer() {
        val videoUrl = intent.getStringExtra(VIDEO_PATH_EXTRA)
        if (videoUrl == null) { finish(); return }

        val startPosition = intent.getLongExtra(START_POSITION_EXTRA, 0L)
        Log.d(TAG, "Video URL: $videoUrl, StartPosition: $startPosition")

        player = ExoPlayer.Builder(this).build()
        playerView.player = player

        // Build subtitle configurations
        val subtitleConfigs = mutableListOf<MediaItem.SubtitleConfiguration>()

        val subtitleUrls = intent.getStringArrayExtra(SUBTITLE_URLS_EXTRA)
        val subtitleLanguages = intent.getStringArrayExtra(SUBTITLE_LANGUAGES_EXTRA)
        val labels = intent.getStringArrayExtra(SUBTITLE_LABELS_EXTRA)
        val subtitleMimeTypes = intent.getStringArrayExtra(SUBTITLE_MIMETYPES_EXTRA)

        subtitleLabelsArray = labels

        if (subtitleUrls != null && subtitleUrls.isNotEmpty()) {
            for (i in subtitleUrls.indices) {
                val subUrl = subtitleUrls[i]
                val lang = subtitleLanguages?.getOrNull(i) ?: "en"
                val label = labels?.getOrNull(i) ?: "Track ${i + 1}"
                val mimeType = resolveMimeType(subtitleMimeTypes?.getOrNull(i) ?: "text/vtt")

                val config = MediaItem.SubtitleConfiguration.Builder(Uri.parse(subUrl))
                    .setMimeType(mimeType)
                    .setLanguage(lang)
                    .setLabel(label)
                    .apply { if (i == 0) setSelectionFlags(C.SELECTION_FLAG_DEFAULT) }
                    .build()
                subtitleConfigs.add(config)
                Log.d(TAG, "Subtitle [$i]: $label ($lang) - $mimeType")
            }
        } else {
            val subtitleUrl = intent.getStringExtra(SUBTITLE_URL_EXTRA)
            if (subtitleUrl != null) {
                val config = MediaItem.SubtitleConfiguration.Builder(Uri.parse(subtitleUrl))
                    .setMimeType(MimeTypes.TEXT_VTT)
                    .setLanguage("en")
                    .setLabel("English")
                    .setSelectionFlags(C.SELECTION_FLAG_DEFAULT)
                    .build()
                subtitleConfigs.add(config)
                subtitleLabelsArray = arrayOf("English")
            }
        }

        buildSubtitlePicker()

        val mediaItem = MediaItem.Builder()
            .setUri(videoUrl)
            .apply {
                if (subtitleConfigs.isNotEmpty()) {
                    setSubtitleConfigurations(subtitleConfigs)
                }
            }
            .build()

        player?.apply {
            setMediaItem(mediaItem)

            addListener(object : Player.Listener {
                override fun onPlaybackStateChanged(state: Int) {
                    val stateName = when (state) {
                        Player.STATE_IDLE -> "idle"
                        Player.STATE_BUFFERING -> "buffering"
                        Player.STATE_READY -> "ready"
                        Player.STATE_ENDED -> "ended"
                        else -> "unknown"
                    }
                    Log.d(TAG, "Playback state: $stateName")

                    bufferingSpinner.visibility = if (state == Player.STATE_BUFFERING) View.VISIBLE else View.GONE

                    val data = JSObject()
                    data.put("state", stateName)
                    ExamplePlugin.instance?.emitEvent("playbackState", data)

                    if (state == Player.STATE_ENDED) finishWithResult()
                    if (state == Player.STATE_READY) updateDuration()
                }

                override fun onIsPlayingChanged(isPlaying: Boolean) {
                    updatePlayPauseIcon()
                    val data = JSObject()
                    data.put("state", if (isPlaying) "playing" else "paused")
                    ExamplePlugin.instance?.emitEvent("playbackState", data)

                    if (isPlaying) scheduleHideControls() else showControls()
                }

                override fun onPlayerError(error: androidx.media3.common.PlaybackException) {
                    Log.e(TAG, "Player error: ${error.message}")
                    val data = JSObject()
                    data.put("error", error.message ?: "Unknown error")
                    data.put("errorCode", error.errorCode)
                    ExamplePlugin.instance?.emitEvent("playerError", data)
                }
            })

            prepare()

            val resumePosition = if (startPosition > 0) startPosition
            else prefs.getLong("pos_${videoUrl.hashCode()}", 0L)

            if (resumePosition > 0) {
                seekTo(resumePosition)
                Log.d(TAG, "Resuming from: ${resumePosition}ms")
            }

            playWhenReady = true
        }

        startPositionUpdates()
    }

    // ---- Controls Visibility ----

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
            closeSubtitlePanel()
            closeSpeedPanel()
        }
    }

    private fun scheduleHideControls() {
        hideControlsRunnable?.let { handler.removeCallbacks(it) }
        hideControlsRunnable = Runnable { hideControls() }
        handler.postDelayed(hideControlsRunnable!!, CONTROLS_HIDE_DELAY)
    }

    // ---- D-Pad Key Handling ----

    override fun dispatchKeyEvent(event: KeyEvent): Boolean {
        if (event.action != KeyEvent.ACTION_DOWN) return super.dispatchKeyEvent(event)

        // When a panel is open, trap focus within it
        val panelOpen = subtitlePanel.visibility == View.VISIBLE || speedPanel.visibility == View.VISIBLE
        if (panelOpen) {
            when (event.keyCode) {
                KeyEvent.KEYCODE_BACK -> {
                    closeSubtitlePanel()
                    closeSpeedPanel()
                    return true
                }
                // Let Android handle up/down navigation within the panel's LinearLayout
                KeyEvent.KEYCODE_DPAD_UP, KeyEvent.KEYCODE_DPAD_DOWN,
                KeyEvent.KEYCODE_DPAD_CENTER, KeyEvent.KEYCODE_ENTER -> {
                    return super.dispatchKeyEvent(event)
                }
                // Block left/right from escaping the panel
                KeyEvent.KEYCODE_DPAD_LEFT, KeyEvent.KEYCODE_DPAD_RIGHT -> {
                    return true
                }
            }
            return super.dispatchKeyEvent(event)
        }

        if (!controlsVisible) {
            showControls()
            if (event.keyCode != KeyEvent.KEYCODE_BACK) {
                btnPlayPause.requestFocus()
                return true
            }
        } else {
            scheduleHideControls()
        }

        when (event.keyCode) {
            KeyEvent.KEYCODE_DPAD_CENTER, KeyEvent.KEYCODE_ENTER -> {
                val focused = currentFocus
                if (focused == null || focused == playerView || focused == controlsOverlay) {
                    togglePlayPause()
                    return true
                }
            }
            KeyEvent.KEYCODE_DPAD_LEFT -> {
                val focused = currentFocus
                if (focused == seekBar || focused == null || focused == playerView) {
                    seekRelative(-SEEK_INCREMENT_MS)
                    return true
                }
            }
            KeyEvent.KEYCODE_DPAD_RIGHT -> {
                val focused = currentFocus
                if (focused == seekBar || focused == null || focused == playerView) {
                    seekRelative(SEEK_INCREMENT_MS)
                    return true
                }
            }
            KeyEvent.KEYCODE_BACK -> {
                if (controlsVisible) {
                    finishWithResult()
                    return true
                }
            }
            KeyEvent.KEYCODE_MEDIA_PLAY_PAUSE -> { togglePlayPause(); return true }
            KeyEvent.KEYCODE_MEDIA_PLAY -> { player?.play(); return true }
            KeyEvent.KEYCODE_MEDIA_PAUSE -> { player?.pause(); return true }
            KeyEvent.KEYCODE_MEDIA_FAST_FORWARD -> { seekRelative(SEEK_INCREMENT_MS); return true }
            KeyEvent.KEYCODE_MEDIA_REWIND -> { seekRelative(-SEEK_INCREMENT_MS); return true }
        }

        return super.dispatchKeyEvent(event)
    }

    // ---- Playback Controls ----

    private fun togglePlayPause() {
        player?.let { if (it.isPlaying) it.pause() else it.play() }
        updatePlayPauseIcon()
    }

    private fun updatePlayPauseIcon() {
        val isPlaying = player?.isPlaying == true
        btnPlayPause.setImageResource(
            if (isPlaying) android.R.drawable.ic_media_pause
            else android.R.drawable.ic_media_play
        )
    }

    private fun seekRelative(deltaMs: Long) {
        player?.let {
            val newPos = (it.currentPosition + deltaMs).coerceIn(0, it.duration.coerceAtLeast(0))
            it.seekTo(newPos)
            txtCurrentTime.text = formatTime(newPos)
            if (!seekBarTracking) seekBar.progress = newPos.toInt()
        }
        showControls()
    }

    private fun updateDuration() {
        player?.let {
            val dur = if (it.duration == C.TIME_UNSET) 0L else it.duration
            videoDuration = dur
            seekBar.max = dur.toInt()
            txtDuration.text = formatTime(dur)
        }
    }

    // ---- Speed Picker ----

    private fun buildSpeedPicker() {
        speedList.removeAllViews()
        for (i in speedOptions.indices) {
            val btn = Button(this).apply {
                text = speedLabels[i]
                setTextColor(resources.getColor(R.color.omnius_text_primary, null))
                textSize = 16f
                setBackgroundResource(R.drawable.panel_item_focus)
                isFocusable = true
                minimumHeight = (48 * resources.displayMetrics.density).toInt()
                setPadding((16 * resources.displayMetrics.density).toInt(), 0,
                    (16 * resources.displayMetrics.density).toInt(), 0)
                gravity = android.view.Gravity.START or android.view.Gravity.CENTER_VERTICAL
                isSelected = i == currentSpeedIndex
                setOnClickListener { selectSpeed(i) }
            }
            speedList.addView(btn)
        }
    }

    private fun selectSpeed(index: Int) {
        currentSpeedIndex = index
        val speed = speedOptions[index]
        player?.playbackParameters = PlaybackParameters(speed)
        btnSpeed.text = speedLabels[index]

        for (i in 0 until speedList.childCount) {
            speedList.getChildAt(i).isSelected = i == index
        }

        val data = JSObject()
        data.put("speed", speed.toDouble())
        ExamplePlugin.instance?.emitEvent("speedChanged", data)

        closeSpeedPanel()
        Log.d(TAG, "Speed: ${speed}x")
    }

    private fun toggleSpeedPanel() {
        if (speedPanel.visibility == View.VISIBLE) closeSpeedPanel()
        else {
            closeSubtitlePanel()
            speedPanel.visibility = View.VISIBLE
            // Post focus request after layout so the view is focusable
            speedPanel.post { speedList.getChildAt(currentSpeedIndex)?.requestFocus() }
        }
    }

    private fun closeSpeedPanel() {
        speedPanel.visibility = View.GONE
        btnSpeed.requestFocus()
    }

    // ---- Subtitle Picker ----

    private fun buildSubtitlePicker() {
        subtitleList.removeAllViews()
        val labels = subtitleLabelsArray ?: return

        for (i in labels.indices) {
            val btn = Button(this).apply {
                text = labels[i]
                setTextColor(resources.getColor(R.color.omnius_text_primary, null))
                textSize = 16f
                setBackgroundResource(R.drawable.panel_item_focus)
                isFocusable = true
                minimumHeight = (48 * resources.displayMetrics.density).toInt()
                setPadding((16 * resources.displayMetrics.density).toInt(), 0,
                    (16 * resources.displayMetrics.density).toInt(), 0)
                gravity = android.view.Gravity.START or android.view.Gravity.CENTER_VERTICAL
                isSelected = i == currentSubtitleIndex
                setOnClickListener { selectSubtitle(i) }
            }
            subtitleList.addView(btn)
        }
    }

    private fun selectSubtitle(index: Int) {
        currentSubtitleIndex = index
        player?.let { p ->
            // Re-enable text tracks first
            p.trackSelectionParameters = p.trackSelectionParameters.buildUpon()
                .setTrackTypeDisabled(C.TRACK_TYPE_TEXT, false)
                .build()

            val textGroups = p.currentTracks.groups.filter { g -> g.type == C.TRACK_TYPE_TEXT }
            if (index < textGroups.size) {
                val group = textGroups[index]
                val override = TrackSelectionOverride(group.mediaTrackGroup, 0)
                p.trackSelectionParameters = p.trackSelectionParameters.buildUpon()
                    .setOverrideForType(override)
                    .build()
            }
        }

        btnSubtitleOff.isSelected = false
        for (i in 0 until subtitleList.childCount) {
            subtitleList.getChildAt(i).isSelected = i == index
        }

        val label = subtitleLabelsArray?.getOrNull(index) ?: "Track $index"
        val data = JSObject()
        data.put("language", label)
        data.put("label", label)
        ExamplePlugin.instance?.emitEvent("subtitleChanged", data)

        closeSubtitlePanel()
        Log.d(TAG, "Subtitle: $label")
    }

    private fun disableSubtitles() {
        currentSubtitleIndex = -1
        player?.let { p ->
            p.trackSelectionParameters = p.trackSelectionParameters.buildUpon()
                .setTrackTypeDisabled(C.TRACK_TYPE_TEXT, true)
                .build()
        }
        btnSubtitleOff.isSelected = true
        for (i in 0 until subtitleList.childCount) {
            subtitleList.getChildAt(i).isSelected = false
        }

        val data = JSObject()
        data.put("language", "off")
        data.put("label", "Off")
        ExamplePlugin.instance?.emitEvent("subtitleChanged", data)
    }

    private fun toggleSubtitlePanel() {
        if (subtitlePanel.visibility == View.VISIBLE) closeSubtitlePanel()
        else {
            closeSpeedPanel()
            subtitlePanel.visibility = View.VISIBLE
            subtitlePanel.translationX = 320f * resources.displayMetrics.density
            subtitlePanel.animate()
                .translationX(0f)
                .setDuration(250)
                .setInterpolator(DecelerateInterpolator())
                .start()
            // Post focus request after animation starts so view is laid out
            subtitlePanel.post { btnSubtitleOff.requestFocus() }
        }
    }

    private fun closeSubtitlePanel() {
        if (subtitlePanel.visibility == View.VISIBLE) {
            subtitlePanel.animate()
                .translationX(320f * resources.displayMetrics.density)
                .setDuration(200)
                .withEndAction { subtitlePanel.visibility = View.GONE }
                .start()
            btnSubtitle.requestFocus()
        }
    }

    // ---- Position Updates ----

    private fun startPositionUpdates() {
        positionUpdateRunnable = object : Runnable {
            override fun run() {
                player?.let { p ->
                    lastPosition = p.currentPosition
                    videoDuration = if (p.duration == C.TIME_UNSET) 0L else p.duration

                    if (!seekBarTracking) {
                        seekBar.progress = lastPosition.toInt()
                        seekBar.secondaryProgress = p.bufferedPosition.toInt()
                        txtCurrentTime.text = formatTime(lastPosition)
                    }

                    val data = JSObject()
                    data.put("position", p.currentPosition)
                    data.put("duration", videoDuration)
                    data.put("bufferedPosition", p.bufferedPosition)
                    ExamplePlugin.instance?.emitEvent("positionUpdate", data)
                }
                handler.postDelayed(this, POSITION_UPDATE_INTERVAL_MS)
            }
        }
        handler.postDelayed(positionUpdateRunnable!!, POSITION_UPDATE_INTERVAL_MS)
    }

    private fun stopPositionUpdates() {
        positionUpdateRunnable?.let { handler.removeCallbacks(it) }
        positionUpdateRunnable = null
    }

    // ---- Lifecycle ----

    private fun savePosition() {
        val videoUrl = intent.getStringExtra(VIDEO_PATH_EXTRA) ?: return
        player?.let { p ->
            lastPosition = p.currentPosition
            videoDuration = if (p.duration == C.TIME_UNSET) 0L else p.duration
            prefs.edit().putLong("pos_${videoUrl.hashCode()}", lastPosition).apply()
            Log.d(TAG, "Saved position: ${lastPosition}ms")
        }
    }

    private fun finishWithResult() {
        // Resolve the pending invoke in ExamplePlugin directly
        ExamplePlugin.instance?.resolvePlayVideo(lastPosition, videoDuration)
        finish()
    }

    @Deprecated("Deprecated in Java")
    override fun onBackPressed() {
        finishWithResult()
    }

    private fun releasePlayer() {
        stopPositionUpdates()
        hideControlsRunnable?.let { handler.removeCallbacks(it) }
        player?.let {
            lastPosition = it.currentPosition
            videoDuration = if (it.duration == C.TIME_UNSET) 0L else it.duration
            it.release()
            player = null
        }
    }

    // ---- Helpers ----

    private fun resolveMimeType(mimeType: String): String {
        return when (mimeType.lowercase()) {
            "text/vtt", "vtt" -> MimeTypes.TEXT_VTT
            "application/x-subrip", "srt" -> MimeTypes.APPLICATION_SUBRIP
            "text/x-ssa", "text/x-ass", "ass", "ssa" -> MimeTypes.TEXT_SSA
            else -> MimeTypes.TEXT_VTT
        }
    }

    private fun formatTime(ms: Long): String {
        val totalSec = ms / 1000
        val h = totalSec / 3600
        val m = (totalSec % 3600) / 60
        val s = totalSec % 60
        return if (h > 0) String.format("%d:%02d:%02d", h, m, s)
        else String.format("%d:%02d", m, s)
    }

    private fun hideSystemUI() {
        WindowCompat.setDecorFitsSystemWindows(window, false)
        WindowInsetsControllerCompat(window, window.decorView).let { controller ->
            controller.hide(WindowInsetsCompat.Type.systemBars())
            controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        }
    }
}
