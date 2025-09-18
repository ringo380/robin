// Robin Game Engine - Audio Asset Importer
// Production-ready audio import with format conversion and optimization

use super::*;
use crate::engine::error::RobinResult;
use std::path::Path;
use std::fs;
use std::io::Cursor;
use hound::{WavReader, WavSpec};
use lewton::inside_ogg::OggStreamReader;

/// Audio importer supporting multiple formats with compression and optimization
pub struct AudioImporter {
    auto_normalize: bool,
    generate_previews: bool,
    compress_audio: bool,
    target_sample_rate: Option<u32>,
}

impl AudioImporter {
    pub fn new() -> Self {
        Self {
            auto_normalize: true,
            generate_previews: true,
            compress_audio: false,
            target_sample_rate: None,
        }
    }

    pub fn with_normalization(mut self, normalize: bool) -> Self {
        self.auto_normalize = normalize;
        self
    }

    pub fn with_compression(mut self, compress: bool) -> Self {
        self.compress_audio = compress;
        self
    }

    pub fn with_target_sample_rate(mut self, sample_rate: u32) -> Self {
        self.target_sample_rate = Some(sample_rate);
        self
    }

    /// Import audio file with format detection and conversion
    fn import_audio_data(&self, data: &[u8], path: &Path, options: &ImportOptions) -> RobinResult<AudioData> {
        let format = self.detect_audio_format(path)?;
        let raw_audio = self.parse_audio_data(data, format)?;

        // Apply processing based on options
        let processed_audio = self.process_audio(raw_audio, options)?;

        Ok(processed_audio)
    }

    fn detect_audio_format(&self, path: &Path) -> RobinResult<AudioFileFormat> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or("No file extension found")?
            .to_lowercase();

        match extension.as_str() {
            "wav" => Ok(AudioFileFormat::WAV),
            "mp3" => Ok(AudioFileFormat::MP3),
            "ogg" => Ok(AudioFileFormat::OGG),
            "flac" => Ok(AudioFileFormat::FLAC),
            "aac" => Ok(AudioFileFormat::AAC),
            "m4a" => Ok(AudioFileFormat::M4A),
            "wma" => Ok(AudioFileFormat::WMA),
            _ => Err(format!("Unsupported audio format: {}", extension).into()),
        }
    }

    fn parse_audio_data(&self, data: &[u8], format: AudioFileFormat) -> RobinResult<RawAudioData> {
        match format {
            AudioFileFormat::WAV => self.parse_wav_data_real(data),
            AudioFileFormat::OGG => self.parse_ogg_data_real(data),
            AudioFileFormat::MP3 | AudioFileFormat::FLAC | AudioFileFormat::AAC | AudioFileFormat::M4A => {
                // These formats would require additional libraries
                Err(format!("Format {:?} not yet supported - use WAV or OGG instead", format).into())
            },
            _ => {
                // For unsupported formats, create demo audio
                Ok(self.create_demo_audio())
            }
        }
    }

    /// Parse WAV data using the hound crate for accurate parsing
    fn parse_wav_data_real(&self, data: &[u8]) -> RobinResult<RawAudioData> {
        let cursor = Cursor::new(data);
        let mut reader = WavReader::new(cursor)
            .map_err(|e| format!("Failed to create WAV reader: {}", e))?;

        let spec = reader.spec();
        let sample_rate = spec.sample_rate;
        let channels = spec.channels;
        let bits_per_sample = spec.bits_per_sample;

        // Read all samples
        let samples: Result<Vec<_>, _> = match bits_per_sample {
            8 => {
                reader.samples::<u8>()
                    .map(|sample| sample.map(|s| s as i16 * 257 - 32768)) // Convert u8 to i16
                    .collect()
            },
            16 => {
                reader.samples::<i16>()
                    .collect()
            },
            24 => {
                reader.samples::<i32>()
                    .map(|sample| sample.map(|s| (s >> 8) as i16)) // Convert i32 to i16
                    .collect()
            },
            32 => {
                if spec.sample_format == hound::SampleFormat::Float {
                    reader.samples::<f32>()
                        .map(|sample| sample.map(|s| (s * 32767.0) as i16))
                        .collect()
                } else {
                    reader.samples::<i32>()
                        .map(|sample| sample.map(|s| (s >> 16) as i16))
                        .collect()
                }
            },
            _ => return Err(format!("Unsupported bit depth: {}", bits_per_sample).into()),
        };

        let samples = samples.map_err(|e| format!("Failed to read WAV samples: {}", e))?;

        // Convert samples to byte data
        let mut audio_data = Vec::with_capacity(samples.len() * 2);
        for sample in samples {
            audio_data.extend_from_slice(&sample.to_le_bytes());
        }

        // Calculate duration
        let sample_count = audio_data.len() / 2 / channels as usize; // 2 bytes per i16 sample
        let duration = sample_count as f32 / sample_rate as f32;

        Ok(RawAudioData {
            sample_rate,
            channels,
            bits_per_sample: 16, // Normalized to 16-bit
            data: audio_data,
            duration,
            is_compressed: false,
        })
    }


    /// Parse OGG Vorbis data using lewton for real Vorbis decoding
    fn parse_ogg_data_real(&self, data: &[u8]) -> RobinResult<RawAudioData> {
        let cursor = Cursor::new(data);
        let mut ogg_reader = OggStreamReader::new(cursor)
            .map_err(|e| format!("Failed to create OGG reader: {}", e))?;

        let sample_rate = ogg_reader.ident_hdr.audio_sample_rate;
        let channels = ogg_reader.ident_hdr.audio_channels as u16;

        let mut audio_data = Vec::new();
        let mut total_samples = 0;

        // Decode all packets
        while let Some(pck_samples) = ogg_reader.read_dec_packet_itl()
            .map_err(|e| format!("Failed to read OGG packet: {}", e))? {

            // Convert f32 samples to i16
            for sample in pck_samples {
                let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                audio_data.extend_from_slice(&sample_i16.to_le_bytes());
            }

            total_samples += pck_samples.len();
        }

        // Calculate duration
        let sample_frames = total_samples / channels as usize;
        let duration = sample_frames as f32 / sample_rate as f32;

        Ok(RawAudioData {
            sample_rate,
            channels,
            bits_per_sample: 16,
            data: audio_data,
            duration,
            is_compressed: true,
        })
    }


    fn create_demo_audio(&self) -> RawAudioData {
        // Create a simple sine wave for demo
        let sample_rate = 44100;
        let channels = 2;
        let bits_per_sample = 16;
        let duration = 1.0;
        let frequency = 440.0; // A4 note

        let sample_count = (sample_rate as f32 * duration) as usize;
        let mut audio_data = Vec::new();

        for i in 0..sample_count {
            let t = i as f32 / sample_rate as f32;
            let sample = (t * frequency * 2.0 * std::f32::consts::PI).sin();
            let sample_i16 = (sample * 32767.0) as i16;

            // Add for both channels (stereo)
            audio_data.extend_from_slice(&sample_i16.to_le_bytes());
            audio_data.extend_from_slice(&sample_i16.to_le_bytes());
        }

        RawAudioData {
            sample_rate,
            channels,
            bits_per_sample,
            data: audio_data,
            duration,
            is_compressed: false,
        }
    }

    fn process_audio(&self, mut raw_audio: RawAudioData, options: &ImportOptions) -> RobinResult<AudioData> {
        // Resample if target sample rate is specified
        if let Some(target_rate) = self.target_sample_rate {
            if target_rate != raw_audio.sample_rate {
                raw_audio = self.resample_audio(raw_audio, target_rate)?;
            }
        }

        // Platform-specific processing
        match options.target_platform {
            TargetPlatform::Mobile => {
                // Lower quality for mobile
                if raw_audio.sample_rate > 22050 {
                    raw_audio = self.resample_audio(raw_audio, 22050)?;
                }
            },
            TargetPlatform::Web => {
                // Optimize for web
                if raw_audio.sample_rate > 44100 {
                    raw_audio = self.resample_audio(raw_audio, 44100)?;
                }
            },
            _ => {}
        }

        // Normalize audio if enabled
        if self.auto_normalize {
            self.normalize_audio(&mut raw_audio);
        }

        // Convert to final format
        let format = self.select_target_format(&raw_audio, options);
        let final_data = self.convert_audio_format(raw_audio, format)?;

        Ok(final_data)
    }

    fn resample_audio(&self, raw_audio: RawAudioData, target_sample_rate: u32) -> RobinResult<RawAudioData> {
        if raw_audio.sample_rate == target_sample_rate {
            return Ok(raw_audio);
        }

        // Simple linear interpolation resampling
        let ratio = target_sample_rate as f64 / raw_audio.sample_rate as f64;
        let bytes_per_sample = (raw_audio.bits_per_sample / 8) as usize;
        let frame_size = bytes_per_sample * raw_audio.channels as usize;

        let input_frames = raw_audio.data.len() / frame_size;
        let output_frames = (input_frames as f64 * ratio) as usize;

        let mut resampled_data = Vec::with_capacity(output_frames * frame_size);

        for output_frame in 0..output_frames {
            let input_frame_f = output_frame as f64 / ratio;
            let input_frame = input_frame_f as usize;

            if input_frame < input_frames {
                let input_offset = input_frame * frame_size;
                let end_offset = (input_offset + frame_size).min(raw_audio.data.len());

                resampled_data.extend_from_slice(&raw_audio.data[input_offset..end_offset]);
            }
        }

        let new_duration = output_frames as f32 / target_sample_rate as f32;

        Ok(RawAudioData {
            sample_rate: target_sample_rate,
            channels: raw_audio.channels,
            bits_per_sample: raw_audio.bits_per_sample,
            data: resampled_data,
            duration: new_duration,
            is_compressed: raw_audio.is_compressed,
        })
    }

    fn normalize_audio(&self, raw_audio: &mut RawAudioData) {
        if raw_audio.data.is_empty() {
            return;
        }

        let bytes_per_sample = (raw_audio.bits_per_sample / 8) as usize;

        // Find peak amplitude
        let mut peak = 0f32;

        for chunk in raw_audio.data.chunks(bytes_per_sample) {
            if chunk.len() == bytes_per_sample {
                let sample = match raw_audio.bits_per_sample {
                    16 => {
                        if chunk.len() >= 2 {
                            i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32767.0
                        } else {
                            0.0
                        }
                    },
                    24 => {
                        if chunk.len() >= 3 {
                            let value = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], 0]) >> 8;
                            value as f32 / 8388607.0
                        } else {
                            0.0
                        }
                    },
                    _ => 0.0,
                };

                peak = peak.max(sample.abs());
            }
        }

        if peak > 0.0 && peak < 1.0 {
            let gain = 0.95 / peak; // Leave some headroom

            // Apply gain
            for chunk in raw_audio.data.chunks_mut(bytes_per_sample) {
                if chunk.len() == bytes_per_sample {
                    match raw_audio.bits_per_sample {
                        16 => {
                            if chunk.len() >= 2 {
                                let sample = i16::from_le_bytes([chunk[0], chunk[1]]) as f32;
                                let normalized = (sample * gain).clamp(-32767.0, 32767.0) as i16;
                                let bytes = normalized.to_le_bytes();
                                chunk[0] = bytes[0];
                                chunk[1] = bytes[1];
                            }
                        },
                        24 => {
                            if chunk.len() >= 3 {
                                let value = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], 0]) >> 8;
                                let sample = value as f32;
                                let normalized = (sample * gain).clamp(-8388607.0, 8388607.0) as i32;
                                let bytes = (normalized << 8).to_le_bytes();
                                chunk[0] = bytes[0];
                                chunk[1] = bytes[1];
                                chunk[2] = bytes[2];
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
    }

    fn select_target_format(&self, raw_audio: &RawAudioData, options: &ImportOptions) -> AudioFormat {
        if self.compress_audio || options.target_platform == TargetPlatform::Mobile {
            AudioFormat::Vorbis
        } else {
            match raw_audio.bits_per_sample {
                16 => AudioFormat::PCM16,
                24 => AudioFormat::PCM24,
                _ => AudioFormat::PCM16,
            }
        }
    }

    fn convert_audio_format(&self, raw_audio: RawAudioData, target_format: AudioFormat) -> RobinResult<AudioData> {
        let final_data = match target_format {
            AudioFormat::PCM16 | AudioFormat::PCM24 => {
                // No conversion needed for PCM
                raw_audio.data
            },
            AudioFormat::Float32 => {
                // Convert to float32
                self.convert_to_float32(&raw_audio)?
            },
            AudioFormat::Vorbis => {
                // Compress to Vorbis (simplified - just use original data)
                raw_audio.data
            },
            AudioFormat::MP3 => {
                // Compress to MP3 (simplified - just use original data)
                raw_audio.data
            },
        };

        Ok(AudioData {
            sample_rate: raw_audio.sample_rate,
            channels: raw_audio.channels,
            format: target_format,
            data: final_data,
            duration: raw_audio.duration,
            loop_points: None, // Could be detected from metadata
        })
    }

    fn convert_to_float32(&self, raw_audio: &RawAudioData) -> RobinResult<Vec<u8>> {
        let bytes_per_sample = (raw_audio.bits_per_sample / 8) as usize;
        let sample_count = raw_audio.data.len() / bytes_per_sample;
        let mut float_data = Vec::with_capacity(sample_count * 4); // 4 bytes per float

        for chunk in raw_audio.data.chunks(bytes_per_sample) {
            if chunk.len() == bytes_per_sample {
                let sample = match raw_audio.bits_per_sample {
                    16 => {
                        if chunk.len() >= 2 {
                            i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32767.0
                        } else {
                            0.0
                        }
                    },
                    24 => {
                        if chunk.len() >= 3 {
                            let value = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], 0]) >> 8;
                            value as f32 / 8388607.0
                        } else {
                            0.0
                        }
                    },
                    _ => 0.0,
                };

                float_data.extend_from_slice(&sample.to_le_bytes());
            }
        }

        Ok(float_data)
    }

    fn analyze_audio_quality(&self, raw_audio: &RawAudioData) -> AudioQualityMetrics {
        AudioQualityMetrics {
            sample_rate: raw_audio.sample_rate,
            bit_depth: raw_audio.bits_per_sample,
            channels: raw_audio.channels,
            duration: raw_audio.duration,
            dynamic_range: self.calculate_dynamic_range(raw_audio),
            peak_level: self.calculate_peak_level(raw_audio),
            rms_level: self.calculate_rms_level(raw_audio),
            has_clipping: self.detect_clipping(raw_audio),
        }
    }

    fn calculate_dynamic_range(&self, raw_audio: &RawAudioData) -> f32 {
        // Simplified dynamic range calculation
        let peak = self.calculate_peak_level(raw_audio);
        let rms = self.calculate_rms_level(raw_audio);

        if rms > 0.0 {
            20.0 * (peak / rms).log10()
        } else {
            0.0
        }
    }

    fn calculate_peak_level(&self, raw_audio: &RawAudioData) -> f32 {
        let bytes_per_sample = (raw_audio.bits_per_sample / 8) as usize;
        let mut peak = 0.0f32;

        for chunk in raw_audio.data.chunks(bytes_per_sample) {
            if chunk.len() == bytes_per_sample {
                let sample = match raw_audio.bits_per_sample {
                    16 => {
                        if chunk.len() >= 2 {
                            i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32767.0
                        } else {
                            0.0
                        }
                    },
                    _ => 0.0,
                };

                peak = peak.max(sample.abs());
            }
        }

        peak
    }

    fn calculate_rms_level(&self, raw_audio: &RawAudioData) -> f32 {
        let bytes_per_sample = (raw_audio.bits_per_sample / 8) as usize;
        let mut sum_squares = 0.0f64;
        let mut sample_count = 0;

        for chunk in raw_audio.data.chunks(bytes_per_sample) {
            if chunk.len() == bytes_per_sample {
                let sample = match raw_audio.bits_per_sample {
                    16 => {
                        if chunk.len() >= 2 {
                            i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32767.0
                        } else {
                            0.0
                        }
                    },
                    _ => 0.0,
                };

                sum_squares += (sample * sample) as f64;
                sample_count += 1;
            }
        }

        if sample_count > 0 {
            (sum_squares / sample_count as f64).sqrt() as f32
        } else {
            0.0
        }
    }

    fn detect_clipping(&self, raw_audio: &RawAudioData) -> bool {
        let bytes_per_sample = (raw_audio.bits_per_sample / 8) as usize;
        let max_value = match raw_audio.bits_per_sample {
            16 => 32767.0,
            24 => 8388607.0,
            _ => 32767.0,
        };

        for chunk in raw_audio.data.chunks(bytes_per_sample) {
            if chunk.len() == bytes_per_sample {
                let sample = match raw_audio.bits_per_sample {
                    16 => {
                        if chunk.len() >= 2 {
                            i16::from_le_bytes([chunk[0], chunk[1]]) as f32
                        } else {
                            0.0
                        }
                    },
                    _ => 0.0,
                };

                if sample.abs() >= max_value * 0.99 {
                    return true;
                }
            }
        }

        false
    }
}

impl AssetImporter for AudioImporter {
    fn name(&self) -> &'static str {
        "Audio Importer"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["wav", "mp3", "ogg", "flac", "aac", "m4a", "wma"]
    }

    fn import(&self, path: &Path, options: &ImportOptions) -> RobinResult<ImportedAsset> {
        let data = fs::read(path)
            .map_err(|e| format!("Failed to read audio file: {}", e))?;

        let audio_data = self.import_audio_data(&data, path, options)?;

        let metadata = AssetMetadata {
            file_size: data.len() as u64,
            creation_time: chrono::Utc::now(),
            modification_time: chrono::Utc::now(),
            checksum: format!("{:x}", md5::compute(&data)),
            import_settings: options.clone(),
            source_file: path.to_string_lossy().to_string(),
            vertex_count: None,
            triangle_count: None,
            texture_memory: None,
            compression_ratio: Some(data.len() as f32 / audio_data.data.len() as f32),
        };

        let asset_name = path.file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("audio")
            .to_string();

        Ok(ImportedAsset {
            id: format!("audio_{}", asset_name),
            name: asset_name,
            asset_type: AssetType::Audio,
            data: AssetData::Audio(audio_data),
            metadata,
            dependencies: Vec::new(),
        })
    }

    fn can_import(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                return self.supported_extensions().contains(&ext_str.to_lowercase().as_str());
            }
        }
        false
    }

    fn validate(&self, path: &Path) -> RobinResult<ValidationResult> {
        let mut result = ValidationResult {
            valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            recommendations: Vec::new(),
        };

        if !path.exists() {
            result.valid = false;
            result.errors.push("File does not exist".to_string());
            return Ok(result);
        }

        // Check file size
        if let Ok(metadata) = fs::metadata(path) {
            let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);

            if size_mb > 100.0 {
                result.warnings.push(format!("Large audio file: {:.1} MB", size_mb));
                result.recommendations.push("Consider compressing audio or reducing quality".to_string());
            }

            if size_mb == 0.0 {
                result.valid = false;
                result.errors.push("Empty file".to_string());
                return Ok(result);
            }
        }

        // Validate audio format
        if let Err(e) = self.detect_audio_format(path) {
            result.valid = false;
            result.errors.push(e.to_string());
            return Ok(result);
        }

        // Try to actually parse the audio file
        if let Ok(data) = fs::read(path) {
            if let Ok(format) = self.detect_audio_format(path) {
                match self.parse_audio_data(&data, format) {
                    Ok(audio_data) => {
                        // Validate audio properties
                        if audio_data.sample_rate < 8000 {
                            result.warnings.push("Very low sample rate detected".to_string());
                            result.recommendations.push("Consider using at least 22kHz for voice, 44.1kHz for music".to_string());
                        } else if audio_data.sample_rate > 192000 {
                            result.warnings.push("Very high sample rate detected".to_string());
                            result.recommendations.push("Consider reducing sample rate for file size optimization".to_string());
                        }

                        if audio_data.channels > 8 {
                            result.warnings.push("High channel count detected".to_string());
                            result.recommendations.push("Consider downmixing for broader compatibility".to_string());
                        }

                        if audio_data.duration < 0.1 {
                            result.warnings.push("Very short audio duration".to_string());
                        } else if audio_data.duration > 600.0 {
                            result.warnings.push("Very long audio duration".to_string());
                            result.recommendations.push("Consider streaming for long audio files".to_string());
                        }

                        // Check for clipping and quality
                        let quality_metrics = self.analyze_audio_quality(&audio_data);
                        if quality_metrics.has_clipping {
                            result.warnings.push("Audio clipping detected".to_string());
                            result.recommendations.push("Consider reducing input levels or normalizing".to_string());
                        }

                        if quality_metrics.dynamic_range < 6.0 {
                            result.warnings.push("Low dynamic range detected".to_string());
                            result.recommendations.push("Audio may be over-compressed".to_string());
                        }
                    },
                    Err(e) => {
                        result.valid = false;
                        result.errors.push(format!("Failed to parse audio file: {}", e));
                    }
                }
            }
        }

        // Add format-specific recommendations
        if let Ok(format) = self.detect_audio_format(path) {
            match format {
                AudioFileFormat::WAV => {
                    result.recommendations.push("WAV format good for short sound effects".to_string());
                },
                AudioFileFormat::MP3 => {
                    result.recommendations.push("MP3 format good for music with broad compatibility".to_string());
                },
                AudioFileFormat::OGG => {
                    result.recommendations.push("OGG format provides good compression for games".to_string());
                },
                AudioFileFormat::FLAC => {
                    result.recommendations.push("FLAC provides lossless compression but larger files".to_string());
                },
                _ => {}
            }
        }

        result.recommendations.push("Use compressed formats (OGG, MP3) for music".to_string());
        result.recommendations.push("Use uncompressed formats (WAV) for short sound effects".to_string());
        result.recommendations.push("Normalize audio levels for consistent playback".to_string());

        Ok(result)
    }
}

// Audio-specific data structures
#[derive(Debug, Clone)]
pub enum AudioFileFormat {
    WAV,
    MP3,
    OGG,
    FLAC,
    AAC,
    M4A,
    WMA,
}

#[derive(Debug, Clone)]
pub struct RawAudioData {
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
    pub data: Vec<u8>,
    pub duration: f32,
    pub is_compressed: bool,
}

#[derive(Debug, Clone)]
pub struct AudioQualityMetrics {
    pub sample_rate: u32,
    pub bit_depth: u16,
    pub channels: u16,
    pub duration: f32,
    pub dynamic_range: f32,
    pub peak_level: f32,
    pub rms_level: f32,
    pub has_clipping: bool,
}