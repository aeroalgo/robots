from pymongo import MongoClient

def apply_migration(db):
    db.create_collection('strategy_configs', validator={
        '$jsonSchema': {
            'bsonType': 'object',
            'required': ['strategy_id', 'config', 'version'],
            'properties': {
                'strategy_id': {'bsonType': 'string'},
                'strategy_name': {'bsonType': 'string'},
                'config': {'bsonType': 'object'},
                'version': {'bsonType': 'int'},
                'indicators': {'bsonType': 'array'},
                'entry_conditions': {'bsonType': 'object'},
                'exit_conditions': {'bsonType': 'object'},
                'risk_management': {'bsonType': 'object'},
                'created_at': {'bsonType': 'date'},
                'updated_at': {'bsonType': 'date'}
            }
        }
    })
    db.strategy_configs.create_index([('strategy_id', 1)], unique=True)
    db.strategy_configs.create_index([('strategy_name', 1)])
    db.strategy_configs.create_index([('created_at', -1)])

    db.create_collection('indicator_metadata', validator={
        '$jsonSchema': {
            'bsonType': 'object',
            'required': ['indicator_id', 'name', 'category'],
            'properties': {
                'indicator_id': {'bsonType': 'string'},
                'name': {'bsonType': 'string'},
                'category': {'bsonType': 'string', 'enum': ['trend', 'momentum', 'volatility', 'volume', 'custom', 'ml']},
                'description': {'bsonType': 'string'},
                'parameters': {'bsonType': 'array'},
                'default_params': {'bsonType': 'object'},
                'computation_complexity': {'bsonType': 'string', 'enum': ['low', 'medium', 'high', 'very_high']},
                'supports_simd': {'bsonType': 'bool'},
                'version': {'bsonType': 'string'}
            }
        }
    })
    db.indicator_metadata.create_index([('indicator_id', 1)], unique=True)
    db.indicator_metadata.create_index([('category', 1)])
    db.indicator_metadata.create_index([('name', 1)])

    db.create_collection('system_logs', validator={
        '$jsonSchema': {
            'bsonType': 'object',
            'required': ['timestamp', 'level', 'message'],
            'properties': {
                'timestamp': {'bsonType': 'date'},
                'level': {'bsonType': 'string', 'enum': ['debug', 'info', 'warn', 'error', 'critical']},
                'service': {'bsonType': 'string'},
                'message': {'bsonType': 'string'},
                'context': {'bsonType': 'object'},
                'error_stack': {'bsonType': 'string'}
            }
        }
    })
    db.system_logs.create_index([('timestamp', -1)])
    db.system_logs.create_index([('level', 1), ('timestamp', -1)])
    db.system_logs.create_index([('service', 1), ('timestamp', -1)])

    db.create_collection('event_store', validator={
        '$jsonSchema': {
            'bsonType': 'object',
            'required': ['event_id', 'event_type', 'timestamp'],
            'properties': {
                'event_id': {'bsonType': 'string'},
                'event_type': {'bsonType': 'string'},
                'aggregate_id': {'bsonType': 'string'},
                'aggregate_type': {'bsonType': 'string'},
                'payload': {'bsonType': 'object'},
                'metadata': {'bsonType': 'object'},
                'timestamp': {'bsonType': 'date'},
                'user_id': {'bsonType': 'string'}
            }
        }
    })
    db.event_store.create_index([('event_id', 1)], unique=True)
    db.event_store.create_index([('aggregate_id', 1), ('timestamp', 1)])
    db.event_store.create_index([('event_type', 1), ('timestamp', -1)])

    db.create_collection('ml_models', validator={
        '$jsonSchema': {
            'bsonType': 'object',
            'required': ['model_id', 'model_type', 'version'],
            'properties': {
                'model_id': {'bsonType': 'string'},
                'model_name': {'bsonType': 'string'},
                'model_type': {'bsonType': 'string', 'enum': ['classifier', 'regressor', 'clustering', 'reinforcement']},
                'architecture': {'bsonType': 'string'},
                'hyperparameters': {'bsonType': 'object'},
                'training_config': {'bsonType': 'object'},
                'performance_metrics': {'bsonType': 'object'},
                'version': {'bsonType': 'string'},
                'status': {'bsonType': 'string', 'enum': ['training', 'trained', 'deployed', 'archived']},
                'trained_at': {'bsonType': 'date'},
                'deployed_at': {'bsonType': 'date'}
            }
        }
    })
    db.ml_models.create_index([('model_id', 1)], unique=True)
    db.ml_models.create_index([('status', 1)])
    db.ml_models.create_index([('trained_at', -1)])

    db.create_collection('genetic_algorithm_config', validator={
        '$jsonSchema': {
            'bsonType': 'object',
            'required': ['config_id', 'algorithm_type'],
            'properties': {
                'config_id': {'bsonType': 'string'},
                'algorithm_type': {'bsonType': 'string', 'enum': ['simple_ga', 'nsga2', 'nsga3', 'custom']},
                'population_size': {'bsonType': 'int'},
                'generations': {'bsonType': 'int'},
                'mutation_rate': {'bsonType': 'double'},
                'crossover_rate': {'bsonType': 'double'},
                'selection_method': {'bsonType': 'string', 'enum': ['tournament', 'roulette', 'rank', 'elitist']},
                'fitness_function': {'bsonType': 'object'},
                'objectives': {'bsonType': 'array'}
            }
        }
    })
    db.genetic_algorithm_config.create_index([('config_id', 1)], unique=True)
    
    print('MongoDB collections and indexes created successfully!')



