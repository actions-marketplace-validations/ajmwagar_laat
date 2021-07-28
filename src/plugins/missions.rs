use crate::context::AddonManager;
use crate::create_handlebars;
use crate::BuildContext;
use crate::Plugin;
use crate::Result;
use armake2::config::{Config, ConfigArrayElement, ConfigClass, ConfigEntry};
use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const MISSION_SETTINGS_KEY: &str = "missions";

#[derive(Debug)]
pub struct MissionPlugin;

#[async_trait]
impl Plugin for MissionPlugin {
    #[instrument(err, skip(build_config))]
    async fn build(&self, build_config: BuildContext) -> Result<()> {
        // Extract MissionSettings from BuildContext
        let mission_settings = MissionSettings::from_build_config(&build_config)?;

        // Load composition file
        let composition = load_composition(
            &mission_settings.composition,
            mission_settings.composition_offset,
        )
        .await?;

        // For each Map create mission based on settings.
        let mut missions = create_missions(&mission_settings, &build_config).await?;

        // Merge composition into mission
        missions.iter_mut().for_each(|mission| {
            if let Err(why) = mission.merge_composition(&composition) {
                warn!("Failed to merge composition: {}", why);
            }
        });

        // Save mission to addon
        let mut addon_manager =
            AddonManager::from_context(&mission_settings.addon_name, build_config.clone());

        let classes = missions
            .into_iter()
            .filter_map(|mission| {
                let path: PathBuf =
                    format!("missions/{}/mission.sqm", mission.mission_name()).into();

                let sqm = match mission.to_sqm() {
                    Ok(sqm) => sqm,
                    Err(err) => {
                        warn!("Error creating sqm: {}", err);
                        return None;
                    }
                };

                addon_manager.add_file(sqm, path.clone());

                Some((path, mission))
            })
            .collect::<Vec<_>>();

        // Write config exposing Missions
        info!("Writing config.cpp...");
        let handlebars = create_handlebars()?;

        let addon = Addon::from_parts(build_config.prefix, mission_settings.addon_name, classes);
        let config_cpp = handlebars.render("missions_addon", &addon)?;

        addon_manager.add_file(config_cpp, "config.cpp".into());

        info!("Building Addon...");
        addon_manager.build_addon().await?;

        Ok(())
    }

    fn name(&self) -> String {
        "missions".to_string()
    }
}

#[derive(Debug, Deserialize)]
struct MissionSettings {
    #[serde(default = "default_addon_name")]
    /// Name of the generated Addon
    addon_name: String,

    /// List of maps to create missions for
    maps: Vec<String>,

    /// Mission name
    #[serde(default = "default_mission_name")]
    mission_name: String,

    /// Delay, in seconds between death and when allowed to respawn.
    #[serde(default = "default_respawn_delay")]
    respawn_delay: usize,

    /// Composition to add to missions
    composition: PathBuf,

    #[serde(default)]
    /// X, Y, Z offset for the composition.
    composition_offset: (f32, f32, f32),
}

impl MissionSettings {
    pub fn from_build_config(build_config: &BuildContext) -> Result<MissionSettings> {
        if let Some(mission_settings) = build_config.extra.get(MISSION_SETTINGS_KEY) {
            let mission_settings: MissionSettings = mission_settings.clone().try_into()?;

            Ok(mission_settings)
        } else {
            Err(format!(
                "Failed to get field: {} from LAAT.toml",
                MISSION_SETTINGS_KEY
            )
            .into())
        }
    }
}

fn default_addon_name() -> String {
    "Missions".to_string()
}

fn default_mission_name() -> String {
    "ZeusMission".to_string()
}

fn default_respawn_delay() -> usize {
    2
}

struct Composition {
    header: Config,
    composition: Config,
    offset: (f32, f32, f32),
}

impl Composition {
    #[instrument(err)]
    pub async fn from_path(path: &PathBuf, offset: (f32, f32, f32)) -> Result<Self> {
        let (header, composition) = tokio::join!(
            tokio::fs::File::open(format!("{}/header.sqe", path.display())),
            tokio::fs::File::open(format!("{}/composition.sqe", path.display()))
        );

        let header = Config::read(&mut header?.into_std().await, None, &Vec::new())?;
        let composition = Config::read(&mut composition?.into_std().await, None, &Vec::new())?;

        Ok(Composition {
            header,
            composition,
            offset,
        })
    }

    /// Get "center[]" from SQE, cast it into a tuple
    pub fn get_center(&self) -> Result<(f32, f32, f32)> {
        let config = self.composition.inner();

        if let Some(entries) = config.entries.clone() {
            let map: HashMap<String, ConfigEntry> = entries.into_iter().collect();

            if let Some(ConfigEntry::ArrayEntry(array)) = map.get("center") {
                debug!("Center Array: {:?}", array);
                if let &[ConfigArrayElement::FloatElement(x), ConfigArrayElement::FloatElement(y), ConfigArrayElement::FloatElement(z)] =
                    &array.elements[..]
                {
                    return Ok((x, y, z));
                }
            };
        }

        Err("Failed to get center[]".into())
    }
    pub fn get_offset(&self) -> Result<(f32, f32, f32)> {
        let (x1, y1, z1) = self.get_center()?;
        let (x2, y2, z2) = self.offset;

        Ok((x1 + x2, y1 + y2, z1 + z2))
    }

    /// Get and offset items from the SQE
    pub fn get_offseted_items(&self) -> Result<EntryList> {
        let config = self.composition.inner();

        if let Some(entries) = config.entries.clone() {
            let map: HashMap<String, ConfigEntry> = entries.into_iter().collect();

            if let Some(ConfigEntry::ClassEntry(items)) = map.get("items") {
                if let Some(entries) = items.entries.clone() {
                    debug!("Item Classes: {}", entries.len());
                    return Ok(offset_classes(entries, self.get_offset()?));
                }
            };
        }

        Err("Failed to get offseted items".into())
    }
}

type EntryList = Vec<(String, ConfigEntry)>;

const POSITION_INFO: &str = "PositionInfo";

/// Offset classes recursively
#[instrument(skip(entries, composition_offset))]
fn offset_classes(entries: EntryList, composition_offset: (f32, f32, f32)) -> EntryList {
    let offsets = [
        composition_offset.0,
        composition_offset.1,
        composition_offset.2,
    ];

    entries
        .into_iter()
        .map(|(name, entry)| {
            let entry = if let ConfigEntry::ClassEntry(mut class) = entry {
                if name == POSITION_INFO {
                    // Offset
                    class.entries = class.entries.map(|mut entries| {
                        entries.iter_mut().find(|(name, _)| name == "position").map(
                            |(name, entry)| {
                                if let ConfigEntry::ArrayEntry(position) = entry {
                                    position.elements = position
                                        .elements
                                        .iter_mut()
                                        .enumerate()
                                        .map(|(idx, el)| add_to_element(el.clone(), offsets[idx]))
                                        .collect();

                                    (name, entry)
                                } else {
                                    (name, entry)
                                }
                            },
                        );

                        entries
                    });
                } else {
                    // Recurse
                    class.entries = class
                        .entries
                        .map(|entries| offset_classes(entries, composition_offset));
                }

                ConfigEntry::ClassEntry(class)
            } else {
                entry
            };

            (name, entry)
        })
        .collect()
}

fn add_to_element(element: ConfigArrayElement, increment: f32) -> ConfigArrayElement {
    match element {
        ConfigArrayElement::StringElement(_) => {}
        ConfigArrayElement::FloatElement(float) => {
            return ConfigArrayElement::FloatElement(float + increment);
        }
        ConfigArrayElement::IntElement(_) => {}
        ConfigArrayElement::ArrayElement(_) => {}
    }

    element
}

#[instrument(err)]
async fn load_composition(
    composition_path: &PathBuf,
    composition_offset: (f32, f32, f32),
) -> Result<Composition> {
    info!("Loading composition at: {:?}", composition_path);
    Ok(Composition::from_path(composition_path, composition_offset).await?)
}

#[instrument(err)]
async fn create_missions(
    mission_settings: &MissionSettings,
    build_config: &BuildContext,
) -> Result<Vec<Mission>> {
    info!("Creating missions...");
    Ok(mission_settings
        .maps
        .iter()
        .filter_map(|map| {
            Mission::new(
                build_config.prefix.clone(),
                mission_settings.mission_name.clone(),
                map.clone(),
                &mission_settings,
                &build_config,
            )
            .ok()
        })
        .collect())
}

struct Mission {
    map_name: String,
    mission_name: String,
    prefix: String,

    sqm: Config,
}

impl Mission {
    #[instrument(skip(mission_settings), err)]
    pub fn new(
        prefix: String,
        mission_name: String,
        map_name: String,
        mission_settings: &MissionSettings,
        build_config: &BuildContext,
    ) -> Result<Self> {
        let handlebars = create_handlebars()?;

        #[derive(Serialize)]
        struct MissionTemplate {
            author: String,
            respawn_delay: usize,
            mission_name: String,
        }

        let template = MissionTemplate {
            author: build_config
                .extra
                .get("author")
                .map(|v| v.to_string())
                .unwrap_or_default(),
            mission_name: mission_name.clone(),
            respawn_delay: mission_settings.respawn_delay,
        };

        let sqm = handlebars.render("mission.sqm", &template)?;

        let config = Config::read(&mut sqm.as_bytes(), None, &Vec::new())?;

        Ok(Mission {
            map_name,
            mission_name,
            prefix,
            sqm: config,
        })
    }

    #[instrument(skip(self, composition))]
    pub fn merge_composition(&mut self, composition: &Composition) -> Result<()> {
        let items = composition.get_offseted_items()?;

        let mut class = self.sqm.inner_mut();

        // Mission.Entities = items

        class.entries = class.entries.clone().map(|entries| {
            entries.into_iter().map(|(name, config)| {
                if name == "Mission" {
                    if let ConfigEntry::ClassEntry(mut mission) = config {
                        mission.entries = mission.entries.map(|entries| {
                            let mut map: HashMap<String, ConfigEntry> = entries.into_iter().collect();
                            let entities = ConfigEntry::ClassEntry(ConfigClass {
                                parent: "Mission".to_string(),
                                is_external: false,
                                is_deletion: false,
                                entries: Some(items.clone())
                            });

                            map.insert("Entities".to_string(), entities);

                            map.into_iter().collect()
                        });

                        return (name, ConfigEntry::ClassEntry(mission));
                    }
                }

                (name, config)
            }).collect()
        });

        Ok(())
    }

    /// Convert this mission to SQM
    pub fn to_sqm(&self) -> Result<String> {
        let mut buffer = Vec::new();
        self.sqm.write(&mut buffer)?;

        Ok(std::str::from_utf8(&buffer)?.to_string())
    }

    /// Return the class_name for this mission
    pub fn mission_name(&self) -> String {
        format!("{}.{}", self.class_name(), self.map_name)
    }

    pub fn class_name(&self) -> String {
        format!("{}_{}{}", self.prefix, self.map_name, self.mission_name,)
    }
}

#[derive(Serialize)]
struct Addon {
    prefix: String,
    addon_name: String,
    missions: Vec<MissionClass>,
}

impl Addon {
    pub fn from_parts(
        prefix: String,
        addon_name: String,
        missions: Vec<(PathBuf, Mission)>,
    ) -> Self {
        let missions = missions
            .into_iter()
            .map(|(directory, mission)| {
                let directory = format!(
                    r"{}\{}\missions\{}",
                    prefix,
                    addon_name,
                    directory
                        .parent()
                        .map(|p| p.file_name().map(|p| p.to_string_lossy().to_owned()))
                        .flatten()
                        .unwrap()
                );
                MissionClass {
                    briefing_name: format!("[{}] {}", prefix, mission.class_name()),
                    class_name: mission.class_name(),
                    directory,
                }
            })
            .collect();

        Addon {
            prefix,
            addon_name,
            missions,
        }
    }
}

#[derive(Serialize)]
struct MissionClass {
    class_name: String,
    briefing_name: String,
    directory: String,
}