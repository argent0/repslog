//! Application config (sanity absolute limits).
//!
//! Default path: `$XDG_CONFIG_HOME/repslog/config.toml` (usually `~/.config/repslog/config.toml`).
//! Override with global `--config PATH`.
//!
//! Absolute ranges only — no variation / delta layer.
//! Create the file with `repslog config generate` (does not auto-create on load).

use crate::error::{RepslogError, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Full on-disk application configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AppConfig {
    /// Absolute min/max ranges for inserted training metrics.
    #[serde(default)]
    pub sanity: SanityLimits,
}

/// Inclusive absolute min/max for one metric.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AbsoluteLimits {
    pub min: f64,
    pub max: f64,
}

impl AbsoluteLimits {
    pub const fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    fn collect_errors(&self, field: &str, meta: MetaBounds, errors: &mut Vec<String>) {
        if !self.min.is_finite() || !self.max.is_finite() {
            errors.push(format!("{field}: min and max must be finite"));
            return;
        }
        if self.min > self.max {
            errors.push(format!(
                "{field}: min ({}) must be <= max ({})",
                self.min, self.max
            ));
        }
        if meta.floor_exclusive {
            if self.min <= meta.floor {
                errors.push(format!(
                    "{field}: min ({}) must be > {}",
                    self.min, meta.floor
                ));
            }
        } else if self.min < meta.floor {
            errors.push(format!(
                "{field}: min ({}) must be >= {}",
                self.min, meta.floor
            ));
        }
        if self.max > meta.ceiling {
            errors.push(format!(
                "{field}: max ({}) must be <= {}",
                self.max, meta.ceiling
            ));
        }
    }
}

struct MetaBounds {
    floor: f64,
    floor_exclusive: bool,
    ceiling: f64,
}

/// Absolute-only limits for all numeric insert paths (no deltas).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SanityLimits {
    #[serde(default = "default_reps")]
    pub reps: AbsoluteLimits,
    #[serde(default = "default_weight_kg")]
    pub weight_kg: AbsoluteLimits,
    #[serde(default = "default_external_load_kg")]
    pub external_load_kg: AbsoluteLimits,
    #[serde(default = "default_duration_seconds")]
    pub duration_seconds: AbsoluteLimits,
    #[serde(default = "default_distance_km")]
    pub distance_km: AbsoluteLimits,
    #[serde(default = "default_rpe")]
    pub rpe: AbsoluteLimits,
    #[serde(default = "default_rir")]
    pub rir: AbsoluteLimits,
    #[serde(default = "default_effective_reps")]
    pub effective_reps: AbsoluteLimits,
    #[serde(default = "default_rest_seconds")]
    pub rest_seconds: AbsoluteLimits,
    #[serde(default = "default_heart_rate_bpm")]
    pub heart_rate_bpm: AbsoluteLimits,
    #[serde(default = "default_pace_min_per_km")]
    pub pace_min_per_km: AbsoluteLimits,
    #[serde(default = "default_calories_burned")]
    pub calories_burned: AbsoluteLimits,
    #[serde(default = "default_cadence_spm")]
    pub cadence_spm: AbsoluteLimits,
    #[serde(default = "default_elevation_m")]
    pub elevation_m: AbsoluteLimits,
    #[serde(default = "default_hr_zone_seconds")]
    pub hr_zone_seconds: AbsoluteLimits,
    #[serde(default = "default_duration_minutes")]
    pub duration_minutes: AbsoluteLimits,
    #[serde(default = "default_overall_feeling")]
    pub overall_feeling: AbsoluteLimits,
    #[serde(default = "default_latitude")]
    pub latitude: AbsoluteLimits,
    #[serde(default = "default_longitude")]
    pub longitude: AbsoluteLimits,
    #[serde(default = "default_speed_m_s")]
    pub speed_m_s: AbsoluteLimits,
}

fn default_reps() -> AbsoluteLimits {
    AbsoluteLimits::new(0.0, 500.0)
}
fn default_weight_kg() -> AbsoluteLimits {
    AbsoluteLimits::new(0.001, 500.0)
}
fn default_external_load_kg() -> AbsoluteLimits {
    AbsoluteLimits::new(-100.0, 200.0)
}
fn default_duration_seconds() -> AbsoluteLimits {
    AbsoluteLimits::new(1.0, 86400.0)
}
fn default_distance_km() -> AbsoluteLimits {
    AbsoluteLimits::new(0.001, 500.0)
}
fn default_rpe() -> AbsoluteLimits {
    AbsoluteLimits::new(0.0, 10.0)
}
fn default_rir() -> AbsoluteLimits {
    AbsoluteLimits::new(0.0, 20.0)
}
fn default_effective_reps() -> AbsoluteLimits {
    AbsoluteLimits::new(0.0, 500.0)
}
fn default_rest_seconds() -> AbsoluteLimits {
    AbsoluteLimits::new(0.0, 7200.0)
}
fn default_heart_rate_bpm() -> AbsoluteLimits {
    AbsoluteLimits::new(30.0, 250.0)
}
fn default_pace_min_per_km() -> AbsoluteLimits {
    AbsoluteLimits::new(2.0, 30.0)
}
fn default_calories_burned() -> AbsoluteLimits {
    AbsoluteLimits::new(0.0, 20000.0)
}
fn default_cadence_spm() -> AbsoluteLimits {
    AbsoluteLimits::new(0.0, 300.0)
}
fn default_elevation_m() -> AbsoluteLimits {
    AbsoluteLimits::new(0.0, 20000.0)
}
fn default_hr_zone_seconds() -> AbsoluteLimits {
    AbsoluteLimits::new(0.0, 86400.0)
}
fn default_duration_minutes() -> AbsoluteLimits {
    AbsoluteLimits::new(0.0, 1440.0)
}
fn default_overall_feeling() -> AbsoluteLimits {
    AbsoluteLimits::new(1.0, 5.0)
}
fn default_latitude() -> AbsoluteLimits {
    AbsoluteLimits::new(-90.0, 90.0)
}
fn default_longitude() -> AbsoluteLimits {
    AbsoluteLimits::new(-180.0, 180.0)
}
fn default_speed_m_s() -> AbsoluteLimits {
    AbsoluteLimits::new(0.0, 30.0)
}

impl Default for SanityLimits {
    fn default() -> Self {
        Self {
            reps: default_reps(),
            weight_kg: default_weight_kg(),
            external_load_kg: default_external_load_kg(),
            duration_seconds: default_duration_seconds(),
            distance_km: default_distance_km(),
            rpe: default_rpe(),
            rir: default_rir(),
            effective_reps: default_effective_reps(),
            rest_seconds: default_rest_seconds(),
            heart_rate_bpm: default_heart_rate_bpm(),
            pace_min_per_km: default_pace_min_per_km(),
            calories_burned: default_calories_burned(),
            cadence_spm: default_cadence_spm(),
            elevation_m: default_elevation_m(),
            hr_zone_seconds: default_hr_zone_seconds(),
            duration_minutes: default_duration_minutes(),
            overall_feeling: default_overall_feeling(),
            latitude: default_latitude(),
            longitude: default_longitude(),
            speed_m_s: default_speed_m_s(),
        }
    }
}

impl SanityLimits {
    /// Ensure each limit is structurally valid.
    pub fn validate(&self) -> Result<()> {
        let mut errors = Vec::new();
        let pos = |floor_exclusive| MetaBounds {
            floor: 0.0,
            floor_exclusive,
            ceiling: 1_000_000.0,
        };

        self.reps.collect_errors(
            "sanity.reps",
            MetaBounds {
                floor: 0.0,
                floor_exclusive: false,
                ceiling: 10_000.0,
            },
            &mut errors,
        );
        self.weight_kg
            .collect_errors("sanity.weight_kg", pos(true), &mut errors);
        self.external_load_kg.collect_errors(
            "sanity.external_load_kg",
            MetaBounds {
                floor: -500.0,
                floor_exclusive: false,
                ceiling: 1000.0,
            },
            &mut errors,
        );
        self.duration_seconds.collect_errors(
            "sanity.duration_seconds",
            MetaBounds {
                floor: 0.0,
                floor_exclusive: true,
                ceiling: 604_800.0,
            },
            &mut errors,
        );
        self.distance_km
            .collect_errors("sanity.distance_km", pos(true), &mut errors);
        self.rpe.collect_errors(
            "sanity.rpe",
            MetaBounds {
                floor: 0.0,
                floor_exclusive: false,
                ceiling: 10.0,
            },
            &mut errors,
        );
        self.rir.collect_errors(
            "sanity.rir",
            MetaBounds {
                floor: 0.0,
                floor_exclusive: false,
                ceiling: 50.0,
            },
            &mut errors,
        );
        self.effective_reps.collect_errors(
            "sanity.effective_reps",
            MetaBounds {
                floor: 0.0,
                floor_exclusive: false,
                ceiling: 10_000.0,
            },
            &mut errors,
        );
        self.rest_seconds.collect_errors(
            "sanity.rest_seconds",
            MetaBounds {
                floor: 0.0,
                floor_exclusive: false,
                ceiling: 86_400.0,
            },
            &mut errors,
        );
        self.heart_rate_bpm.collect_errors(
            "sanity.heart_rate_bpm",
            MetaBounds {
                floor: 0.0,
                floor_exclusive: true,
                ceiling: 400.0,
            },
            &mut errors,
        );
        self.pace_min_per_km.collect_errors(
            "sanity.pace_min_per_km",
            MetaBounds {
                floor: 0.0,
                floor_exclusive: true,
                ceiling: 120.0,
            },
            &mut errors,
        );
        self.calories_burned.collect_errors(
            "sanity.calories_burned",
            MetaBounds {
                floor: 0.0,
                floor_exclusive: false,
                ceiling: 100_000.0,
            },
            &mut errors,
        );
        self.cadence_spm.collect_errors(
            "sanity.cadence_spm",
            MetaBounds {
                floor: 0.0,
                floor_exclusive: false,
                ceiling: 500.0,
            },
            &mut errors,
        );
        self.elevation_m.collect_errors(
            "sanity.elevation_m",
            MetaBounds {
                floor: 0.0,
                floor_exclusive: false,
                ceiling: 100_000.0,
            },
            &mut errors,
        );
        self.hr_zone_seconds.collect_errors(
            "sanity.hr_zone_seconds",
            MetaBounds {
                floor: 0.0,
                floor_exclusive: false,
                ceiling: 604_800.0,
            },
            &mut errors,
        );
        self.duration_minutes.collect_errors(
            "sanity.duration_minutes",
            MetaBounds {
                floor: 0.0,
                floor_exclusive: false,
                ceiling: 10_080.0,
            },
            &mut errors,
        );
        self.overall_feeling.collect_errors(
            "sanity.overall_feeling",
            MetaBounds {
                floor: 1.0,
                floor_exclusive: false,
                ceiling: 5.0,
            },
            &mut errors,
        );
        self.latitude.collect_errors(
            "sanity.latitude",
            MetaBounds {
                floor: -90.0,
                floor_exclusive: false,
                ceiling: 90.0,
            },
            &mut errors,
        );
        self.longitude.collect_errors(
            "sanity.longitude",
            MetaBounds {
                floor: -180.0,
                floor_exclusive: false,
                ceiling: 180.0,
            },
            &mut errors,
        );
        self.speed_m_s.collect_errors(
            "sanity.speed_m_s",
            MetaBounds {
                floor: 0.0,
                floor_exclusive: false,
                ceiling: 100.0,
            },
            &mut errors,
        );

        if errors.is_empty() {
            Ok(())
        } else {
            Err(RepslogError::Config(errors.join("; ")))
        }
    }
}

impl AppConfig {
    pub fn validate(&self) -> Result<()> {
        self.sanity.validate()
    }
}

/// Result of resolving config for a run.
#[derive(Debug)]
pub struct LoadedConfig {
    pub config: AppConfig,
    pub path: PathBuf,
    /// True when a file was read from disk.
    pub from_file: bool,
}

/// Default config file path: `~/.config/repslog/config.toml`.
pub fn default_config_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "repslog") {
        proj_dirs.config_dir().join("config.toml")
    } else {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"));
        home.join(".config/repslog/config.toml")
    }
}

pub fn resolve_config_path(override_path: Option<&str>) -> PathBuf {
    match override_path {
        Some(p) => PathBuf::from(p),
        None => default_config_path(),
    }
}

/// Load config from disk if present; otherwise return in-memory defaults (no write).
pub fn load(override_path: Option<&str>) -> Result<LoadedConfig> {
    let path = resolve_config_path(override_path);
    if path.exists() {
        let config = load_from_path(&path)?;
        config.validate()?;
        Ok(LoadedConfig {
            config,
            path,
            from_file: true,
        })
    } else {
        let config = AppConfig::default();
        config.validate()?;
        Ok(LoadedConfig {
            config,
            path,
            from_file: false,
        })
    }
}

fn load_from_path(path: &Path) -> Result<AppConfig> {
    let raw = fs::read_to_string(path).map_err(|e| {
        RepslogError::Config(format!(
            "failed to read config at {}: {}",
            path.display(),
            e
        ))
    })?;
    toml::from_str(&raw).map_err(|e| {
        RepslogError::Config(format!(
            "failed to parse config at {}: {}",
            path.display(),
            e
        ))
    })
}

/// Write default config to `path`. Fails if file exists unless `force`.
pub fn generate_default_config(path: &Path, force: bool) -> Result<()> {
    if path.exists() && !force {
        return Err(RepslogError::Config(format!(
            "config already exists at {} (use --force to overwrite)",
            path.display()
        )));
    }
    let config = AppConfig::default();
    config.validate()?;
    write_config(path, &config)
}

fn write_config(path: &Path, config: &AppConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            RepslogError::Config(format!(
                "failed to create config directory {}: {}",
                parent.display(),
                e
            ))
        })?;
    }
    let pretty = toml::to_string_pretty(config)
        .map_err(|e| RepslogError::Config(format!("failed to serialize default config: {}", e)))?;
    // Header comment for humans
    let contents = format!(
        "# repslog configuration\n\
         # Absolute min/max ranges for metrics (hard-fail on insert/update).\n\
         # Generate with: repslog config generate\n\
         # Override path: repslog --config PATH ...\n\n\
         {}",
        pretty
    );
    fs::write(path, contents.as_bytes()).map_err(|e| {
        RepslogError::Config(format!(
            "failed to write config at {}: {}",
            path.display(),
            e
        ))
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn default_config_validates() {
        AppConfig::default().validate().unwrap();
    }

    #[test]
    fn generate_and_load_roundtrip() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("repslog-config-test-{}.toml", stamp));
        let _ = fs::remove_file(&path);
        generate_default_config(&path, false).unwrap();
        assert!(path.exists());
        let err = generate_default_config(&path, false).unwrap_err();
        assert!(format!("{}", err).contains("already exists"));
        generate_default_config(&path, true).unwrap();
        let loaded = load(Some(path.to_str().unwrap())).unwrap();
        assert!(loaded.from_file);
        assert_eq!(loaded.config, AppConfig::default());
        let _ = fs::remove_file(&path);
    }
}
