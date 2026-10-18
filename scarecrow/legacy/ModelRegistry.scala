package com.twitter.botmaker.app.scarecrow.legacy

import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.spam.modelprovider.Model
import com.twitter.botmaker.common.util.NullObjectPattern

object ModelRuntime {
  val empty: ModelRuntime = NullObjectPattern.newNullObject[ModelRuntime]()
}

trait ModelRuntime {

  def containsModel(modelId: String): Boolean
  def getModel[T <: Model](modelId: String): T
}

trait ModelScarecrowRuntime extends ModelRuntime { self: ScarecrowRuntime =>

  override def containsModel(modelId: String): Boolean = {
    models.containsModel(modelId)
  }

  override def getModel[T <: Model](modelId: String): T = {
    models.getModel[T](modelId)
  }
}
