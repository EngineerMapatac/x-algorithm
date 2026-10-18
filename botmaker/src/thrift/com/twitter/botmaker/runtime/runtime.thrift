namespace java com.twitter.botmaker.runtime.thriftjava

include "com/twitter/relevance/feature_store/fs_common.thrift"


struct BotMakerStat {
  1: required string eventType
  2: required binary payload
  3: optional string producer
  4: optional i64 seqNum
}(persisted='true')

struct BotMakerStats {
  1: required string group
  2: required list<BotMakerStat> stats
  3: required i64 timestamp

  10: optional string appName
  11: optional string serviceName
  12: optional string environment
  13: optional string dataCenter
  14: optional i64 shardId
}(persisted='true')

struct BotMakerFunctionCall {
  1: required string functionName
  2: required i64 ruleId
  3: required i64 timestamp

  6: optional fs_common.FeatureValue arg0 (personalDataType='OpaqueLowSensitivityData,OpaqueMediumSensitivityData,OpaqueHighSensitivityData')
  7: optional fs_common.FeatureValue arg1 (personalDataType='OpaqueLowSensitivityData,OpaqueMediumSensitivityData,OpaqueHighSensitivityData')
  8: optional fs_common.FeatureValue arg2 (personalDataType='OpaqueLowSensitivityData,OpaqueMediumSensitivityData,OpaqueHighSensitivityData')
  9: optional fs_common.FeatureValue arg3 (personalDataType='OpaqueLowSensitivityData,OpaqueMediumSensitivityData,OpaqueHighSensitivityData')
  10: optional fs_common.FeatureValue arg4 (personalDataType='OpaqueLowSensitivityData,OpaqueMediumSensitivityData,OpaqueHighSensitivityData')
  11: optional fs_common.FeatureValue arg5 (personalDataType='OpaqueLowSensitivityData,OpaqueMediumSensitivityData,OpaqueHighSensitivityData')
  12: optional fs_common.FeatureValue arg6 (personalDataType='OpaqueLowSensitivityData,OpaqueMediumSensitivityData,OpaqueHighSensitivityData')
  13: optional fs_common.FeatureValue arg7 (personalDataType='OpaqueLowSensitivityData,OpaqueMediumSensitivityData,OpaqueHighSensitivityData')
  14: optional fs_common.FeatureValue arg8 (personalDataType='OpaqueLowSensitivityData,OpaqueMediumSensitivityData,OpaqueHighSensitivityData')
  15: optional fs_common.FeatureValue arg9 (personalDataType='OpaqueLowSensitivityData,OpaqueMediumSensitivityData,OpaqueHighSensitivityData')
  16: optional fs_common.FeatureValue arg10 (personalDataType='OpaqueLowSensitivityData,OpaqueMediumSensitivityData,OpaqueHighSensitivityData')
  17: optional fs_common.FeatureValue arg11 (personalDataType='OpaqueLowSensitivityData,OpaqueMediumSensitivityData,OpaqueHighSensitivityData')
}(persisted='true', hasPersonalData='true')

struct BotMakerLog {
  1: required string eventType;
  2: required i64 timestamp;

  10: optional string appName
  11: optional string serviceName
  12: optional string environment
  13: optional string dataCenter
  14: optional i64 shardId

  20: optional set<i64> violatedRules
  21: optional set<i64> unviolatedRules
  22: optional set<i64> abortedRules
  23: optional set<i64> failedRules
  24: optional set<i64> timeoutRules

  30: optional map<string, binary> inputFeatures
  31: optional map<string, binary> derivedFeatures
  32: optional map<string, binary> outputFeatures
}(persisted='true')

struct RegexConfig {
  1: required i64 timeoutMilis = 1000
  2: required i64 maximumPendingRequests = 20
  3: required i64 maximumCacheItems = 1000
  4: required i64 cacheExpirationMillis = 1800000
}

struct ThriftFieldConfig {
  1: required string fieldName
  2: required string description = ""
  3: required string fieldType
  4: required i32 fieldId
  5: optional bool isRequired
}

struct BotMakerEventTypeConfig {
  1: required string eventType
  2: required string description = ""
  3: required list<ThriftFieldConfig> inputFeatures = []
  4: optional bool strict = false
}

struct BatchQueueConfig {
  2: required string description = ""
  3: required string itemType
  4: required string eventType
  5: required i32 batchSize = 16
  6: required i32 queueSize = 1024
  7: required i32 concurrentSize = 1
  9: optional string returnType = ""
}

struct LocalAggregatorConfig {
  1: required string eventType
  2: required string description = ""
  3: required list<string> keyTypes
  4: optional string valueType = ""
  5: optional map<string,string> options

  11: optional i32 flushIntervalMillis = 60000
  12: optional i32 flushBatchSize = 4
  13: optional i32 flushThreshold = 1
  14: optional i32 flushSlices = 4
  15: optional i32 maximumCacheItems = 50000
  16: optional i32 cacheExpirationMillis = 3600000
}(persisted='true')

struct LogWriterConfig {
  1: required string funcName
  2: required string description = ""
  3: required string rootDirectory = "."
  4: required i32 maxLogFileSize = 10485760
  5: required i32 logFilesToKeep = 16
  6: required i32 batchSize = 16
  7: required i32 queueSize = 1024
}

struct EventPublisherConfig {
  1: required string funcName
  2: required string description = ""
  3: required string thriftClass
  4: required string clientId
  5: required string streamName
  6: optional i32 maxQueuedEvents = 100
  7: optional list<string> partitionFields = []
  8: optional string kafkaDest
}

struct EventSubscriberConfig {
  1: optional string subscriberId
  2: required string description = ""
  3: required string thriftClass
  4: required string eventType
  10: required i32 maxConcurrentEvents = 100
  11: required i32 maxUnackedEvents = 10000
  12: required i32 processTimeoutMillis = 60000
  13: required bool skipToLatest = false
  14: optional bool fromAllZones = false
  15: optional i32 delayMillis = 0
  16: optional i32 groupId
  17: optional i32 numThreads = 1
  18: optional string streamName
  19: optional string kafkaDest
  20: optional string kafkaGroupId
}

struct KafkaFuturePoolConfig {
  1: required i32 timeoutMs = 10000

  2: required i32 corePoolSize = 2

  3: required i32 maxPoolSize = 6

  4: required i32 keepAliveTimeMillis = 60000

  5: required i32 queueCapacity = 524288
}

struct KafkaProducerConfig {
  1: required string funcName
  2: required string topicName
  3: required string dest
  4: required string description = ""
  5: required string keyType
  6: required string valueThriftClass
  7: required string clientId
  11: optional bool includeNodeMetrics = false
  12: optional map<string, string> configMap = {}
  13: optional KafkaFuturePoolConfig futurePoolConfig
}

struct KafkaConsumerConfig {
  1: required string eventType
  2: required string topicName
  3: required string groupId
  4: required string dest
  5: required string description = ""
  6: required string keyType
  7: required string valueThriftClass
  8: required string clientId
  10: required i64 pollTimeoutMillis = 100
  11: required string seekStrategy
  12: optional i64 rewindDurationMillis
  13: optional bool includeNodeMetrics = false
  14: optional map<string, string> configMap = {}
}

struct ScribePublisherConfig {
  1: required string funcName
  2: required string description = ""
  3: required string thriftEndpoint
  4: required string thriftClass
  5: required string category
}

struct ThriftMethodConfig {
  1: required string funcName
  2: required string description = ""
  3: required string methodName
  10: optional i64 timeoutMillis
}

struct ThriftEndpointConfig {
  1: required string endpoint
  2: required string description = ""
  3: required string clientId
  4: required string servicePath
  5: required string thriftClass
  6: optional i64 requestTimeoutMillis
  7: optional i64 sessionTimeoutMillis
  10: optional list<ThriftMethodConfig> methods = []
}

struct HttpEndpointConfig {
  1: required string endpoint = "httpClient"
  2: required string description = ""
  3: required string host = "httpproxy.local.twitter.com"
  4: required i32 port = 3128
  5: required bool withTls = false
  6: optional string proxyCredentialPath
  7: optional string proxyUserName
  8: optional string proxyUserPassword
  9: optional i64 requestTimeoutMillis
  10: optional i64 sessionTimeoutMillis
}

struct ThriftTypeConfig {
  1: required string className
  2: required string typeName
}

struct StructTupleFieldConfig {
  1: required string fieldName
  2: required string fieldType
  3: required string description = ""
}

struct StructTupleTypeConfig {
  1: required string typeName
  2: required list<StructTupleFieldConfig> fields
}

struct PackageConfig {
  1: required string packageName
  2: required string description = ""
}

struct ManhattanEndpointConfig {
  1: required string endpoint
  2: required string description = ""
  3: required string servicePath
  4: required string appId
  5: required i32 defaultTimeoutMillis = 100
  6: required i32 maxRetries = 2
}

enum ThriftProtocol {
  COMPACT = 0,
  BINARY = 1
}


struct ManhattanClientConfig {
  1: required string funcName
  2: required string endpoint
  3: required string description = ""
  4: required string dataset
  5: required list<string> keyTypes
  6: required string valueType
  7: optional list<string> localKeyTypes = []
  8: optional i64 ttlMillis
  9: optional bool allowNulls = false
  10: optional ThriftProtocol thriftProtocol = ThriftProtocol.COMPACT
}

struct MemCachedEndpointConfig {
  1: required string endpoint
  2: required string label
  3: required string servicePath
  4: required bool ejectFailedHost = false
  5: required i32 timeoutFilterMilis = 200
  6: required i32 timeoutFactoryMilis = 200
  7: required i32 numFailuresFailureAccrual = 30
  8: required i32 markDeadForMilisFailuresFailureAccrual = 1000
}

struct MemCachedClientConfig {
  1: required string funcName
  2: required string endpoint
  3: required string description = ""
  4: required string dataset
  5: required list<string> keyTypes
  6: required string valueType
}

struct MemCachedCounterConfig {
  1: required string funcName
  2: required string endpoint
  3: required string description = ""
  4: required string dataset
  5: required list<string> keyTypes
}

struct LocalCounterConfig {
  1: required string funcName
  2: required string description = ""
  3: required list<string> keyTypes = []
  5: required i32 maximumCacheItems = 50000
  6: required i32 cacheExpirationMillis = 3600000
}

struct LocalStoreConfig {
  1: required string funcName
  2: required string description = ""
  3: required list<string> keyTypes = []
  4: required string valueType
  5: required i32 maximumCacheItems = 50000
  6: required i32 cacheExpirationMillis = 3600000
}

struct ScheduledEventConfig {
  1: required string eventType
  2: required string description = ""
  3: required i32 intervalMillis
}

struct TeamConfig {
  1: required string name

  2: required string email

  3: optional string pagerdutyId

  4: optional map<string, string> additionalFeatures
}


struct BotActionHaltedState {
  1: required bool isHalted
}(persisted='true')
