//! macOS System TTS using `say` command

use crate::types::{TtsConfig, TtsProvider};
use async_trait::async_trait;
use openclaw_core::Result;
use std::process::Command;

pub struct MacOSSystts {
    voice: String,
    rate: i32,
}

impl MacOSSystts {
    pub fn new(config: TtsConfig) -> Self {
        Self {
            voice: config.default_voice.as_str().to_string(),
            rate: (config.default_speed * 100.0) as i32,
        }
    }
}

#[async_trait]
impl crate::tts::TextToSpeech for MacOSSystts {
    fn provider(&self) -> TtsProvider {
        TtsProvider::MacOSSystem
    }

    async fn synthesize(&self, text: &str, _options: Option<crate::types::SynthesisOptions>) -> Result<Vec<u8>> {
        let output = Command::new("say")
            .arg("-v")
            .arg(&self.voice)
            .arg("-r")
            .arg(self.rate.to_string())
            .arg("-o")
            .arg("/tmp/openclaw_tts.aiff")
            .arg(text)
            .output()
            .map_err(|e| openclaw_core::OpenClawError::Io(e))?;

        if !output.status.success() {
            return Err(openclaw_core::OpenClawError::Io(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("say command failed: {}", String::from_utf8_lossy(&output.stderr)),
                ),
            ));
        }

        let audio_data = std::fs::read("/tmp/openclaw_tts.aiff")
            .map_err(|e| openclaw_core::OpenClawError::Io(e))?;

        std::fs::remove_file("/tmp/openclaw_tts.aiff").ok();

        Ok(audio_data)
    }

    async fn is_available(&self) -> bool {
        Command::new("say").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
    }

    fn available_voices(&self) -> Vec<String> {
        let output = Command::new("say")
            .arg("-v")
            .arg("?")
            .output();

        match output {
            Ok(o) if o.status.success() => {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .filter_map(|line| line.split_whitespace().next().map(String::from))
                    .collect()
            }
            _ => vec![
                "Alex".to_string(),
                "Samantha".to_string(),
                "Victoria".to_string(),
                "Daniel".to_string(),
                "Fred".to_string(),
                "Alice".to_string(),
            ],
        }
    }
}
