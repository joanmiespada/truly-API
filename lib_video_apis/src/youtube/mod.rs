use lib_config::{environment::EnvironmentVariables, config::Config};




#[derive(Clone, Debug)]
pub struct YoutubeAPI {
    //client: Client,
    environment_vars: EnvironmentVariables,
}

impl YoutubeAPI {
    pub fn new(conf: &Config) -> YoutubeAPI {
        YoutubeAPI {
            //client: Client::new(conf.aws_config()),
            environment_vars: conf.env_vars().clone(),
        }
    }
}
