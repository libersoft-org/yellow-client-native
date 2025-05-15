// Conditional compilation based on platform
// Only include rodio-based implementation on non-Android/iOS platforms
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod audio_impl {
    use log::info;
    use rodio::{Decoder, OutputStream, Sink};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;
    use std::sync::{Arc, Mutex};
    use std::thread;

    // A simple audio player using rodio
    // Uses a global sink manager to keep track of audio playback
    lazy_static::lazy_static! {
        static ref AUDIO_PLAYERS: Mutex<HashMap<String, Arc<Mutex<Option<Sink>>>>> = Mutex::new(HashMap::new());
    }

    // Play an audio file with a given ID
    pub fn play_audio(file_path: String, id: Option<String>) -> Result<String, String> {
        let audio_id = id.unwrap_or_else(|| file_path.clone());
        let audio_id2 = audio_id.clone();

        info!("Playing audio file: {} with ID: {}", file_path, audio_id);

        // Check if file exists
        if !Path::new(&file_path).exists() {
            return Err(format!("Audio file not found: {}", file_path));
        }

        // Create a thread to handle audio playback
        thread::spawn(move || match play_audio_internal(&file_path, &audio_id) {
            Ok(_) => info!("Audio playback completed: {}", audio_id),
            Err(e) => info!("Audio playback error: {}", e),
        });

        Ok(audio_id2)
    }

    // Stop playback for a specific ID
    pub fn stop_audio(id: String) -> Result<(), String> {
        let players = AUDIO_PLAYERS
            .lock()
            .map_err(|e| format!("Failed to lock audio players: {}", e))?;
        info!("stop_audio: {}", id.clone());

        if let Some(sink_mutex) = players.get(&id) {
            if let Ok(mut sink_opt) = sink_mutex.lock() {
                if let Some(sink) = sink_opt.take() {
                    sink.stop();
                    info!("Stopped audio: {}", id);
                }
            }
        }

        Ok(())
    }

    // Check if audio is playing
    pub fn is_audio_playing(id: String) -> Result<bool, String> {
        let players = AUDIO_PLAYERS
            .lock()
            .map_err(|e| format!("Failed to lock audio players: {}", e))?;

        if let Some(sink_mutex) = players.get(&id) {
            if let Ok(sink_opt) = sink_mutex.lock() {
                return Ok(sink_opt.is_some());
            }
        }

        Ok(false)
    }

    fn play_audio_internal(file_path: &str, id: &str) -> Result<(), String> {
        // Get output stream
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| format!("Failed to create audio output stream: {}", e))?;

        // Open the audio file
        let file = File::open(file_path).map_err(|e| format!("Failed to open audio file: {}", e))?;

        // Decode the audio file
        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| format!("Failed to decode audio file: {}", e))?;

        // Create a new sink
        let sink =
            Sink::try_new(&stream_handle).map_err(|e| format!("Failed to create audio sink: {}", e))?;

        // Add the source to the sink
        sink.append(source);

        // Store the sink
        {
            let mut players = AUDIO_PLAYERS
                .lock()
                .map_err(|e| format!("Failed to lock audio players: {}", e))?;

            let sink_mutex = Arc::new(Mutex::new(Some(sink)));
            players.insert(id.to_string(), sink_mutex.clone());

            // Get the sink back
            let sink_clone = {
                let mut sink_opt = sink_mutex
                    .lock()
                    .map_err(|e| format!("Failed to lock sink: {}", e))?;
                sink_opt.take()
            };

            // Wait for playback to complete
            if let Some(sink) = sink_clone {
                sink.sleep_until_end();
            }

            // Remove the sink from the map
            players.remove(id);
        }

        Ok(())
    }
}

// Android/iOS implementation (stub)
#[cfg(any(target_os = "android", target_os = "ios"))]
mod audio_impl {
    use log::info;

    pub fn play_audio(file_path: String, id: Option<String>) -> Result<String, String> {
        let audio_id = id.unwrap_or_else(|| file_path.clone());
        info!("Audio not supported on this platform. Ignoring play request for: {}", file_path);
        Ok(audio_id)
    }

    pub fn stop_audio(_id: String) -> Result<(), String> {
        info!("Audio not supported on this platform. Ignoring stop request.");
        Ok(())
    }

    pub fn is_audio_playing(_id: String) -> Result<bool, String> {
        Ok(false)
    }
}

// Expose commands using the platform-specific implementations
#[cfg(not(target_os = "android"))]
#[tauri::command]
pub fn play_audio(file_path: String, id: Option<String>) -> Result<String, String> {
    audio_impl::play_audio(file_path, id)
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
pub fn stop_audio(id: String) -> Result<(), String> {
    audio_impl::stop_audio(id)
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
pub fn is_audio_playing(id: String) -> Result<bool, String> {
    audio_impl::is_audio_playing(id)
}
