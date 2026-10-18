package com.twitter.botmaker

import java.util.{List => JList}
import scala.collection.JavaConverters._
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.compiler.BotMakerDoc
import com.twitter.botmaker.compiler.Fingerprint
import com.twitter.util.Future
import com.twitter.botmaker.compiler.types.ScalaType._
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function._

abstract class FunctionUnit0V[E, V, R](
  implicit v: Manifest[V],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](),
    ofType[V],
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  val toScalaV: AnyRef => Any = asScala[V]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(context: Context[E], vargs: Seq[V]): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = FunctionUnit0V.this
              .evaluate(context, args.asScala.drop(0).map(toScalaV(_).asInstanceOf[V]))
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit0V.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = Future(
              FunctionUnit0V.this
                .evaluate(context, args.asScala.drop(0).map(toScalaV(_).asInstanceOf[V])))
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = FunctionUnit0V.this
              .evaluate(context, args.asScala.drop(0).map(toScalaV(_).asInstanceOf[V]))
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit0[E, R](
  implicit r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](),
    ImmutableList.of[Type](),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(context: Context[E]): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = FunctionUnit0.this.evaluate(context)
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit0.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = Future(FunctionUnit0.this.evaluate(context))
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = FunctionUnit0.this.evaluate(context)
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit0O1[E, T1, R](
  implicit t1: Manifest[T1],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](),
    ImmutableList.of[Type](ofType[T1]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(context: Context[E], opt1: Option[T1] = None): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 0) {
              FunctionUnit0O1.this.evaluate(context)
            } else {
              FunctionUnit0O1.this.evaluate(context, Option(toScala1(args.get(0)).asInstanceOf[T1]))
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit0O1.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 0) {
              Future(FunctionUnit0O1.this.evaluate(context))
            } else {
              Future(
                FunctionUnit0O1.this
                  .evaluate(context, Option(toScala1(args.get(0)).asInstanceOf[T1])))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 0) {
              FunctionUnit0O1.this.evaluate(context)
            } else {
              FunctionUnit0O1.this.evaluate(context, Option(toScala1(args.get(0)).asInstanceOf[T1]))
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit0O2[E, T1, T2, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](),
    ImmutableList.of[Type](ofType[T1], ofType[T2]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(context: Context[E], opt1: Option[T1] = None, opt2: Option[T2] = None): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 0) {
              FunctionUnit0O2.this.evaluate(context)
            } else if (args.size == 1) {
              FunctionUnit0O2.this.evaluate(context, Option(toScala1(args.get(0)).asInstanceOf[T1]))
            } else {
              FunctionUnit0O2.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]))
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit0O2.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 0) {
              Future(FunctionUnit0O2.this.evaluate(context))
            } else if (args.size == 1) {
              Future(
                FunctionUnit0O2.this
                  .evaluate(context, Option(toScala1(args.get(0)).asInstanceOf[T1])))
            } else {
              Future(
                FunctionUnit0O2.this.evaluate(
                  context,
                  Option(toScala1(args.get(0)).asInstanceOf[T1]),
                  Option(toScala2(args.get(1)).asInstanceOf[T2])))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 0) {
              FunctionUnit0O2.this.evaluate(context)
            } else if (args.size == 1) {
              FunctionUnit0O2.this.evaluate(context, Option(toScala1(args.get(0)).asInstanceOf[T1]))
            } else {
              FunctionUnit0O2.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]))
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit0O3[E, T1, T2, T3, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](),
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    opt1: Option[T1] = None,
    opt2: Option[T2] = None,
    opt3: Option[T3] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 0) {
              FunctionUnit0O3.this.evaluate(context)
            } else if (args.size == 1) {
              FunctionUnit0O3.this.evaluate(context, Option(toScala1(args.get(0)).asInstanceOf[T1]))
            } else if (args.size == 2) {
              FunctionUnit0O3.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]))
            } else {
              FunctionUnit0O3.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit0O3.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 0) {
              Future(FunctionUnit0O3.this.evaluate(context))
            } else if (args.size == 1) {
              Future(
                FunctionUnit0O3.this
                  .evaluate(context, Option(toScala1(args.get(0)).asInstanceOf[T1])))
            } else if (args.size == 2) {
              Future(
                FunctionUnit0O3.this.evaluate(
                  context,
                  Option(toScala1(args.get(0)).asInstanceOf[T1]),
                  Option(toScala2(args.get(1)).asInstanceOf[T2])))
            } else {
              Future(
                FunctionUnit0O3.this.evaluate(
                  context,
                  Option(toScala1(args.get(0)).asInstanceOf[T1]),
                  Option(toScala2(args.get(1)).asInstanceOf[T2]),
                  Option(toScala3(args.get(2)).asInstanceOf[T3])))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 0) {
              FunctionUnit0O3.this.evaluate(context)
            } else if (args.size == 1) {
              FunctionUnit0O3.this.evaluate(context, Option(toScala1(args.get(0)).asInstanceOf[T1]))
            } else if (args.size == 2) {
              FunctionUnit0O3.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]))
            } else {
              FunctionUnit0O3.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit0O4[E, T1, T2, T3, T4, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](),
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    opt1: Option[T1] = None,
    opt2: Option[T2] = None,
    opt3: Option[T3] = None,
    opt4: Option[T4] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 0) {
              FunctionUnit0O4.this.evaluate(context)
            } else if (args.size == 1) {
              FunctionUnit0O4.this.evaluate(context, Option(toScala1(args.get(0)).asInstanceOf[T1]))
            } else if (args.size == 2) {
              FunctionUnit0O4.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              FunctionUnit0O4.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else {
              FunctionUnit0O4.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit0O4.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 0) {
              Future(FunctionUnit0O4.this.evaluate(context))
            } else if (args.size == 1) {
              Future(
                FunctionUnit0O4.this
                  .evaluate(context, Option(toScala1(args.get(0)).asInstanceOf[T1])))
            } else if (args.size == 2) {
              Future(
                FunctionUnit0O4.this.evaluate(
                  context,
                  Option(toScala1(args.get(0)).asInstanceOf[T1]),
                  Option(toScala2(args.get(1)).asInstanceOf[T2])))
            } else if (args.size == 3) {
              Future(
                FunctionUnit0O4.this.evaluate(
                  context,
                  Option(toScala1(args.get(0)).asInstanceOf[T1]),
                  Option(toScala2(args.get(1)).asInstanceOf[T2]),
                  Option(toScala3(args.get(2)).asInstanceOf[T3])))
            } else {
              Future(
                FunctionUnit0O4.this.evaluate(
                  context,
                  Option(toScala1(args.get(0)).asInstanceOf[T1]),
                  Option(toScala2(args.get(1)).asInstanceOf[T2]),
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 0) {
              FunctionUnit0O4.this.evaluate(context)
            } else if (args.size == 1) {
              FunctionUnit0O4.this.evaluate(context, Option(toScala1(args.get(0)).asInstanceOf[T1]))
            } else if (args.size == 2) {
              FunctionUnit0O4.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              FunctionUnit0O4.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else {
              FunctionUnit0O4.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit0O5[E, T1, T2, T3, T4, T5, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](),
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    opt1: Option[T1] = None,
    opt2: Option[T2] = None,
    opt3: Option[T3] = None,
    opt4: Option[T4] = None,
    opt5: Option[T5] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 0) {
              FunctionUnit0O5.this.evaluate(context)
            } else if (args.size == 1) {
              FunctionUnit0O5.this.evaluate(context, Option(toScala1(args.get(0)).asInstanceOf[T1]))
            } else if (args.size == 2) {
              FunctionUnit0O5.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              FunctionUnit0O5.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit0O5.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else {
              FunctionUnit0O5.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit0O5.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 0) {
              Future(FunctionUnit0O5.this.evaluate(context))
            } else if (args.size == 1) {
              Future(
                FunctionUnit0O5.this
                  .evaluate(context, Option(toScala1(args.get(0)).asInstanceOf[T1])))
            } else if (args.size == 2) {
              Future(
                FunctionUnit0O5.this.evaluate(
                  context,
                  Option(toScala1(args.get(0)).asInstanceOf[T1]),
                  Option(toScala2(args.get(1)).asInstanceOf[T2])))
            } else if (args.size == 3) {
              Future(
                FunctionUnit0O5.this.evaluate(
                  context,
                  Option(toScala1(args.get(0)).asInstanceOf[T1]),
                  Option(toScala2(args.get(1)).asInstanceOf[T2]),
                  Option(toScala3(args.get(2)).asInstanceOf[T3])))
            } else if (args.size == 4) {
              Future(
                FunctionUnit0O5.this.evaluate(
                  context,
                  Option(toScala1(args.get(0)).asInstanceOf[T1]),
                  Option(toScala2(args.get(1)).asInstanceOf[T2]),
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            } else {
              Future(
                FunctionUnit0O5.this.evaluate(
                  context,
                  Option(toScala1(args.get(0)).asInstanceOf[T1]),
                  Option(toScala2(args.get(1)).asInstanceOf[T2]),
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 0) {
              FunctionUnit0O5.this.evaluate(context)
            } else if (args.size == 1) {
              FunctionUnit0O5.this.evaluate(context, Option(toScala1(args.get(0)).asInstanceOf[T1]))
            } else if (args.size == 2) {
              FunctionUnit0O5.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              FunctionUnit0O5.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit0O5.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else {
              FunctionUnit0O5.this.evaluate(
                context,
                Option(toScala1(args.get(0)).asInstanceOf[T1]),
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit1V[E, T1, V, R](
  implicit t1: Manifest[T1],
  v: Manifest[V],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1]),
    ofType[V],
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  val toScalaV: AnyRef => Any = asScala[V]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(context: Context[E], arg1: T1, vargs: Seq[V]): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = FunctionUnit1V.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              args.asScala.drop(1).map(toScalaV(_).asInstanceOf[V]))
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit1V.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = Future(
              FunctionUnit1V.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                args.asScala.drop(1).map(toScalaV(_).asInstanceOf[V])))
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = FunctionUnit1V.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              args.asScala.drop(1).map(toScalaV(_).asInstanceOf[V]))
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit1[E, T1, R](
  implicit t1: Manifest[T1],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1]),
    ImmutableList.of[Type](),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(context: Context[E], arg1: T1): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result =
              FunctionUnit1.this.evaluate(context, toScala1(args.get(0)).asInstanceOf[T1])
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit1.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result =
              Future(FunctionUnit1.this.evaluate(context, toScala1(args.get(0)).asInstanceOf[T1]))
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result =
              FunctionUnit1.this.evaluate(context, toScala1(args.get(0)).asInstanceOf[T1])
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit1O1[E, T1, T2, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1]),
    ImmutableList.of[Type](ofType[T2]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(context: Context[E], arg1: T1, opt1: Option[T2] = None): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 1) {
              FunctionUnit1O1.this.evaluate(context, toScala1(args.get(0)).asInstanceOf[T1])
            } else {
              FunctionUnit1O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]))
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit1O1.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 1) {
              Future(FunctionUnit1O1.this.evaluate(context, toScala1(args.get(0)).asInstanceOf[T1]))
            } else {
              Future(
                FunctionUnit1O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  Option(toScala2(args.get(1)).asInstanceOf[T2])))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 1) {
              FunctionUnit1O1.this.evaluate(context, toScala1(args.get(0)).asInstanceOf[T1])
            } else {
              FunctionUnit1O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]))
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit1O2[E, T1, T2, T3, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1]),
    ImmutableList.of[Type](ofType[T2], ofType[T3]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(context: Context[E], arg1: T1, opt1: Option[T2] = None, opt2: Option[T3] = None): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 1) {
              FunctionUnit1O2.this.evaluate(context, toScala1(args.get(0)).asInstanceOf[T1])
            } else if (args.size == 2) {
              FunctionUnit1O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]))
            } else {
              FunctionUnit1O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit1O2.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 1) {
              Future(FunctionUnit1O2.this.evaluate(context, toScala1(args.get(0)).asInstanceOf[T1]))
            } else if (args.size == 2) {
              Future(
                FunctionUnit1O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  Option(toScala2(args.get(1)).asInstanceOf[T2])))
            } else {
              Future(
                FunctionUnit1O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  Option(toScala2(args.get(1)).asInstanceOf[T2]),
                  Option(toScala3(args.get(2)).asInstanceOf[T3])))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 1) {
              FunctionUnit1O2.this.evaluate(context, toScala1(args.get(0)).asInstanceOf[T1])
            } else if (args.size == 2) {
              FunctionUnit1O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]))
            } else {
              FunctionUnit1O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit1O3[E, T1, T2, T3, T4, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1]),
    ImmutableList.of[Type](ofType[T2], ofType[T3], ofType[T4]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    opt1: Option[T2] = None,
    opt2: Option[T3] = None,
    opt3: Option[T4] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 1) {
              FunctionUnit1O3.this.evaluate(context, toScala1(args.get(0)).asInstanceOf[T1])
            } else if (args.size == 2) {
              FunctionUnit1O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              FunctionUnit1O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else {
              FunctionUnit1O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit1O3.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 1) {
              Future(FunctionUnit1O3.this.evaluate(context, toScala1(args.get(0)).asInstanceOf[T1]))
            } else if (args.size == 2) {
              Future(
                FunctionUnit1O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  Option(toScala2(args.get(1)).asInstanceOf[T2])))
            } else if (args.size == 3) {
              Future(
                FunctionUnit1O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  Option(toScala2(args.get(1)).asInstanceOf[T2]),
                  Option(toScala3(args.get(2)).asInstanceOf[T3])))
            } else {
              Future(
                FunctionUnit1O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  Option(toScala2(args.get(1)).asInstanceOf[T2]),
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 1) {
              FunctionUnit1O3.this.evaluate(context, toScala1(args.get(0)).asInstanceOf[T1])
            } else if (args.size == 2) {
              FunctionUnit1O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              FunctionUnit1O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else {
              FunctionUnit1O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit1O4[E, T1, T2, T3, T4, T5, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1]),
    ImmutableList.of[Type](ofType[T2], ofType[T3], ofType[T4], ofType[T5]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    opt1: Option[T2] = None,
    opt2: Option[T3] = None,
    opt3: Option[T4] = None,
    opt4: Option[T5] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 1) {
              FunctionUnit1O4.this.evaluate(context, toScala1(args.get(0)).asInstanceOf[T1])
            } else if (args.size == 2) {
              FunctionUnit1O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              FunctionUnit1O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit1O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else {
              FunctionUnit1O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit1O4.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 1) {
              Future(FunctionUnit1O4.this.evaluate(context, toScala1(args.get(0)).asInstanceOf[T1]))
            } else if (args.size == 2) {
              Future(
                FunctionUnit1O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  Option(toScala2(args.get(1)).asInstanceOf[T2])))
            } else if (args.size == 3) {
              Future(
                FunctionUnit1O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  Option(toScala2(args.get(1)).asInstanceOf[T2]),
                  Option(toScala3(args.get(2)).asInstanceOf[T3])))
            } else if (args.size == 4) {
              Future(
                FunctionUnit1O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  Option(toScala2(args.get(1)).asInstanceOf[T2]),
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            } else {
              Future(
                FunctionUnit1O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  Option(toScala2(args.get(1)).asInstanceOf[T2]),
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 1) {
              FunctionUnit1O4.this.evaluate(context, toScala1(args.get(0)).asInstanceOf[T1])
            } else if (args.size == 2) {
              FunctionUnit1O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              FunctionUnit1O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit1O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else {
              FunctionUnit1O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit1O5[E, T1, T2, T3, T4, T5, T6, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1]),
    ImmutableList.of[Type](ofType[T2], ofType[T3], ofType[T4], ofType[T5], ofType[T6]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    opt1: Option[T2] = None,
    opt2: Option[T3] = None,
    opt3: Option[T4] = None,
    opt4: Option[T5] = None,
    opt5: Option[T6] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 1) {
              FunctionUnit1O5.this.evaluate(context, toScala1(args.get(0)).asInstanceOf[T1])
            } else if (args.size == 2) {
              FunctionUnit1O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              FunctionUnit1O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit1O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit1O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else {
              FunctionUnit1O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit1O5.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 1) {
              Future(FunctionUnit1O5.this.evaluate(context, toScala1(args.get(0)).asInstanceOf[T1]))
            } else if (args.size == 2) {
              Future(
                FunctionUnit1O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  Option(toScala2(args.get(1)).asInstanceOf[T2])))
            } else if (args.size == 3) {
              Future(
                FunctionUnit1O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  Option(toScala2(args.get(1)).asInstanceOf[T2]),
                  Option(toScala3(args.get(2)).asInstanceOf[T3])))
            } else if (args.size == 4) {
              Future(
                FunctionUnit1O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  Option(toScala2(args.get(1)).asInstanceOf[T2]),
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            } else if (args.size == 5) {
              Future(
                FunctionUnit1O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  Option(toScala2(args.get(1)).asInstanceOf[T2]),
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            } else {
              Future(
                FunctionUnit1O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  Option(toScala2(args.get(1)).asInstanceOf[T2]),
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 1) {
              FunctionUnit1O5.this.evaluate(context, toScala1(args.get(0)).asInstanceOf[T1])
            } else if (args.size == 2) {
              FunctionUnit1O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              FunctionUnit1O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit1O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit1O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else {
              FunctionUnit1O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                Option(toScala2(args.get(1)).asInstanceOf[T2]),
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit2V[E, T1, T2, V, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  v: Manifest[V],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2]),
    ofType[V],
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  val toScalaV: AnyRef => Any = asScala[V]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(context: Context[E], arg1: T1, arg2: T2, vargs: Seq[V]): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = FunctionUnit2V.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              args.asScala.drop(2).map(toScalaV(_).asInstanceOf[V]))
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit2V.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = Future(
              FunctionUnit2V.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                args.asScala.drop(2).map(toScalaV(_).asInstanceOf[V])))
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = FunctionUnit2V.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              args.asScala.drop(2).map(toScalaV(_).asInstanceOf[V]))
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit2[E, T1, T2, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2]),
    ImmutableList.of[Type](),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(context: Context[E], arg1: T1, arg2: T2): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = FunctionUnit2.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2])
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit2.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = Future(
              FunctionUnit2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2]))
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = FunctionUnit2.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2])
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit2O1[E, T1, T2, T3, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2]),
    ImmutableList.of[Type](ofType[T3]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(context: Context[E], arg1: T1, arg2: T2, opt1: Option[T3] = None): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else {
              FunctionUnit2O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit2O1.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              Future(
                FunctionUnit2O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2]))
            } else {
              Future(
                FunctionUnit2O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3])))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else {
              FunctionUnit2O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit2O2[E, T1, T2, T3, T4, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2]),
    ImmutableList.of[Type](ofType[T3], ofType[T4]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    opt1: Option[T3] = None,
    opt2: Option[T4] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else if (args.size == 3) {
              FunctionUnit2O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else {
              FunctionUnit2O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit2O2.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              Future(
                FunctionUnit2O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              Future(
                FunctionUnit2O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3])))
            } else {
              Future(
                FunctionUnit2O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else if (args.size == 3) {
              FunctionUnit2O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else {
              FunctionUnit2O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit2O3[E, T1, T2, T3, T4, T5, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2]),
    ImmutableList.of[Type](ofType[T3], ofType[T4], ofType[T5]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    opt1: Option[T3] = None,
    opt2: Option[T4] = None,
    opt3: Option[T5] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else if (args.size == 3) {
              FunctionUnit2O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit2O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else {
              FunctionUnit2O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit2O3.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              Future(
                FunctionUnit2O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              Future(
                FunctionUnit2O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3])))
            } else if (args.size == 4) {
              Future(
                FunctionUnit2O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            } else {
              Future(
                FunctionUnit2O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else if (args.size == 3) {
              FunctionUnit2O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit2O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else {
              FunctionUnit2O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit2O4[E, T1, T2, T3, T4, T5, T6, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2]),
    ImmutableList.of[Type](ofType[T3], ofType[T4], ofType[T5], ofType[T6]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    opt1: Option[T3] = None,
    opt2: Option[T4] = None,
    opt3: Option[T5] = None,
    opt4: Option[T6] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else if (args.size == 3) {
              FunctionUnit2O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit2O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit2O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else {
              FunctionUnit2O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit2O4.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              Future(
                FunctionUnit2O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              Future(
                FunctionUnit2O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3])))
            } else if (args.size == 4) {
              Future(
                FunctionUnit2O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            } else if (args.size == 5) {
              Future(
                FunctionUnit2O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            } else {
              Future(
                FunctionUnit2O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else if (args.size == 3) {
              FunctionUnit2O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit2O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit2O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else {
              FunctionUnit2O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit2O5[E, T1, T2, T3, T4, T5, T6, T7, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2]),
    ImmutableList.of[Type](ofType[T3], ofType[T4], ofType[T5], ofType[T6], ofType[T7]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    opt1: Option[T3] = None,
    opt2: Option[T4] = None,
    opt3: Option[T5] = None,
    opt4: Option[T6] = None,
    opt5: Option[T7] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else if (args.size == 3) {
              FunctionUnit2O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit2O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit2O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit2O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else {
              FunctionUnit2O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit2O5.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              Future(
                FunctionUnit2O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              Future(
                FunctionUnit2O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3])))
            } else if (args.size == 4) {
              Future(
                FunctionUnit2O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            } else if (args.size == 5) {
              Future(
                FunctionUnit2O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            } else if (args.size == 6) {
              Future(
                FunctionUnit2O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            } else {
              Future(
                FunctionUnit2O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else if (args.size == 3) {
              FunctionUnit2O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit2O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit2O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit2O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else {
              FunctionUnit2O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit3V[E, T1, T2, T3, V, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  v: Manifest[V],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3]),
    ofType[V],
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  val toScalaV: AnyRef => Any = asScala[V]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(context: Context[E], arg1: T1, arg2: T2, arg3: T3, vargs: Seq[V]): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = FunctionUnit3V.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              args.asScala.drop(3).map(toScalaV(_).asInstanceOf[V])
            )
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit3V.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = Future(
              FunctionUnit3V.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                args.asScala.drop(3).map(toScalaV(_).asInstanceOf[V])
              ))
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = FunctionUnit3V.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              args.asScala.drop(3).map(toScalaV(_).asInstanceOf[V])
            )
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit3[E, T1, T2, T3, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3]),
    ImmutableList.of[Type](),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(context: Context[E], arg1: T1, arg2: T2, arg3: T3): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = FunctionUnit3.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3])
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit3.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = Future(
              FunctionUnit3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3]))
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = FunctionUnit3.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3])
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit3O1[E, T1, T2, T3, T4, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3]),
    ImmutableList.of[Type](ofType[T4]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(context: Context[E], arg1: T1, arg2: T2, arg3: T3, opt1: Option[T4] = None): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 3) {
              FunctionUnit3O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3])
            } else {
              FunctionUnit3O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit3O1.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 3) {
              Future(
                FunctionUnit3O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3]))
            } else {
              Future(
                FunctionUnit3O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 3) {
              FunctionUnit3O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3])
            } else {
              FunctionUnit3O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit3O2[E, T1, T2, T3, T4, T5, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3]),
    ImmutableList.of[Type](ofType[T4], ofType[T5]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    opt1: Option[T4] = None,
    opt2: Option[T5] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 3) {
              FunctionUnit3O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3])
            } else if (args.size == 4) {
              FunctionUnit3O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else {
              FunctionUnit3O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit3O2.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 3) {
              Future(
                FunctionUnit3O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              Future(
                FunctionUnit3O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            } else {
              Future(
                FunctionUnit3O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 3) {
              FunctionUnit3O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3])
            } else if (args.size == 4) {
              FunctionUnit3O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else {
              FunctionUnit3O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit3O3[E, T1, T2, T3, T4, T5, T6, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3]),
    ImmutableList.of[Type](ofType[T4], ofType[T5], ofType[T6]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    opt1: Option[T4] = None,
    opt2: Option[T5] = None,
    opt3: Option[T6] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 3) {
              FunctionUnit3O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3])
            } else if (args.size == 4) {
              FunctionUnit3O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit3O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else {
              FunctionUnit3O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit3O3.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 3) {
              Future(
                FunctionUnit3O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              Future(
                FunctionUnit3O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            } else if (args.size == 5) {
              Future(
                FunctionUnit3O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            } else {
              Future(
                FunctionUnit3O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 3) {
              FunctionUnit3O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3])
            } else if (args.size == 4) {
              FunctionUnit3O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit3O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else {
              FunctionUnit3O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit3O4[E, T1, T2, T3, T4, T5, T6, T7, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3]),
    ImmutableList.of[Type](ofType[T4], ofType[T5], ofType[T6], ofType[T7]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    opt1: Option[T4] = None,
    opt2: Option[T5] = None,
    opt3: Option[T6] = None,
    opt4: Option[T7] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 3) {
              FunctionUnit3O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3])
            } else if (args.size == 4) {
              FunctionUnit3O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit3O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit3O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else {
              FunctionUnit3O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit3O4.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 3) {
              Future(
                FunctionUnit3O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              Future(
                FunctionUnit3O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            } else if (args.size == 5) {
              Future(
                FunctionUnit3O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            } else if (args.size == 6) {
              Future(
                FunctionUnit3O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            } else {
              Future(
                FunctionUnit3O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 3) {
              FunctionUnit3O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3])
            } else if (args.size == 4) {
              FunctionUnit3O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit3O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit3O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else {
              FunctionUnit3O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit3O5[E, T1, T2, T3, T4, T5, T6, T7, T8, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3]),
    ImmutableList.of[Type](ofType[T4], ofType[T5], ofType[T6], ofType[T7], ofType[T8]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    opt1: Option[T4] = None,
    opt2: Option[T5] = None,
    opt3: Option[T6] = None,
    opt4: Option[T7] = None,
    opt5: Option[T8] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 3) {
              FunctionUnit3O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3])
            } else if (args.size == 4) {
              FunctionUnit3O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit3O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit3O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit3O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else {
              FunctionUnit3O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit3O5.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 3) {
              Future(
                FunctionUnit3O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              Future(
                FunctionUnit3O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            } else if (args.size == 5) {
              Future(
                FunctionUnit3O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            } else if (args.size == 6) {
              Future(
                FunctionUnit3O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            } else if (args.size == 7) {
              Future(
                FunctionUnit3O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            } else {
              Future(
                FunctionUnit3O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 3) {
              FunctionUnit3O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3])
            } else if (args.size == 4) {
              FunctionUnit3O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit3O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit3O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit3O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else {
              FunctionUnit3O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit4V[E, T1, T2, T3, T4, V, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  v: Manifest[V],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4]),
    ofType[V],
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  val toScalaV: AnyRef => Any = asScala[V]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(context: Context[E], arg1: T1, arg2: T2, arg3: T3, arg4: T4, vargs: Seq[V]): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = FunctionUnit4V.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              toScala4(args.get(3)).asInstanceOf[T4],
              args.asScala.drop(4).map(toScalaV(_).asInstanceOf[V])
            )
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit4V.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = Future(
              FunctionUnit4V.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                args.asScala.drop(4).map(toScalaV(_).asInstanceOf[V])
              ))
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = FunctionUnit4V.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              toScala4(args.get(3)).asInstanceOf[T4],
              args.asScala.drop(4).map(toScalaV(_).asInstanceOf[V])
            )
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit4[E, T1, T2, T3, T4, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4]),
    ImmutableList.of[Type](),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(context: Context[E], arg1: T1, arg2: T2, arg3: T3, arg4: T4): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = FunctionUnit4.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              toScala4(args.get(3)).asInstanceOf[T4]
            )
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit4.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = Future(
              FunctionUnit4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4]
              ))
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = FunctionUnit4.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              toScala4(args.get(3)).asInstanceOf[T4]
            )
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit4O1[E, T1, T2, T3, T4, T5, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4]),
    ImmutableList.of[Type](ofType[T5]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    opt1: Option[T5] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 4) {
              FunctionUnit4O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4]
              )
            } else {
              FunctionUnit4O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit4O1.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 4) {
              Future(
                FunctionUnit4O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4]
                ))
            } else {
              Future(
                FunctionUnit4O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 4) {
              FunctionUnit4O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4]
              )
            } else {
              FunctionUnit4O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit4O2[E, T1, T2, T3, T4, T5, T6, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4]),
    ImmutableList.of[Type](ofType[T5], ofType[T6]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    opt1: Option[T5] = None,
    opt2: Option[T6] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 4) {
              FunctionUnit4O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4]
              )
            } else if (args.size == 5) {
              FunctionUnit4O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else {
              FunctionUnit4O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit4O2.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 4) {
              Future(
                FunctionUnit4O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4]
                ))
            } else if (args.size == 5) {
              Future(
                FunctionUnit4O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            } else {
              Future(
                FunctionUnit4O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 4) {
              FunctionUnit4O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4]
              )
            } else if (args.size == 5) {
              FunctionUnit4O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else {
              FunctionUnit4O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit4O3[E, T1, T2, T3, T4, T5, T6, T7, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4]),
    ImmutableList.of[Type](ofType[T5], ofType[T6], ofType[T7]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    opt1: Option[T5] = None,
    opt2: Option[T6] = None,
    opt3: Option[T7] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 4) {
              FunctionUnit4O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4]
              )
            } else if (args.size == 5) {
              FunctionUnit4O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit4O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else {
              FunctionUnit4O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit4O3.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 4) {
              Future(
                FunctionUnit4O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4]
                ))
            } else if (args.size == 5) {
              Future(
                FunctionUnit4O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            } else if (args.size == 6) {
              Future(
                FunctionUnit4O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            } else {
              Future(
                FunctionUnit4O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 4) {
              FunctionUnit4O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4]
              )
            } else if (args.size == 5) {
              FunctionUnit4O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit4O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else {
              FunctionUnit4O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit4O4[E, T1, T2, T3, T4, T5, T6, T7, T8, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4]),
    ImmutableList.of[Type](ofType[T5], ofType[T6], ofType[T7], ofType[T8]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    opt1: Option[T5] = None,
    opt2: Option[T6] = None,
    opt3: Option[T7] = None,
    opt4: Option[T8] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 4) {
              FunctionUnit4O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4]
              )
            } else if (args.size == 5) {
              FunctionUnit4O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit4O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit4O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else {
              FunctionUnit4O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit4O4.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 4) {
              Future(
                FunctionUnit4O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4]
                ))
            } else if (args.size == 5) {
              Future(
                FunctionUnit4O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            } else if (args.size == 6) {
              Future(
                FunctionUnit4O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            } else if (args.size == 7) {
              Future(
                FunctionUnit4O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            } else {
              Future(
                FunctionUnit4O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 4) {
              FunctionUnit4O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4]
              )
            } else if (args.size == 5) {
              FunctionUnit4O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit4O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit4O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else {
              FunctionUnit4O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit4O5[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4]),
    ImmutableList.of[Type](ofType[T5], ofType[T6], ofType[T7], ofType[T8], ofType[T9]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    opt1: Option[T5] = None,
    opt2: Option[T6] = None,
    opt3: Option[T7] = None,
    opt4: Option[T8] = None,
    opt5: Option[T9] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 4) {
              FunctionUnit4O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4]
              )
            } else if (args.size == 5) {
              FunctionUnit4O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit4O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit4O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit4O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else {
              FunctionUnit4O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit4O5.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 4) {
              Future(
                FunctionUnit4O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4]
                ))
            } else if (args.size == 5) {
              Future(
                FunctionUnit4O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            } else if (args.size == 6) {
              Future(
                FunctionUnit4O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            } else if (args.size == 7) {
              Future(
                FunctionUnit4O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            } else if (args.size == 8) {
              Future(
                FunctionUnit4O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            } else {
              Future(
                FunctionUnit4O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 4) {
              FunctionUnit4O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4]
              )
            } else if (args.size == 5) {
              FunctionUnit4O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit4O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit4O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit4O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else {
              FunctionUnit4O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit5V[E, T1, T2, T3, T4, T5, V, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  v: Manifest[V],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5]),
    ofType[V],
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  val toScalaV: AnyRef => Any = asScala[V]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    vargs: Seq[V]
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = FunctionUnit5V.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              toScala4(args.get(3)).asInstanceOf[T4],
              toScala5(args.get(4)).asInstanceOf[T5],
              args.asScala.drop(5).map(toScalaV(_).asInstanceOf[V])
            )
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit5V.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = Future(
              FunctionUnit5V.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                args.asScala.drop(5).map(toScalaV(_).asInstanceOf[V])
              ))
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = FunctionUnit5V.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              toScala4(args.get(3)).asInstanceOf[T4],
              toScala5(args.get(4)).asInstanceOf[T5],
              args.asScala.drop(5).map(toScalaV(_).asInstanceOf[V])
            )
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit5[E, T1, T2, T3, T4, T5, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5]),
    ImmutableList.of[Type](),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(context: Context[E], arg1: T1, arg2: T2, arg3: T3, arg4: T4, arg5: T5): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = FunctionUnit5.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              toScala4(args.get(3)).asInstanceOf[T4],
              toScala5(args.get(4)).asInstanceOf[T5]
            )
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit5.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = Future(
              FunctionUnit5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5]
              ))
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = FunctionUnit5.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              toScala4(args.get(3)).asInstanceOf[T4],
              toScala5(args.get(4)).asInstanceOf[T5]
            )
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit5O1[E, T1, T2, T3, T4, T5, T6, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5]),
    ImmutableList.of[Type](ofType[T6]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    opt1: Option[T6] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 5) {
              FunctionUnit5O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5]
              )
            } else {
              FunctionUnit5O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit5O1.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 5) {
              Future(
                FunctionUnit5O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5]
                ))
            } else {
              Future(
                FunctionUnit5O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 5) {
              FunctionUnit5O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5]
              )
            } else {
              FunctionUnit5O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit5O2[E, T1, T2, T3, T4, T5, T6, T7, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5]),
    ImmutableList.of[Type](ofType[T6], ofType[T7]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    opt1: Option[T6] = None,
    opt2: Option[T7] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 5) {
              FunctionUnit5O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5]
              )
            } else if (args.size == 6) {
              FunctionUnit5O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else {
              FunctionUnit5O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit5O2.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 5) {
              Future(
                FunctionUnit5O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5]
                ))
            } else if (args.size == 6) {
              Future(
                FunctionUnit5O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            } else {
              Future(
                FunctionUnit5O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 5) {
              FunctionUnit5O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5]
              )
            } else if (args.size == 6) {
              FunctionUnit5O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else {
              FunctionUnit5O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit5O3[E, T1, T2, T3, T4, T5, T6, T7, T8, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5]),
    ImmutableList.of[Type](ofType[T6], ofType[T7], ofType[T8]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    opt1: Option[T6] = None,
    opt2: Option[T7] = None,
    opt3: Option[T8] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 5) {
              FunctionUnit5O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5]
              )
            } else if (args.size == 6) {
              FunctionUnit5O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit5O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else {
              FunctionUnit5O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit5O3.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 5) {
              Future(
                FunctionUnit5O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5]
                ))
            } else if (args.size == 6) {
              Future(
                FunctionUnit5O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            } else if (args.size == 7) {
              Future(
                FunctionUnit5O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            } else {
              Future(
                FunctionUnit5O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 5) {
              FunctionUnit5O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5]
              )
            } else if (args.size == 6) {
              FunctionUnit5O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit5O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else {
              FunctionUnit5O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit5O4[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5]),
    ImmutableList.of[Type](ofType[T6], ofType[T7], ofType[T8], ofType[T9]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    opt1: Option[T6] = None,
    opt2: Option[T7] = None,
    opt3: Option[T8] = None,
    opt4: Option[T9] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 5) {
              FunctionUnit5O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5]
              )
            } else if (args.size == 6) {
              FunctionUnit5O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit5O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit5O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else {
              FunctionUnit5O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit5O4.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 5) {
              Future(
                FunctionUnit5O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5]
                ))
            } else if (args.size == 6) {
              Future(
                FunctionUnit5O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            } else if (args.size == 7) {
              Future(
                FunctionUnit5O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            } else if (args.size == 8) {
              Future(
                FunctionUnit5O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            } else {
              Future(
                FunctionUnit5O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 5) {
              FunctionUnit5O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5]
              )
            } else if (args.size == 6) {
              FunctionUnit5O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit5O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit5O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else {
              FunctionUnit5O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit5O5[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  t10: Manifest[T10],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5]),
    ImmutableList.of[Type](ofType[T6], ofType[T7], ofType[T8], ofType[T9], ofType[T10]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toScala10: AnyRef => Any = asScala[T10]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    opt1: Option[T6] = None,
    opt2: Option[T7] = None,
    opt3: Option[T8] = None,
    opt4: Option[T9] = None,
    opt5: Option[T10] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 5) {
              FunctionUnit5O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5]
              )
            } else if (args.size == 6) {
              FunctionUnit5O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit5O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit5O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit5O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else {
              FunctionUnit5O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit5O5.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 5) {
              Future(
                FunctionUnit5O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5]
                ))
            } else if (args.size == 6) {
              Future(
                FunctionUnit5O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            } else if (args.size == 7) {
              Future(
                FunctionUnit5O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            } else if (args.size == 8) {
              Future(
                FunctionUnit5O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            } else if (args.size == 9) {
              Future(
                FunctionUnit5O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9])
                ))
            } else {
              Future(
                FunctionUnit5O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 5) {
              FunctionUnit5O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5]
              )
            } else if (args.size == 6) {
              FunctionUnit5O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit5O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit5O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit5O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else {
              FunctionUnit5O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit6V[E, T1, T2, T3, T4, T5, T6, V, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  v: Manifest[V],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5], ofType[T6]),
    ofType[V],
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  val toScalaV: AnyRef => Any = asScala[V]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    arg6: T6,
    vargs: Seq[V]
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = FunctionUnit6V.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              toScala4(args.get(3)).asInstanceOf[T4],
              toScala5(args.get(4)).asInstanceOf[T5],
              toScala6(args.get(5)).asInstanceOf[T6],
              args.asScala.drop(6).map(toScalaV(_).asInstanceOf[V])
            )
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit6V.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = Future(
              FunctionUnit6V.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                args.asScala.drop(6).map(toScalaV(_).asInstanceOf[V])
              ))
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = FunctionUnit6V.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              toScala4(args.get(3)).asInstanceOf[T4],
              toScala5(args.get(4)).asInstanceOf[T5],
              toScala6(args.get(5)).asInstanceOf[T6],
              args.asScala.drop(6).map(toScalaV(_).asInstanceOf[V])
            )
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit6[E, T1, T2, T3, T4, T5, T6, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5], ofType[T6]),
    ImmutableList.of[Type](),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(context: Context[E], arg1: T1, arg2: T2, arg3: T3, arg4: T4, arg5: T5, arg6: T6): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = FunctionUnit6.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              toScala4(args.get(3)).asInstanceOf[T4],
              toScala5(args.get(4)).asInstanceOf[T5],
              toScala6(args.get(5)).asInstanceOf[T6]
            )
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit6.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = Future(
              FunctionUnit6.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6]
              ))
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = FunctionUnit6.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              toScala4(args.get(3)).asInstanceOf[T4],
              toScala5(args.get(4)).asInstanceOf[T5],
              toScala6(args.get(5)).asInstanceOf[T6]
            )
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit6O1[E, T1, T2, T3, T4, T5, T6, T7, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5], ofType[T6]),
    ImmutableList.of[Type](ofType[T7]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    arg6: T6,
    opt1: Option[T7] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 6) {
              FunctionUnit6O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6]
              )
            } else {
              FunctionUnit6O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit6O1.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 6) {
              Future(
                FunctionUnit6O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6]
                ))
            } else {
              Future(
                FunctionUnit6O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 6) {
              FunctionUnit6O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6]
              )
            } else {
              FunctionUnit6O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit6O2[E, T1, T2, T3, T4, T5, T6, T7, T8, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5], ofType[T6]),
    ImmutableList.of[Type](ofType[T7], ofType[T8]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    arg6: T6,
    opt1: Option[T7] = None,
    opt2: Option[T8] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 6) {
              FunctionUnit6O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6]
              )
            } else if (args.size == 7) {
              FunctionUnit6O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else {
              FunctionUnit6O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit6O2.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 6) {
              Future(
                FunctionUnit6O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6]
                ))
            } else if (args.size == 7) {
              Future(
                FunctionUnit6O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            } else {
              Future(
                FunctionUnit6O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 6) {
              FunctionUnit6O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6]
              )
            } else if (args.size == 7) {
              FunctionUnit6O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else {
              FunctionUnit6O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit6O3[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5], ofType[T6]),
    ImmutableList.of[Type](ofType[T7], ofType[T8], ofType[T9]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    arg6: T6,
    opt1: Option[T7] = None,
    opt2: Option[T8] = None,
    opt3: Option[T9] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 6) {
              FunctionUnit6O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6]
              )
            } else if (args.size == 7) {
              FunctionUnit6O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit6O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else {
              FunctionUnit6O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit6O3.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 6) {
              Future(
                FunctionUnit6O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6]
                ))
            } else if (args.size == 7) {
              Future(
                FunctionUnit6O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            } else if (args.size == 8) {
              Future(
                FunctionUnit6O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            } else {
              Future(
                FunctionUnit6O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 6) {
              FunctionUnit6O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6]
              )
            } else if (args.size == 7) {
              FunctionUnit6O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit6O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else {
              FunctionUnit6O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit6O4[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  t10: Manifest[T10],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5], ofType[T6]),
    ImmutableList.of[Type](ofType[T7], ofType[T8], ofType[T9], ofType[T10]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toScala10: AnyRef => Any = asScala[T10]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    arg6: T6,
    opt1: Option[T7] = None,
    opt2: Option[T8] = None,
    opt3: Option[T9] = None,
    opt4: Option[T10] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 6) {
              FunctionUnit6O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6]
              )
            } else if (args.size == 7) {
              FunctionUnit6O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit6O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit6O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else {
              FunctionUnit6O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit6O4.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 6) {
              Future(
                FunctionUnit6O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6]
                ))
            } else if (args.size == 7) {
              Future(
                FunctionUnit6O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            } else if (args.size == 8) {
              Future(
                FunctionUnit6O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            } else if (args.size == 9) {
              Future(
                FunctionUnit6O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9])
                ))
            } else {
              Future(
                FunctionUnit6O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 6) {
              FunctionUnit6O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6]
              )
            } else if (args.size == 7) {
              FunctionUnit6O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit6O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit6O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else {
              FunctionUnit6O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit6O5[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  t10: Manifest[T10],
  t11: Manifest[T11],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5], ofType[T6]),
    ImmutableList.of[Type](ofType[T7], ofType[T8], ofType[T9], ofType[T10], ofType[T11]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toScala10: AnyRef => Any = asScala[T10]
  private[this] val toScala11: AnyRef => Any = asScala[T11]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    arg6: T6,
    opt1: Option[T7] = None,
    opt2: Option[T8] = None,
    opt3: Option[T9] = None,
    opt4: Option[T10] = None,
    opt5: Option[T11] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 6) {
              FunctionUnit6O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6]
              )
            } else if (args.size == 7) {
              FunctionUnit6O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit6O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit6O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else if (args.size == 10) {
              FunctionUnit6O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            } else {
              FunctionUnit6O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit6O5.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 6) {
              Future(
                FunctionUnit6O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6]
                ))
            } else if (args.size == 7) {
              Future(
                FunctionUnit6O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            } else if (args.size == 8) {
              Future(
                FunctionUnit6O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            } else if (args.size == 9) {
              Future(
                FunctionUnit6O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9])
                ))
            } else if (args.size == 10) {
              Future(
                FunctionUnit6O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10])
                ))
            } else {
              Future(
                FunctionUnit6O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10]),
                  Option(toScala11(args.get(10)).asInstanceOf[T11])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 6) {
              FunctionUnit6O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6]
              )
            } else if (args.size == 7) {
              FunctionUnit6O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit6O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit6O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else if (args.size == 10) {
              FunctionUnit6O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            } else {
              FunctionUnit6O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit7V[E, T1, T2, T3, T4, T5, T6, T7, V, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  v: Manifest[V],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList
      .of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5], ofType[T6], ofType[T7]),
    ofType[V],
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  val toScalaV: AnyRef => Any = asScala[V]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    arg6: T6,
    arg7: T7,
    vargs: Seq[V]
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = FunctionUnit7V.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              toScala4(args.get(3)).asInstanceOf[T4],
              toScala5(args.get(4)).asInstanceOf[T5],
              toScala6(args.get(5)).asInstanceOf[T6],
              toScala7(args.get(6)).asInstanceOf[T7],
              args.asScala.drop(7).map(toScalaV(_).asInstanceOf[V])
            )
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit7V.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = Future(
              FunctionUnit7V.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                args.asScala.drop(7).map(toScalaV(_).asInstanceOf[V])
              ))
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = FunctionUnit7V.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              toScala4(args.get(3)).asInstanceOf[T4],
              toScala5(args.get(4)).asInstanceOf[T5],
              toScala6(args.get(5)).asInstanceOf[T6],
              toScala7(args.get(6)).asInstanceOf[T7],
              args.asScala.drop(7).map(toScalaV(_).asInstanceOf[V])
            )
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit7[E, T1, T2, T3, T4, T5, T6, T7, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList
      .of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5], ofType[T6], ofType[T7]),
    ImmutableList.of[Type](),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    arg6: T6,
    arg7: T7
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = FunctionUnit7.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              toScala4(args.get(3)).asInstanceOf[T4],
              toScala5(args.get(4)).asInstanceOf[T5],
              toScala6(args.get(5)).asInstanceOf[T6],
              toScala7(args.get(6)).asInstanceOf[T7]
            )
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit7.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = Future(
              FunctionUnit7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7]
              ))
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = FunctionUnit7.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              toScala4(args.get(3)).asInstanceOf[T4],
              toScala5(args.get(4)).asInstanceOf[T5],
              toScala6(args.get(5)).asInstanceOf[T6],
              toScala7(args.get(6)).asInstanceOf[T7]
            )
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit7O1[E, T1, T2, T3, T4, T5, T6, T7, T8, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList
      .of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5], ofType[T6], ofType[T7]),
    ImmutableList.of[Type](ofType[T8]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    arg6: T6,
    arg7: T7,
    opt1: Option[T8] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 7) {
              FunctionUnit7O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7]
              )
            } else {
              FunctionUnit7O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit7O1.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 7) {
              Future(
                FunctionUnit7O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7]
                ))
            } else {
              Future(
                FunctionUnit7O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 7) {
              FunctionUnit7O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7]
              )
            } else {
              FunctionUnit7O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit7O2[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList
      .of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5], ofType[T6], ofType[T7]),
    ImmutableList.of[Type](ofType[T8], ofType[T9]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    arg6: T6,
    arg7: T7,
    opt1: Option[T8] = None,
    opt2: Option[T9] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 7) {
              FunctionUnit7O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7]
              )
            } else if (args.size == 8) {
              FunctionUnit7O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else {
              FunctionUnit7O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit7O2.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 7) {
              Future(
                FunctionUnit7O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7]
                ))
            } else if (args.size == 8) {
              Future(
                FunctionUnit7O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            } else {
              Future(
                FunctionUnit7O2.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 7) {
              FunctionUnit7O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7]
              )
            } else if (args.size == 8) {
              FunctionUnit7O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else {
              FunctionUnit7O2.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit7O3[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  t10: Manifest[T10],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList
      .of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5], ofType[T6], ofType[T7]),
    ImmutableList.of[Type](ofType[T8], ofType[T9], ofType[T10]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toScala10: AnyRef => Any = asScala[T10]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    arg6: T6,
    arg7: T7,
    opt1: Option[T8] = None,
    opt2: Option[T9] = None,
    opt3: Option[T10] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 7) {
              FunctionUnit7O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7]
              )
            } else if (args.size == 8) {
              FunctionUnit7O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit7O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else {
              FunctionUnit7O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit7O3.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 7) {
              Future(
                FunctionUnit7O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7]
                ))
            } else if (args.size == 8) {
              Future(
                FunctionUnit7O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            } else if (args.size == 9) {
              Future(
                FunctionUnit7O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9])
                ))
            } else {
              Future(
                FunctionUnit7O3.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 7) {
              FunctionUnit7O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7]
              )
            } else if (args.size == 8) {
              FunctionUnit7O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit7O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else {
              FunctionUnit7O3.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit7O4[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  t10: Manifest[T10],
  t11: Manifest[T11],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList
      .of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5], ofType[T6], ofType[T7]),
    ImmutableList.of[Type](ofType[T8], ofType[T9], ofType[T10], ofType[T11]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toScala10: AnyRef => Any = asScala[T10]
  private[this] val toScala11: AnyRef => Any = asScala[T11]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    arg6: T6,
    arg7: T7,
    opt1: Option[T8] = None,
    opt2: Option[T9] = None,
    opt3: Option[T10] = None,
    opt4: Option[T11] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 7) {
              FunctionUnit7O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7]
              )
            } else if (args.size == 8) {
              FunctionUnit7O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit7O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else if (args.size == 10) {
              FunctionUnit7O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            } else {
              FunctionUnit7O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit7O4.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 7) {
              Future(
                FunctionUnit7O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7]
                ))
            } else if (args.size == 8) {
              Future(
                FunctionUnit7O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            } else if (args.size == 9) {
              Future(
                FunctionUnit7O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9])
                ))
            } else if (args.size == 10) {
              Future(
                FunctionUnit7O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10])
                ))
            } else {
              Future(
                FunctionUnit7O4.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10]),
                  Option(toScala11(args.get(10)).asInstanceOf[T11])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 7) {
              FunctionUnit7O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7]
              )
            } else if (args.size == 8) {
              FunctionUnit7O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit7O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else if (args.size == 10) {
              FunctionUnit7O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            } else {
              FunctionUnit7O4.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit7O5[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  t10: Manifest[T10],
  t11: Manifest[T11],
  t12: Manifest[T12],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList
      .of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4], ofType[T5], ofType[T6], ofType[T7]),
    ImmutableList.of[Type](ofType[T8], ofType[T9], ofType[T10], ofType[T11], ofType[T12]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toScala10: AnyRef => Any = asScala[T10]
  private[this] val toScala11: AnyRef => Any = asScala[T11]
  private[this] val toScala12: AnyRef => Any = asScala[T12]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    arg6: T6,
    arg7: T7,
    opt1: Option[T8] = None,
    opt2: Option[T9] = None,
    opt3: Option[T10] = None,
    opt4: Option[T11] = None,
    opt5: Option[T12] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 7) {
              FunctionUnit7O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7]
              )
            } else if (args.size == 8) {
              FunctionUnit7O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit7O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else if (args.size == 10) {
              FunctionUnit7O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            } else if (args.size == 11) {
              FunctionUnit7O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11])
              )
            } else {
              FunctionUnit7O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11]),
                Option(toScala12(args.get(11)).asInstanceOf[T12])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit7O5.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 7) {
              Future(
                FunctionUnit7O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7]
                ))
            } else if (args.size == 8) {
              Future(
                FunctionUnit7O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            } else if (args.size == 9) {
              Future(
                FunctionUnit7O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9])
                ))
            } else if (args.size == 10) {
              Future(
                FunctionUnit7O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10])
                ))
            } else if (args.size == 11) {
              Future(
                FunctionUnit7O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10]),
                  Option(toScala11(args.get(10)).asInstanceOf[T11])
                ))
            } else {
              Future(
                FunctionUnit7O5.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10]),
                  Option(toScala11(args.get(10)).asInstanceOf[T11]),
                  Option(toScala12(args.get(11)).asInstanceOf[T12])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 7) {
              FunctionUnit7O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7]
              )
            } else if (args.size == 8) {
              FunctionUnit7O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit7O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else if (args.size == 10) {
              FunctionUnit7O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            } else if (args.size == 11) {
              FunctionUnit7O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11])
              )
            } else {
              FunctionUnit7O5.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11]),
                Option(toScala12(args.get(11)).asInstanceOf[T12])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit2O9[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  t10: Manifest[T10],
  t11: Manifest[T11],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2]),
    ImmutableList.of[Type](
      ofType[T3],
      ofType[T4],
      ofType[T5],
      ofType[T6],
      ofType[T7],
      ofType[T8],
      ofType[T9],
      ofType[T10],
      ofType[T11]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toScala10: AnyRef => Any = asScala[T10]
  private[this] val toScala11: AnyRef => Any = asScala[T11]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    opt1: Option[T3] = None,
    opt2: Option[T4] = None,
    opt3: Option[T5] = None,
    opt4: Option[T6] = None,
    opt5: Option[T7] = None,
    opt6: Option[T8] = None,
    opt7: Option[T9] = None,
    opt8: Option[T10] = None,
    opt9: Option[T11] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else if (args.size == 3) {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else if (args.size == 10) {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            } else {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit2O9.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              Future(
                FunctionUnit2O9.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              Future(
                FunctionUnit2O9.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3])))
            } else if (args.size == 4) {
              Future(
                FunctionUnit2O9.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            } else if (args.size == 5) {
              Future(
                FunctionUnit2O9.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            } else if (args.size == 6) {
              Future(
                FunctionUnit2O9.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            } else if (args.size == 7) {
              Future(
                FunctionUnit2O9.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            } else if (args.size == 8) {
              Future(
                FunctionUnit2O9.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            } else if (args.size == 9) {
              Future(
                FunctionUnit2O9.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9])
                ))
            } else if (args.size == 10) {
              Future(
                FunctionUnit2O9.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10])
                ))
            } else {
              Future(
                FunctionUnit2O9.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10]),
                  Option(toScala11(args.get(10)).asInstanceOf[T11])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else if (args.size == 3) {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else if (args.size == 10) {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            } else {
              FunctionUnit2O9.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit4O7[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  t10: Manifest[T10],
  t11: Manifest[T11],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3], ofType[T4]),
    ImmutableList.of[Type](
      ofType[T5],
      ofType[T6],
      ofType[T7],
      ofType[T8],
      ofType[T9],
      ofType[T10],
      ofType[T11]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toScala10: AnyRef => Any = asScala[T10]
  private[this] val toScala11: AnyRef => Any = asScala[T11]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    opt1: Option[T5] = None,
    opt2: Option[T6] = None,
    opt3: Option[T7] = None,
    opt4: Option[T8] = None,
    opt5: Option[T9] = None,
    opt6: Option[T10] = None,
    opt7: Option[T11] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 4) {
              FunctionUnit4O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4]
              )
            } else if (args.size == 5) {
              FunctionUnit4O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit4O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit4O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit4O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit4O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else if (args.size == 10) {
              FunctionUnit4O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            } else {
              FunctionUnit4O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit4O7.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 4) {
              Future(
                FunctionUnit4O7.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4]
                ))
            } else if (args.size == 5) {
              Future(
                FunctionUnit4O7.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            } else if (args.size == 6) {
              Future(
                FunctionUnit4O7.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            } else if (args.size == 7) {
              Future(
                FunctionUnit4O7.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            } else if (args.size == 8) {
              Future(
                FunctionUnit4O7.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            } else if (args.size == 9) {
              Future(
                FunctionUnit4O7.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9])
                ))
            } else if (args.size == 10) {
              Future(
                FunctionUnit4O7.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10])
                ))
            } else {
              Future(
                FunctionUnit4O7.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10]),
                  Option(toScala11(args.get(10)).asInstanceOf[T11])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 4) {
              FunctionUnit4O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4]
              )
            } else if (args.size == 5) {
              FunctionUnit4O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit4O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit4O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit4O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit4O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else if (args.size == 10) {
              FunctionUnit4O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            } else {
              FunctionUnit4O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit11O1[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  t10: Manifest[T10],
  t11: Manifest[T11],
  t12: Manifest[T12],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](
      ofType[T1],
      ofType[T2],
      ofType[T3],
      ofType[T4],
      ofType[T5],
      ofType[T6],
      ofType[T7],
      ofType[T8],
      ofType[T9],
      ofType[T10],
      ofType[T11]),
    ImmutableList.of[Type](ofType[T12]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toScala10: AnyRef => Any = asScala[T10]
  private[this] val toScala11: AnyRef => Any = asScala[T11]
  private[this] val toScala12: AnyRef => Any = asScala[T12]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    arg6: T6,
    arg7: T7,
    arg8: T8,
    arg9: T9,
    arg10: T10,
    arg11: T11,
    opt1: Option[T12] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 11) {
              FunctionUnit11O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                toScala8(args.get(7)).asInstanceOf[T8],
                toScala9(args.get(8)).asInstanceOf[T9],
                toScala10(args.get(9)).asInstanceOf[T10],
                toScala11(args.get(10)).asInstanceOf[T11]
              )
            } else {
              FunctionUnit11O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                toScala8(args.get(7)).asInstanceOf[T8],
                toScala9(args.get(8)).asInstanceOf[T9],
                toScala10(args.get(9)).asInstanceOf[T10],
                toScala11(args.get(10)).asInstanceOf[T11],
                Option(toScala12(args.get(11)).asInstanceOf[T12])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit11O1.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 11) {
              Future(
                FunctionUnit11O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  toScala8(args.get(7)).asInstanceOf[T8],
                  toScala9(args.get(8)).asInstanceOf[T9],
                  toScala10(args.get(9)).asInstanceOf[T10],
                  toScala11(args.get(10)).asInstanceOf[T11]
                ))
            } else {
              Future(
                FunctionUnit11O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  toScala8(args.get(7)).asInstanceOf[T8],
                  toScala9(args.get(8)).asInstanceOf[T9],
                  toScala10(args.get(9)).asInstanceOf[T10],
                  toScala11(args.get(10)).asInstanceOf[T11],
                  Option(toScala12(args.get(11)).asInstanceOf[T12])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 11) {
              FunctionUnit11O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                toScala8(args.get(7)).asInstanceOf[T8],
                toScala9(args.get(8)).asInstanceOf[T9],
                toScala10(args.get(9)).asInstanceOf[T10],
                toScala11(args.get(10)).asInstanceOf[T11]
              )
            } else {
              FunctionUnit11O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                toScala8(args.get(7)).asInstanceOf[T8],
                toScala9(args.get(8)).asInstanceOf[T9],
                toScala10(args.get(9)).asInstanceOf[T10],
                toScala11(args.get(10)).asInstanceOf[T11],
                Option(toScala12(args.get(11)).asInstanceOf[T12])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit2O6[E, T1, T2, T3, T4, T5, T6, T7, T8, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2]),
    ImmutableList.of[Type](ofType[T3], ofType[T4], ofType[T5], ofType[T6], ofType[T7], ofType[T8]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    opt1: Option[T3] = None,
    opt2: Option[T4] = None,
    opt3: Option[T5] = None,
    opt4: Option[T6] = None,
    opt5: Option[T7] = None,
    opt6: Option[T8] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O6.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else if (args.size == 3) {
              FunctionUnit2O6.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit2O6.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit2O6.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit2O6.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit2O6.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else {
              FunctionUnit2O6.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit2O6.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              Future(
                FunctionUnit2O6.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              Future(
                FunctionUnit2O6.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3])))
            } else if (args.size == 4) {
              Future(
                FunctionUnit2O6.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            } else if (args.size == 5) {
              Future(
                FunctionUnit2O6.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            } else if (args.size == 6) {
              Future(
                FunctionUnit2O6.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            } else if (args.size == 7) {
              Future(
                FunctionUnit2O6.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            } else {
              Future(
                FunctionUnit2O6.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O6.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else if (args.size == 3) {
              FunctionUnit2O6.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit2O6.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit2O6.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit2O6.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit2O6.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else {
              FunctionUnit2O6.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit2O7[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2]),
    ImmutableList
      .of[Type](ofType[T3], ofType[T4], ofType[T5], ofType[T6], ofType[T7], ofType[T8], ofType[T9]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    opt1: Option[T3] = None,
    opt2: Option[T4] = None,
    opt3: Option[T5] = None,
    opt4: Option[T6] = None,
    opt5: Option[T7] = None,
    opt6: Option[T8] = None,
    opt7: Option[T9] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else if (args.size == 3) {
              FunctionUnit2O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit2O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit2O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit2O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit2O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit2O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else {
              FunctionUnit2O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit2O7.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              Future(
                FunctionUnit2O7.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              Future(
                FunctionUnit2O7.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3])))
            } else if (args.size == 4) {
              Future(
                FunctionUnit2O7.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            } else if (args.size == 5) {
              Future(
                FunctionUnit2O7.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            } else if (args.size == 6) {
              Future(
                FunctionUnit2O7.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            } else if (args.size == 7) {
              Future(
                FunctionUnit2O7.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            } else if (args.size == 8) {
              Future(
                FunctionUnit2O7.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            } else {
              Future(
                FunctionUnit2O7.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else if (args.size == 3) {
              FunctionUnit2O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit2O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit2O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit2O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit2O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit2O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else {
              FunctionUnit2O7.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit2O8[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  t10: Manifest[T10],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2]),
    ImmutableList.of[Type](
      ofType[T3],
      ofType[T4],
      ofType[T5],
      ofType[T6],
      ofType[T7],
      ofType[T8],
      ofType[T9],
      ofType[T10]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toScala10: AnyRef => Any = asScala[T10]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    opt1: Option[T3] = None,
    opt2: Option[T4] = None,
    opt3: Option[T5] = None,
    opt4: Option[T6] = None,
    opt5: Option[T7] = None,
    opt6: Option[T8] = None,
    opt7: Option[T9] = None,
    opt8: Option[T10] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else if (args.size == 3) {
              FunctionUnit2O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit2O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit2O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit2O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit2O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit2O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit2O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else {
              FunctionUnit2O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit2O8.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              Future(
                FunctionUnit2O8.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              Future(
                FunctionUnit2O8.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3])))
            } else if (args.size == 4) {
              Future(
                FunctionUnit2O8.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            } else if (args.size == 5) {
              Future(
                FunctionUnit2O8.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            } else if (args.size == 6) {
              Future(
                FunctionUnit2O8.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            } else if (args.size == 7) {
              Future(
                FunctionUnit2O8.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            } else if (args.size == 8) {
              Future(
                FunctionUnit2O8.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            } else if (args.size == 9) {
              Future(
                FunctionUnit2O8.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9])
                ))
            } else {
              Future(
                FunctionUnit2O8.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else if (args.size == 3) {
              FunctionUnit2O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit2O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit2O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit2O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit2O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit2O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit2O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else {
              FunctionUnit2O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit2O11[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  t10: Manifest[T10],
  t11: Manifest[T11],
  t12: Manifest[T12],
  t13: Manifest[T13],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2]),
    ImmutableList.of[Type](
      ofType[T3],
      ofType[T4],
      ofType[T5],
      ofType[T6],
      ofType[T7],
      ofType[T8],
      ofType[T9],
      ofType[T10],
      ofType[T11],
      ofType[T12],
      ofType[T13]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toScala10: AnyRef => Any = asScala[T10]
  private[this] val toScala11: AnyRef => Any = asScala[T11]
  private[this] val toScala12: AnyRef => Any = asScala[T12]
  private[this] val toScala13: AnyRef => Any = asScala[T13]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    opt1: Option[T3] = None,
    opt2: Option[T4] = None,
    opt3: Option[T5] = None,
    opt4: Option[T6] = None,
    opt5: Option[T7] = None,
    opt6: Option[T8] = None,
    opt7: Option[T9] = None,
    opt8: Option[T10] = None,
    opt9: Option[T11] = None,
    opt10: Option[T12] = None,
    opt11: Option[T13] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else if (args.size == 3) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else if (args.size == 10) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            } else if (args.size == 11) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11])
              )
            } else if (args.size == 12) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11]),
                Option(toScala12(args.get(11)).asInstanceOf[T12])
              )
            } else {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11]),
                Option(toScala12(args.get(11)).asInstanceOf[T12]),
                Option(toScala13(args.get(12)).asInstanceOf[T13])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit2O11.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 2) {
              Future(
                FunctionUnit2O11.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2]))
            } else if (args.size == 3) {
              Future(
                FunctionUnit2O11.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3])))
            } else if (args.size == 4) {
              Future(
                FunctionUnit2O11.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            } else if (args.size == 5) {
              Future(
                FunctionUnit2O11.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            } else if (args.size == 6) {
              Future(
                FunctionUnit2O11.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            } else if (args.size == 7) {
              Future(
                FunctionUnit2O11.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            } else if (args.size == 8) {
              Future(
                FunctionUnit2O11.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            } else if (args.size == 9) {
              Future(
                FunctionUnit2O11.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9])
                ))
            } else if (args.size == 10) {
              Future(
                FunctionUnit2O11.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10])
                ))
            } else if (args.size == 11) {
              Future(
                FunctionUnit2O11.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10]),
                  Option(toScala11(args.get(10)).asInstanceOf[T11])
                ))
            } else if (args.size == 12) {
              Future(
                FunctionUnit2O11.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10]),
                  Option(toScala11(args.get(10)).asInstanceOf[T11]),
                  Option(toScala12(args.get(11)).asInstanceOf[T12])
                ))
            } else {
              Future(
                FunctionUnit2O11.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  Option(toScala3(args.get(2)).asInstanceOf[T3]),
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10]),
                  Option(toScala11(args.get(10)).asInstanceOf[T11]),
                  Option(toScala12(args.get(11)).asInstanceOf[T12]),
                  Option(toScala13(args.get(12)).asInstanceOf[T13])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 2) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2])
            } else if (args.size == 3) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else if (args.size == 10) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            } else if (args.size == 11) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11])
              )
            } else if (args.size == 12) {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11]),
                Option(toScala12(args.get(11)).asInstanceOf[T12])
              )
            } else {
              FunctionUnit2O11.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                Option(toScala3(args.get(2)).asInstanceOf[T3]),
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11]),
                Option(toScala12(args.get(11)).asInstanceOf[T12]),
                Option(toScala13(args.get(12)).asInstanceOf[T13])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit8O1[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](
      ofType[T1],
      ofType[T2],
      ofType[T3],
      ofType[T4],
      ofType[T5],
      ofType[T6],
      ofType[T7],
      ofType[T8]),
    ImmutableList.of[Type](ofType[T9]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    arg6: T6,
    arg7: T7,
    arg8: T8,
    opt1: Option[T9] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 8) {
              FunctionUnit8O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                toScala8(args.get(7)).asInstanceOf[T8]
              )
            } else {
              FunctionUnit8O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                toScala8(args.get(7)).asInstanceOf[T8],
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit8O1.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 8) {
              Future(
                FunctionUnit8O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  toScala8(args.get(7)).asInstanceOf[T8]
                ))
            } else {
              Future(
                FunctionUnit8O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  toScala8(args.get(7)).asInstanceOf[T8],
                  Option(toScala9(args.get(8)).asInstanceOf[T9])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 8) {
              FunctionUnit8O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                toScala8(args.get(7)).asInstanceOf[T8]
              )
            } else {
              FunctionUnit8O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                toScala8(args.get(7)).asInstanceOf[T8],
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit10O1[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  t10: Manifest[T10],
  t11: Manifest[T11],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](
      ofType[T1],
      ofType[T2],
      ofType[T3],
      ofType[T4],
      ofType[T5],
      ofType[T6],
      ofType[T7],
      ofType[T8],
      ofType[T9],
      ofType[T10]),
    ImmutableList.of[Type](ofType[T11]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toScala10: AnyRef => Any = asScala[T10]
  private[this] val toScala11: AnyRef => Any = asScala[T11]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    arg6: T6,
    arg7: T7,
    arg8: T8,
    arg9: T9,
    arg10: T10,
    opt1: Option[T11] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 10) {
              FunctionUnit10O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                toScala8(args.get(7)).asInstanceOf[T8],
                toScala9(args.get(8)).asInstanceOf[T9],
                toScala10(args.get(9)).asInstanceOf[T10]
              )
            } else {
              FunctionUnit10O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                toScala8(args.get(7)).asInstanceOf[T8],
                toScala9(args.get(8)).asInstanceOf[T9],
                toScala10(args.get(9)).asInstanceOf[T10],
                Option(toScala11(args.get(10)).asInstanceOf[T11])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit10O1.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 10) {
              Future(
                FunctionUnit10O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  toScala8(args.get(7)).asInstanceOf[T8],
                  toScala9(args.get(8)).asInstanceOf[T9],
                  toScala10(args.get(9)).asInstanceOf[T10]
                ))
            } else {
              Future(
                FunctionUnit10O1.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  toScala4(args.get(3)).asInstanceOf[T4],
                  toScala5(args.get(4)).asInstanceOf[T5],
                  toScala6(args.get(5)).asInstanceOf[T6],
                  toScala7(args.get(6)).asInstanceOf[T7],
                  toScala8(args.get(7)).asInstanceOf[T8],
                  toScala9(args.get(8)).asInstanceOf[T9],
                  toScala10(args.get(9)).asInstanceOf[T10],
                  Option(toScala11(args.get(10)).asInstanceOf[T11])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 10) {
              FunctionUnit10O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                toScala8(args.get(7)).asInstanceOf[T8],
                toScala9(args.get(8)).asInstanceOf[T9],
                toScala10(args.get(9)).asInstanceOf[T10]
              )
            } else {
              FunctionUnit10O1.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                toScala8(args.get(7)).asInstanceOf[T8],
                toScala9(args.get(8)).asInstanceOf[T9],
                toScala10(args.get(9)).asInstanceOf[T10],
                Option(toScala11(args.get(10)).asInstanceOf[T11])
              )
            }
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit8[E, T1, T2, T3, T4, T5, T6, T7, T8, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](
      ofType[T1],
      ofType[T2],
      ofType[T3],
      ofType[T4],
      ofType[T5],
      ofType[T6],
      ofType[T7],
      ofType[T8]),
    ImmutableList.of[Type](),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    arg4: T4,
    arg5: T5,
    arg6: T6,
    arg7: T7,
    arg8: T8
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = FunctionUnit8.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              toScala4(args.get(3)).asInstanceOf[T4],
              toScala5(args.get(4)).asInstanceOf[T5],
              toScala6(args.get(5)).asInstanceOf[T6],
              toScala7(args.get(6)).asInstanceOf[T7],
              toScala8(args.get(7)).asInstanceOf[T8]
            )
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit8.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = Future(
              FunctionUnit8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                toScala4(args.get(3)).asInstanceOf[T4],
                toScala5(args.get(4)).asInstanceOf[T5],
                toScala6(args.get(5)).asInstanceOf[T6],
                toScala7(args.get(6)).asInstanceOf[T7],
                toScala8(args.get(7)).asInstanceOf[T8]
              ))
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = FunctionUnit8.this.evaluate(
              context,
              toScala1(args.get(0)).asInstanceOf[T1],
              toScala2(args.get(1)).asInstanceOf[T2],
              toScala3(args.get(2)).asInstanceOf[T3],
              toScala4(args.get(3)).asInstanceOf[T4],
              toScala5(args.get(4)).asInstanceOf[T5],
              toScala6(args.get(5)).asInstanceOf[T6],
              toScala7(args.get(6)).asInstanceOf[T7],
              toScala8(args.get(7)).asInstanceOf[T8]
            )
            toJava(result)
          }
      }
    }
  }
}

abstract class FunctionUnit3O8[E, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, R](
  implicit t1: Manifest[T1],
  t2: Manifest[T2],
  t3: Manifest[T3],
  t4: Manifest[T4],
  t5: Manifest[T5],
  t6: Manifest[T6],
  t7: Manifest[T7],
  t8: Manifest[T8],
  t9: Manifest[T9],
  t10: Manifest[T10],
  t11: Manifest[T11],
  r: Manifest[R])
    extends FunctionUnit.Managed {

  private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)
  private[this] val r0 = if (isAsync) r.typeArguments.head else r
  val signature: Signature = new Signature(
    ImmutableList.of[Type](ofType[T1], ofType[T2], ofType[T3]),
    ImmutableList.of[Type](
      ofType[T4],
      ofType[T5],
      ofType[T6],
      ofType[T7],
      ofType[T8],
      ofType[T9],
      ofType[T10],
      ofType[T11]),
    ofType(r0)
  )
  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)
  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  private[this] val toScala1: AnyRef => Any = asScala[T1]
  private[this] val toScala2: AnyRef => Any = asScala[T2]
  private[this] val toScala3: AnyRef => Any = asScala[T3]
  private[this] val toScala4: AnyRef => Any = asScala[T4]
  private[this] val toScala5: AnyRef => Any = asScala[T5]
  private[this] val toScala6: AnyRef => Any = asScala[T6]
  private[this] val toScala7: AnyRef => Any = asScala[T7]
  private[this] val toScala8: AnyRef => Any = asScala[T8]
  private[this] val toScala9: AnyRef => Any = asScala[T9]
  private[this] val toScala10: AnyRef => Any = asScala[T10]
  private[this] val toScala11: AnyRef => Any = asScala[T11]
  private[this] val toJava: Any => AnyRef = asJava(r0)

  def evaluate(
    context: Context[E],
    arg1: T1,
    arg2: T2,
    arg3: T3,
    opt1: Option[T4] = None,
    opt2: Option[T5] = None,
    opt3: Option[T6] = None,
    opt4: Option[T7] = None,
    opt5: Option[T8] = None,
    opt6: Option[T9] = None,
    opt7: Option[T10] = None,
    opt8: Option[T11] = None
  ): R

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    if (isAsync) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 3) {
              FunctionUnit3O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3])
            } else if (args.size == 4) {
              FunctionUnit3O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit3O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit3O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit3O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit3O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit3O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else if (args.size == 10) {
              FunctionUnit3O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            } else {
              FunctionUnit3O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11])
              )
            }
            result.asInstanceOf[Future[AnyRef]] map { toJava(_) }
          }
      }
    } else if (FunctionUnit3O8.this.shouldApplyRateLimit) {
      new GeneralNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] =
          manageFuture(context, args) {
            val result = if (args.size == 3) {
              Future(
                FunctionUnit3O8.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3]))
            } else if (args.size == 4) {
              Future(
                FunctionUnit3O8.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4])
                ))
            } else if (args.size == 5) {
              Future(
                FunctionUnit3O8.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5])
                ))
            } else if (args.size == 6) {
              Future(
                FunctionUnit3O8.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6])
                ))
            } else if (args.size == 7) {
              Future(
                FunctionUnit3O8.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7])
                ))
            } else if (args.size == 8) {
              Future(
                FunctionUnit3O8.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8])
                ))
            } else if (args.size == 9) {
              Future(
                FunctionUnit3O8.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9])
                ))
            } else if (args.size == 10) {
              Future(
                FunctionUnit3O8.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10])
                ))
            } else {
              Future(
                FunctionUnit3O8.this.evaluate(
                  context,
                  toScala1(args.get(0)).asInstanceOf[T1],
                  toScala2(args.get(1)).asInstanceOf[T2],
                  toScala3(args.get(2)).asInstanceOf[T3],
                  Option(toScala4(args.get(3)).asInstanceOf[T4]),
                  Option(toScala5(args.get(4)).asInstanceOf[T5]),
                  Option(toScala6(args.get(5)).asInstanceOf[T6]),
                  Option(toScala7(args.get(6)).asInstanceOf[T7]),
                  Option(toScala8(args.get(7)).asInstanceOf[T8]),
                  Option(toScala9(args.get(8)).asInstanceOf[T9]),
                  Option(toScala10(args.get(9)).asInstanceOf[T10]),
                  Option(toScala11(args.get(10)).asInstanceOf[T11])
                ))
            }
            result.map(toJava)
          }
      }
    } else {
      new FunctionNode[E](funcName(), children) {
        override def getSignature: Signature = signature
        override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel
        override protected def computeClassName: String = funcName
        override protected def computePackageName: String = packageName

        override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef =
          manage(context, args) {
            val result = if (args.size == 3) {
              FunctionUnit3O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3])
            } else if (args.size == 4) {
              FunctionUnit3O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4])
              )
            } else if (args.size == 5) {
              FunctionUnit3O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5])
              )
            } else if (args.size == 6) {
              FunctionUnit3O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6])
              )
            } else if (args.size == 7) {
              FunctionUnit3O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7])
              )
            } else if (args.size == 8) {
              FunctionUnit3O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8])
              )
            } else if (args.size == 9) {
              FunctionUnit3O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9])
              )
            } else if (args.size == 10) {
              FunctionUnit3O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10])
              )
            } else {
              FunctionUnit3O8.this.evaluate(
                context,
                toScala1(args.get(0)).asInstanceOf[T1],
                toScala2(args.get(1)).asInstanceOf[T2],
                toScala3(args.get(2)).asInstanceOf[T3],
                Option(toScala4(args.get(3)).asInstanceOf[T4]),
                Option(toScala5(args.get(4)).asInstanceOf[T5]),
                Option(toScala6(args.get(5)).asInstanceOf[T6]),
                Option(toScala7(args.get(6)).asInstanceOf[T7]),
                Option(toScala8(args.get(7)).asInstanceOf[T8]),
                Option(toScala9(args.get(8)).asInstanceOf[T9]),
                Option(toScala10(args.get(9)).asInstanceOf[T10]),
                Option(toScala11(args.get(10)).asInstanceOf[T11])
              )
            }
            toJava(result)
          }
      }
    }
  }
}
