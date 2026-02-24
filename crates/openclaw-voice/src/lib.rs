//! OpenClaw Voice - 语音识别与合成模块
//!
//! 提供统一的语音接口，支持多种 STT/TTS 提供商
//!
//! ## 功能
//! - 语音识别 (STT) - 将语音转换为文本
//! - 语音合成 (TTS) - 将文本转换为语音
//! - 持续对话模式 (Talk Mode)
//! - 语音唤醒 (Voice Wake)
//!
//! ## 支持的提供商
//! - OpenAI Whisper (STT)
//! - OpenAI TTS
//! - Edge TTS (免费)
//! - 本地 Whisper (可选)
//! - Vosk (本地唤醒)

pub mod audio;
pub mod config;
pub mod provider;
pub mod stt;
pub mod talk_mode;
pub mod tts;
pub mod types;
pub mod voice_agent;
pub mod wake;

pub use audio::{AudioPlayer, AudioUtils};
pub use config::{VoiceConfigManager, default_voice_config};
pub use provider::ProviderRegistry;
pub use stt::{SpeechToText, create_stt};
pub use talk_mode::{TalkMode, TalkModeBuilder, TalkModeConfig, TalkModeEvent};
pub use tts::{TextToSpeech, create_tts};
pub use voice_agent::VoiceAgent;

pub use types::{SttConfig, SttProvider, TtsConfig, TtsProvider, VoiceConfig, TranscriptionResult, SynthesisOptions};
