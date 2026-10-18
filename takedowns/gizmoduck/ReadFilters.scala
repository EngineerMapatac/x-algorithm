package com.twitter.gizmoduck.modules.filters

import com.twitter.gizmoduck.modules.filters.chain.FilterChain
import com.twitter.gizmoduck.modules.filters.providers.FiltersProvider
import com.twitter.gizmoduck.request.RequestFilter

class ReadFilters(filtersProvider: FiltersProvider) extends FilterChain {
  import filtersProvider._

  override def filters: RequestFilter = RequestFilter.merged(
    queryFieldsFilter,
    userTypeFilter,
    perspectivalFilter,
    protectedUserFilter,
    dynamicVisibilityFilter,
    cleanlyHandleExceptions,
    blueVerifiedApplyExpirationAndVerification,
    createdAtRepairFilter,
    hideLikesOnProfileFilter,
    legacyVerifiedRemovalFilter,
    redactUserFields,
    deciderableBlackholeLargeReads,
    deciderableLogRequest,
    rateLimitByClient,
    verifyCredentials,
    verifyAccess,
    observeRequest,
    deciderableStratoRequestAttributionCounter,
    deciderableAnnotateTestUserRequests,
    userIdFilter,
    userStringFilter,
    handleEmptyReads,
    deciderableChunkReads,
    filterAnnotations,
    loadShedFilter,
    allowDmsFromFilter,
    redactUnenforcedTakedownsFilter
  )
}
