package com.twitter.botmaker.compiler.types

import scala.collection.JavaConverters._
import scala.runtime.BoxedUnit
import com.google.common.base.Preconditions
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.compiler.tuples.Tuple

object ScalaType {

  def ofType[T](implicit t: Manifest[T]): Type = {

    if (t.runtimeClass == classOf[Unit]) {
      Type.UNIT
    } else if (t.runtimeClass == classOf[Byte]) {
      Type.LONG
    } else if (t.runtimeClass == classOf[Short]) {
      Type.LONG
    } else if (t.runtimeClass == classOf[Int]) {
      Type.LONG
    } else if (t.runtimeClass == classOf[Long]) {
      Type.LONG
    } else if (t.runtimeClass == classOf[Float]) {
      Type.DOUBLE
    } else if (t.runtimeClass == classOf[Double]) {
      Type.DOUBLE
    } else if (t.runtimeClass == classOf[Boolean]) {
      Type.BOOLEAN
    } else if (t.runtimeClass.getName == classOf[Option[_]].getName) {
      Preconditions.checkArgument(t.typeArguments.size == 1)
      ofType(t.typeArguments(0))
    } else if (t.runtimeClass.getName == classOf[Iterable[_]].getName) {
      Preconditions.checkArgument(t.typeArguments.size == 1)
      Type.collectionOf(ofType(t.typeArguments(0)))
    } else if (t.runtimeClass.getName == classOf[Seq[_]].getName) {
      Preconditions.checkArgument(t.typeArguments.size == 1)
      Type.listOf(ofType(t.typeArguments(0)))
    } else if (t.runtimeClass.getName == classOf[List[_]].getName) {
      Preconditions.checkArgument(t.typeArguments.size == 1)
      Type.listOf(ofType(t.typeArguments(0)))
    } else if (t.runtimeClass.getName == classOf[Set[_]].getName) {
      Preconditions.checkArgument(t.typeArguments.size == 1)
      Type.setOf(ofType(t.typeArguments(0)))
    } else if (t.runtimeClass.getName == classOf[Map[_, _]].getName) {
      Preconditions.checkArgument(t.typeArguments.size == 2)
      Type.mapOf(
        ofType(t.typeArguments(0)),
        ofType(t.typeArguments(1))
      )
    } else if (t.runtimeClass.getName.startsWith("scala.Tuple")) {
      Type.tupleOf(
        t.typeArguments.map(ofType(_)).toArray: _*
      )

    } else {
      Type.of(
        t.runtimeClass,
        ImmutableList.copyOf(t.typeArguments.map(ofType(_)).toArray)
      )
    }
  }

  def asList[T](x: Any)(implicit t: Manifest[_ <: T]): List[T] = x.asInstanceOf[List[T]]

  def asJava[T](implicit t: Manifest[T]): Any => AnyRef = {

    if (t.runtimeClass == classOf[Unit]) { (x: Any) => BoxedUnit.UNIT }
    else if (t.runtimeClass == classOf[Byte]) { (x: Any) => Byte.box(x.asInstanceOf[Byte]) }
    else if (t.runtimeClass == classOf[Short]) { (x: Any) => Short.box(x.asInstanceOf[Short]) }
    else if (t.runtimeClass == classOf[Int]) { (x: Any) => Int.box(x.asInstanceOf[Int]) }
    else if (t.runtimeClass == classOf[Long]) { (x: Any) => Long.box(x.asInstanceOf[Long]) }
    else if (t.runtimeClass == classOf[Float]) { (x: Any) =>
      Double.box(x.asInstanceOf[Float].toDouble)
    } else if (t.runtimeClass == classOf[Double]) { (x: Any) => Double.box(x.asInstanceOf[Double]) }
    else if (t.runtimeClass == classOf[Boolean]) { (x: Any) =>
      Boolean.box(x.asInstanceOf[Boolean])
    } else if (t.runtimeClass.getName == classOf[Option[_]].getName) {
      Preconditions.checkArgument(t.typeArguments.size == 1)
      val converter = asJava(t.typeArguments(0))
      (x: Any) =>
        x.asInstanceOf[Option[Any]] match {
          case Some(a) => converter(a)
          case None => null
        }
    } else if (t.runtimeClass.getName == classOf[Iterable[_]].getName) {
      Preconditions.checkArgument(t.typeArguments.size == 1)
      val converter = asJava(t.typeArguments(0))
      (x: Any) => {
        x.asInstanceOf[Iterable[Any]].map(converter(_)).asJava
      }
    } else if (t.runtimeClass.getName == classOf[Seq[_]].getName) {
      Preconditions.checkArgument(t.typeArguments.size == 1)
      val converter = asJava(t.typeArguments(0))
      (x: Any) => {
        x.asInstanceOf[Seq[Any]].map(converter(_)).asJava
      }
    } else if (t.runtimeClass.getName == classOf[List[_]].getName) {
      Preconditions.checkArgument(t.typeArguments.size == 1)
      val converter = asJava(t.typeArguments(0))
      (x: Any) => {
        x.asInstanceOf[List[Any]].map(converter(_)).asJava
      }
    } else if (t.runtimeClass.getName == classOf[Set[_]].getName) {
      Preconditions.checkArgument(t.typeArguments.size == 1)
      val converter = asJava(t.typeArguments(0))
      (x: Any) => {
        x.asInstanceOf[Set[Any]].map(converter(_)).asJava
      }
    } else if (t.runtimeClass.getName == classOf[Map[_, _]].getName) {
      Preconditions.checkArgument(t.typeArguments.size == 2)
      val kconverter = asJava(t.typeArguments(0))
      val vconverter = asJava(t.typeArguments(1))
      (x: Any) => {
        x.asInstanceOf[Map[Any, Any]] map {
          case (k, v) => kconverter(k) -> vconverter(v)
        } asJava
      }
    } else if (t.runtimeClass.getName.startsWith("scala.Tuple")) {
      val converts = t.typeArguments.map(asJava(_)).toArray
      (x: Any) => {
        val tuple = x.asInstanceOf[Product]
        Tuple.of(
          converts.zipWithIndex map {
            case (c, i) =>
              c(tuple.productElement(i))
          }
        )
      }
    } else { (x: Any) => x.asInstanceOf[AnyRef] }
  }

  def toTuple(elems: Array[Any]): Any = elems.length match {
    case 1 =>
      Tuple1.apply(elems(0))
    case 2 =>
      Tuple2.apply(elems(0), elems(1))
    case 3 =>
      Tuple3.apply(elems(0), elems(1), elems(2))
    case 4 =>
      Tuple4.apply(elems(0), elems(1), elems(2), elems(3))
    case 5 =>
      Tuple5.apply(elems(0), elems(1), elems(2), elems(3), elems(4))
    case 6 =>
      Tuple6.apply(elems(0), elems(1), elems(2), elems(3), elems(4), elems(5))
    case 7 =>
      Tuple7.apply(elems(0), elems(1), elems(2), elems(3), elems(4), elems(5), elems(6))
    case 8 =>
      Tuple8.apply(elems(0), elems(1), elems(2), elems(3), elems(4), elems(5), elems(6), elems(7))
    case 9 =>
      Tuple9.apply(
        elems(0),
        elems(1),
        elems(2),
        elems(3),
        elems(4),
        elems(5),
        elems(6),
        elems(7),
        elems(8)
      )
    case 10 =>
      Tuple10.apply(
        elems(0),
        elems(1),
        elems(2),
        elems(3),
        elems(4),
        elems(5),
        elems(6),
        elems(7),
        elems(8),
        elems(9)
      )
    case _ =>
      throw new IndexOutOfBoundsException
  }

  def asScala[T](implicit t: Manifest[T]): AnyRef => Any = {

    if (t.runtimeClass == classOf[Unit]) { (x: AnyRef) => () }
    else if (t.runtimeClass == classOf[Byte]) { (x: AnyRef) =>
      x.asInstanceOf[java.lang.Long].toByte
    } else if (t.runtimeClass == classOf[Short]) { (x: AnyRef) =>
      x.asInstanceOf[java.lang.Long].toShort
    } else if (t.runtimeClass == classOf[Int]) { (x: AnyRef) =>
      x.asInstanceOf[java.lang.Long].toInt
    } else if (t.runtimeClass == classOf[Long]) { (x: AnyRef) =>
      x.asInstanceOf[java.lang.Long].toLong
    } else if (t.runtimeClass == classOf[Float]) { (x: AnyRef) =>
      x.asInstanceOf[java.lang.Double].toFloat
    } else if (t.runtimeClass == classOf[Double]) { (x: AnyRef) =>
      x.asInstanceOf[java.lang.Double].toDouble
    } else if (t.runtimeClass == classOf[Boolean]) { (x: AnyRef) =>
      x.asInstanceOf[java.lang.Boolean].booleanValue()
    } else if (t.runtimeClass.getName == classOf[Option[_]].getName) {
      Preconditions.checkArgument(t.typeArguments.size == 1)
      val converter = asScala(t.typeArguments(0))
      (x: AnyRef) => {
        if (x != null) {
          Some(converter(x))
        } else {
          None
        }
      }

    } else if (t.runtimeClass.getName == classOf[Iterable[_]].getName) {
      Preconditions.checkArgument(t.typeArguments.size == 1)
      val converter = asScala(t.typeArguments(0))
      (x: AnyRef) => {
        x.asInstanceOf[java.util.Collection[AnyRef]].asScala.map(converter(_))
      }
    } else if (t.runtimeClass.getName == classOf[Seq[_]].getName) {
      Preconditions.checkArgument(t.typeArguments.size == 1)
      val converter = asScala(t.typeArguments(0))
      (x: AnyRef) => {
        x.asInstanceOf[java.util.List[AnyRef]].asScala.map(converter(_))
      }
    } else if (t.runtimeClass.getName == classOf[List[_]].getName) {
      Preconditions.checkArgument(t.typeArguments.size == 1)
      val converter = asScala(t.typeArguments(0))
      (x: AnyRef) => {
        x.asInstanceOf[java.util.List[AnyRef]].asScala.map(converter(_)).toList
      }
    } else if (t.runtimeClass.getName == classOf[Set[_]].getName) {
      Preconditions.checkArgument(t.typeArguments.size == 1)
      val converter = asScala(t.typeArguments(0))
      (x: AnyRef) => {
        x.asInstanceOf[java.util.Set[AnyRef]].asScala.map(converter(_)).toSet
      }
    } else if (t.runtimeClass.getName == classOf[Map[_, _]].getName) {
      Preconditions.checkArgument(t.typeArguments.size == 2)
      val kconverter = asScala(t.typeArguments(0))
      val vconverter = asScala(t.typeArguments(1))
      (x: AnyRef) => {
        x.asInstanceOf[java.util.Map[AnyRef, AnyRef]].asScala.toMap map {
          case (k, v) => kconverter(k) -> vconverter(v)
        }
      }
    } else if (t.runtimeClass.getName.startsWith("scala.Tuple")) {
      val converts = t.typeArguments.map(asScala(_)).toArray
      (x: AnyRef) => {
        val tuple = x.asInstanceOf[Tuple]

        toTuple(
          (0 to tuple.size - 1) map { i => converts(i)(tuple.get(i)) } toArray
        )
      }
    } else { (x: AnyRef) => x }

  }
}
