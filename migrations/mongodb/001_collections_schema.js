db = db.getSiblingDB('trading_meta');

db.createCollection('strategy_configs', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['strategy_id', 'config', 'version'],
            properties: {
                strategy_id: {
                    bsonType: 'string',
                    description: 'Уникальный идентификатор стратегии'
                },
                strategy_name: {
                    bsonType: 'string',
                    description: 'Название стратегии'
                },
                config: {
                    bsonType: 'object',
                    description: 'Конфигурация стратегии в JSON формате'
                },
                version: {
                    bsonType: 'int',
                    description: 'Версия конфигурации'
                },
                indicators: {
                    bsonType: 'array',
                    description: 'Список используемых индикаторов'
                },
                entry_conditions: {
                    bsonType: 'object',
                    description: 'Условия входа в позицию'
                },
                exit_conditions: {
                    bsonType: 'object',
                    description: 'Условия выхода из позиции'
                },
                risk_management: {
                    bsonType: 'object',
                    description: 'Настройки управления рисками'
                },
                created_at: {
                    bsonType: 'date'
                },
                updated_at: {
                    bsonType: 'date'
                }
            }
        }
    }
});

db.createCollection('indicator_metadata', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['indicator_id', 'name', 'category'],
            properties: {
                indicator_id: {
                    bsonType: 'string',
                    description: 'Уникальный идентификатор индикатора'
                },
                name: {
                    bsonType: 'string',
                    description: 'Название индикатора'
                },
                category: {
                    bsonType: 'string',
                    enum: ['trend', 'momentum', 'volatility', 'volume', 'custom', 'ml'],
                    description: 'Категория индикатора'
                },
                description: {
                    bsonType: 'string',
                    description: 'Описание индикатора'
                },
                parameters: {
                    bsonType: 'array',
                    description: 'Параметры индикатора'
                },
                default_params: {
                    bsonType: 'object',
                    description: 'Значения параметров по умолчанию'
                },
                computation_complexity: {
                    bsonType: 'string',
                    enum: ['low', 'medium', 'high', 'very_high'],
                    description: 'Сложность вычисления'
                },
                supports_simd: {
                    bsonType: 'bool',
                    description: 'Поддержка SIMD оптимизации'
                },
                version: {
                    bsonType: 'string'
                }
            }
        }
    }
});

db.createCollection('system_logs', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['timestamp', 'level', 'message'],
            properties: {
                timestamp: {
                    bsonType: 'date'
                },
                level: {
                    bsonType: 'string',
                    enum: ['debug', 'info', 'warn', 'error', 'critical']
                },
                service: {
                    bsonType: 'string'
                },
                message: {
                    bsonType: 'string'
                },
                context: {
                    bsonType: 'object'
                },
                error_stack: {
                    bsonType: 'string'
                }
            }
        }
    }
});

db.createCollection('event_store', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['event_id', 'event_type', 'timestamp'],
            properties: {
                event_id: {
                    bsonType: 'string'
                },
                event_type: {
                    bsonType: 'string'
                },
                aggregate_id: {
                    bsonType: 'string'
                },
                aggregate_type: {
                    bsonType: 'string'
                },
                payload: {
                    bsonType: 'object'
                },
                metadata: {
                    bsonType: 'object'
                },
                timestamp: {
                    bsonType: 'date'
                },
                user_id: {
                    bsonType: 'string'
                }
            }
        }
    }
});

db.createCollection('ml_models', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['model_id', 'model_type', 'version'],
            properties: {
                model_id: {
                    bsonType: 'string'
                },
                model_name: {
                    bsonType: 'string'
                },
                model_type: {
                    bsonType: 'string',
                    enum: ['classifier', 'regressor', 'clustering', 'reinforcement']
                },
                architecture: {
                    bsonType: 'string'
                },
                hyperparameters: {
                    bsonType: 'object'
                },
                training_config: {
                    bsonType: 'object'
                },
                performance_metrics: {
                    bsonType: 'object'
                },
                version: {
                    bsonType: 'string'
                },
                status: {
                    bsonType: 'string',
                    enum: ['training', 'trained', 'deployed', 'archived']
                },
                trained_at: {
                    bsonType: 'date'
                },
                deployed_at: {
                    bsonType: 'date'
                }
            }
        }
    }
});

db.createCollection('genetic_algorithm_config', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['config_id', 'algorithm_type'],
            properties: {
                config_id: {
                    bsonType: 'string'
                },
                algorithm_type: {
                    bsonType: 'string',
                    enum: ['simple_ga', 'nsga2', 'nsga3', 'custom']
                },
                population_size: {
                    bsonType: 'int'
                },
                generations: {
                    bsonType: 'int'
                },
                mutation_rate: {
                    bsonType: 'double'
                },
                crossover_rate: {
                    bsonType: 'double'
                },
                selection_method: {
                    bsonType: 'string',
                    enum: ['tournament', 'roulette', 'rank', 'elitist']
                },
                fitness_function: {
                    bsonType: 'object'
                },
                objectives: {
                    bsonType: 'array'
                }
            }
        }
    }
});

db.strategy_configs.createIndex({ strategy_id: 1 }, { unique: true });
db.strategy_configs.createIndex({ strategy_name: 1 });
db.strategy_configs.createIndex({ created_at: -1 });

db.indicator_metadata.createIndex({ indicator_id: 1 }, { unique: true });
db.indicator_metadata.createIndex({ category: 1 });
db.indicator_metadata.createIndex({ name: 1 });

db.system_logs.createIndex({ timestamp: -1 });
db.system_logs.createIndex({ level: 1, timestamp: -1 });
db.system_logs.createIndex({ service: 1, timestamp: -1 });

db.event_store.createIndex({ event_id: 1 }, { unique: true });
db.event_store.createIndex({ aggregate_id: 1, timestamp: 1 });
db.event_store.createIndex({ event_type: 1, timestamp: -1 });

db.ml_models.createIndex({ model_id: 1 }, { unique: true });
db.ml_models.createIndex({ status: 1 });
db.ml_models.createIndex({ trained_at: -1 });

db.genetic_algorithm_config.createIndex({ config_id: 1 }, { unique: true });

print('MongoDB collections and indexes created successfully!');

