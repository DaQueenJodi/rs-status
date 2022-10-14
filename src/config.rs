use std::{fs::File, io::BufReader};

use serde::{Deserialize, Serialize};

pub fn read_config() -> ConfigFile {
    let config_path = xdg::BaseDirectories::with_prefix("rs-status").unwrap();
    let path = config_path.get_config_file("config.yaml");
    let reader = BufReader::new(File::open(path).unwrap());

    let mut config: ConfigFile = serde_yaml::from_reader(reader).unwrap();

    for module in &mut config.modules {
        if module.flavor == ModuleFlavor::Script {
            let string = config_path
                .get_config_home()
                .into_os_string()
                .into_string()
                .unwrap()
                + "/Scripts/"
                + &module.command;
            module.command = string;
        }
    }
    config
}

#[derive(Deserialize, Serialize)]
pub struct ConfigFile {
    pub seperator: String,
    pub modules: Vec<ModuleConfig>,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct ModuleConfig {
    flavor: ModuleFlavor,
    pub interval: u64,
    pub command: String,
}

#[derive(Deserialize, PartialEq, Clone, Serialize)]
enum ModuleFlavor {
    Command,
    Script,
}
