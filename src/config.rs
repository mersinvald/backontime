use toml;
use std::path::PathBuf;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use log::LogLevel;
use global::GLOBAL;

use backup_entity::BackupEntity;

#[derive(Debug)]
pub struct Config {
    pub backups: Vec<BackupEntity>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path.as_ref())?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        let config_model: ConfigModel = toml::from_str(&buffer)?;
        config_model.into_config()
    }
}


#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename = "config")]
struct ConfigModel {
    #[serde(rename = "backup")]
    backups: Vec<BackupModel>,
    verbosity: Option<String>,
}

impl ConfigModel {
    fn into_config(self) -> Result<Config> {
        let verbosity = match self.verbosity {
            Some(level) => match level.to_lowercase().as_ref() {
                "error" => LogLevel::Error,
                "warn" => LogLevel::Warn,
                "info" => LogLevel::Info,
                "debug" => LogLevel::Debug,
                "trace" => LogLevel::Trace,
                _ => bail!(ErrorKind::UnknownVariant(level))
            },
            None => LogLevel::Info
        };

        GLOBAL.lock().unwrap()
              .setup_logger(verbosity);

        let mut backups = Vec::new();
        for mut backup_model in self.backups {
            backup_model.set_defaults()?;
            backups.push(backup_model.into())
        }

        Ok(Config {
            backups
        })
    }
}

#[derive(Deserialize)]
#[serde(rename = "backup")]
#[serde(deny_unknown_fields)]
struct BackupModel {
    path: PathBuf,
    name: Option<String>,
    recursive: Option<bool>,
    changes: Option<u64>,
    timer: Option<u64>,
    exec: String,
}

impl Into<BackupEntity> for BackupModel {
    fn into(self) -> BackupEntity {
        let entity = BackupEntity {
            path: self.path,
            recursive: self.recursive.unwrap(),
            trigger_changes: self.changes.unwrap_or(0),
            trigger_timer: self.timer.unwrap_or(0),
            exec: self.exec,
            ..Default::default()
        };

        info!("Registered entity:");
        info!("path:      {:?}", entity.path.display());
        info!("recursive: {}", entity.recursive);
        info!("changes:   {}", entity.trigger_changes);
        info!("timer:     every {} minute", entity.trigger_timer);
        info!("exec:      {}", entity.exec);
        
        entity
    }
}

impl BackupModel {
    fn set_defaults(&mut self) -> Result<()> {
        self.set_alias_or_default()?;
        self.set_recursive_or_default()?;
        self.substitute_exec_variables()?;

        if self.changes.is_none() && self.timer.is_none() {
            bail!(ErrorKind::MissingRequiredField("both \"timer\" and \"changes\" are unset".to_owned()));
        }

        Ok(())
    }

    fn set_alias_or_default(&mut self) -> Result<()> {
        let last_component = match self.path.components().last() {
            Some(last) => last,
            None => bail!(ErrorKind::PathParseError(self.path.clone()))
        };
          
        self.name = Some(last_component.as_ref()
            .to_str()
            .ok_or_else(|| ErrorKind::PathParseError(self.path.clone()))?
            .to_owned());
        
        Ok(())
    }

    fn set_recursive_or_default(&mut self) -> Result<()> {
        let recursive = match self.recursive {
            Some(true) => {
                if self.path.is_file() {
                    warn!("{} is a file, but recursive is \"true\"", self.path.display());
                    false
                } else {
                    true
                }
            },
            Some(false) => {
                if self.path.is_dir() {
                    warn!("{} is a directory, but recursive is \"false\"", self.path.display());
                }
                false
            },
            None => self.path.is_dir()
        };
        
        Ok(self.recursive = Some(recursive))
    }

    fn substitute_exec_variables(&mut self) -> Result<()> {
        self.exec = {
            let path = format!("{}", self.path.display());
            let name = self.name.as_ref().unwrap();
            self.exec.replace("{{path}}", &path)
                     .replace("{{name}}", name)
        };

        Ok(())
    }
}

error_chain! {
    foreign_links {
        Toml(::toml::de::Error);
        Io(::std::io::Error);
    }

    errors {
        PathParseError(path: PathBuf) {
            description("invalid toolchain name")
            display("{:?} is not a valid path", path.display())
        }

        MissingRequiredField(what: String) {
            description("invalid setting: missing required field")
            display("invalid setting: {}", what)
        }

        UnknownVariant(what: String) {
            description("unknown variant")
            display("unknown variant: {:?}", what)
        }
    }
}



