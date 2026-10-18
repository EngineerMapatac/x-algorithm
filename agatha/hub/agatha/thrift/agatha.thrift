namespace java com.twitter.hub.agatha.thriftjava

include "featureSource.thrift"

struct FeatureClass {
  1: required featureSource.FeatureSource source
  2: string subclass
}(hasPersonalData = 'false', persisted = 'true')

struct FeatureIdentifier {
  1: required FeatureClass featureClass
  2: required string name
}(hasPersonalData = 'false', persisted = 'true')

struct Feature {
  1: FeatureIdentifier identifier
  2: double value
}(hasPersonalData = 'false', persisted = 'true')

struct UserFeature {
  1: required i64 userId(personalDataType = 'UserId')
  2: required Feature feature
}(hasPersonalData = 'true', persisted = 'true')

struct LabelDateRange {
  1: i64 startTimestamp
  2: i64 endTimestamp
}(hasPersonalData = 'false', persisted = 'true')

struct ModelledFeatureLabelMetadata {
  1: map<string, LabelDateRange> labelDateRangeMap
}(hasPersonalData = 'false')

struct ModelledFeature {
  1: required FeatureIdentifier identifier
  2: required string label
  3: double sumOfUnionProduct
  4: double sumOfSquaredUnionProduct
  5: double sumOfFeatureIdentifierValues
  6: double sumOfLabelValues
  7: i64 totalUniqueUserIds
  8: double pmi
}(hasPersonalData = 'false', persisted = 'true')

struct UserLabel {
  1: required i64 userId(personalDataType = 'UserId')
  2: required string label
  3: required i32 positiveCount
  4: required i32 totalCount
  5: optional double smoothedRatio
}(hasPersonalData = 'true', persisted = 'true')

