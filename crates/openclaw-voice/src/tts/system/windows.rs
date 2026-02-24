//! Windows System TTS using PowerShell or SAPI

use crate::types::{TtsConfig, TtsProvider};
use async_trait::async_trait;
use openclaw_core::Result;
use std::process::Command;

pub struct WindowsSystemTts {
    voice: String,
    rate: i32,
}

impl WindowsSystemTts {
    pub fn new(config: TtsConfig) -> Self {
        Self {
            voice: if config.default_voice.as_str().is_empty() {
                "David".to_string()
            } else {
                config.default_voice.as_str().to_string()
            },
            rate: ((config.default_speed - 1.0) * 50.0) as i32,
        }
    }
}

#[async_trait]
impl crate::tts::TextToSpeech for WindowsSystemTts {
    fn provider(&self) -> TtsProvider {
        TtsProvider::WindowsSystem
    }

    async fn synthesize(&self, text: &str, _options: Option<crate::types::SynthesisOptions>) -> Result<Vec<u8>> {
        let escaped_text = text.replace("'", "''");
        let script = format!(
            r#"
            Add-Type -AssemblyName System.Speech
            $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
            $synth.SelectVoice('{}')
            $synth.Rate = {}
            $synth.SetOutputToWaveFile('C:\Windows\Temp\openclaw_tts.wav')
            $synth.Speak('{}')
            $synth.Dispose()
            "#,
            self.voice, self.rate, escaped_text
        );

        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
            .map_err(|e| openclaw_core::OpenClawError::Io(e))?;

        if !output.status.success() {
            return Err(openclaw_core::OpenClawError::Io(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("PowerShell TTS failed: {}", String::from_utf8_lossy(&output.stderr)),
                ),
            ));
        }

        let audio_data = std::fs::read("C:\\Windows\\Temp\\openclaw_tts.wav")
            .map_err(|e| openclaw_core::OpenClawError::Io(e))?;

        std::fs::remove_file("C:\\Windows\\Temp\\openclaw_tts.wav").ok();

        Ok(audio_data)
    }

    async fn is_available(&self) -> bool {
        Command::new("powershell")
            .args(["-NoProfile", "-Command", "Add-Type -AssemblyName System.Speech; $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer; $synth.GetInstalledVoices()"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn available_voices(&self) -> Vec<String> {
        let script = r#"Add-Type -AssemblyName System.Speech; $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer; $synth.GetInstalledVoices() | ForEach-Object { $_.VoiceInfo.Name }"#;
        
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .map(String::from)
                    .collect()
            }
            _ => vec![
                "David".to_string(),
                "Zira".to_string(),
                "James".to_string(),
                "Haruka".to_string(),
                "Huihui".to_string(),
            ],
        }
    }
}
