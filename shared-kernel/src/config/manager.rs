use super::Config;

pub struct ConfigManager {

    config: Config,
}

impl ConfigManager {

    pub fn load() -> Self {

        Self {

            config: Config {

                product_name: "DEZH".into(),

                version: "0.1.0-alpha".into(),

                debug: true,
            },
        }
    }

    pub fn config(&self) -> &Config {

        &self.config
    }
}