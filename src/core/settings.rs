use super::agt::indicators::common::OptimizationParam;
use dotenv;
use std::env;
pub struct Settings {
    pub mongodb_uri: String,
    pub redis_uri: String,
    pub rust_log: String,
    pub rust_backtrace: String,
    pub optimization_param: OptimizationParam,
    pub env_optimization: String,
}

impl Settings {
    pub fn new(env: String) -> Settings {
        let mut result = None;
        if env == "local".to_string() {
            dotenv::from_filename("./.env.local").ok();
            for (key, value) in env::vars() {
                println!("{}: {}", key, value);
            }
            result = Some(Settings {
                mongodb_uri: env::var("MONGODB_URI")
                    .expect("MONGODB_URI env var should be specified"),
                redis_uri: env::var("REDIS_URI").expect("REDIS_URI env var should be specified"),
                rust_log: env::var("RUST_LOG").expect("RUST_LOG env var should be specified"),
                rust_backtrace: env::var("RUST_BACKTRACE")
                    .expect("RUST_BACKTRACE env var should be specified"),
                optimization_param: OptimizationParam {
                    start: env::var("start")
                        .expect("OPTIMIZATION_PARAM env var should be specified")
                        .parse()
                        .unwrap(),
                    stop: env::var("stop")
                        .expect("OPTIMIZATION_PARAM env var should be specified")
                        .parse()
                        .unwrap(),
                    step: env::var("step")
                        .expect("OPTIMIZATION_PARAM env var should be specified")
                        .parse()
                        .unwrap(),
                },
                env_optimization: env::var("ENV_OPTIMIZATION")
                    .expect("env_optimization env var should be specified")
                    .parse()
                    .unwrap(),
            });
        } else if env == "docker".to_string() {
            dotenv::from_filename(".env.docker").ok();
            result = Some(Settings {
                mongodb_uri: env::var("MONGODB_URI")
                    .expect("MONGODB_URI env var should be specified"),
                redis_uri: env::var("REDIS_URI").expect("REDIS_URI env var should be specified"),
                rust_log: env::var("RUST_LOG").expect("RUST_LOG env var should be specified"),
                rust_backtrace: env::var("RUST_BACKTRACE")
                    .expect("RUST_BACKTRACE env var should be specified"),
                optimization_param: OptimizationParam {
                    start: env::var("start")
                        .expect("OPTIMIZATION_PARAM env var should be specified")
                        .parse()
                        .unwrap(),
                    stop: env::var("stop")
                        .expect("OPTIMIZATION_PARAM env var should be specified")
                        .parse()
                        .unwrap(),
                    step: env::var("step")
                        .expect("OPTIMIZATION_PARAM env var should be specified")
                        .parse()
                        .unwrap(),
                },
                env_optimization: env::var("ENV_OPTIMIZATION")
                    .expect("env_optimization env var should be specified")
                    .parse()
                    .unwrap(),
            });
        } else {
            println!("{:}", "Неправильно указанно окружение")
        }
        return result.expect("Загрузка env закончилось ошибкой");
    }
}
