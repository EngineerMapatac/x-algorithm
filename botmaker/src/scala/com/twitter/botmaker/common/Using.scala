package com.twitter.botmaker.common

import scala.util.Try
import scala.util.control.ControlThrowable
import scala.util.control.NonFatal

object Using {

  def apply[R: Releasable, A](resource: => R)(f: R => A): Try[A] = Try {
    Using.resource(resource)(f)
  }

  final class Manager private {
    import Manager._

    private var closed = false
    private[this] var resources: List[Resource[_]] = Nil

    def apply[R: Releasable](resource: R): R = {
      acquire(resource)
      resource
    }

    def acquire[R: Releasable](resource: R): Unit = {
      if (resource == null) throw new NullPointerException("null resource")
      if (closed) throw new IllegalStateException("Manager has already been closed")
      resources = new Resource(resource) :: resources
    }

    private def manage[A](op: Manager => A): A = {
      var toThrow: Throwable = null
      try {
        op(this)
      } catch {
        case t: Throwable =>
          toThrow = t
          null.asInstanceOf[A]
      } finally {
        closed = true
        var rs = resources
        resources = null
        while (rs.nonEmpty) {
          val resource = rs.head
          rs = rs.tail
          try resource.release()
          catch {
            case t: Throwable =>
              if (toThrow == null) toThrow = t
              else toThrow = preferentiallySuppress(toThrow, t)
          }
        }
        if (toThrow != null) throw toThrow
      }
    }
  }

  object Manager {

    def apply[A](op: Manager => A): Try[A] = Try { (new Manager).manage(op) }

    private final class Resource[R](resource: R)(implicit releasable: Releasable[R]) {
      def release(): Unit = releasable.release(resource)
    }
  }

  private def preferentiallySuppress(primary: Throwable, secondary: Throwable): Throwable = {
    def score(t: Throwable): Int = t match {
      case _: VirtualMachineError => 4
      case _: LinkageError => 3
      case _: InterruptedException | _: ThreadDeath => 2
      case _: ControlThrowable => 0
      case e if !NonFatal(e) => 1
      case _ => -1
    }
    @inline def suppress(t: Throwable, suppressed: Throwable): Throwable = {
      t.addSuppressed(suppressed); t
    }

    if (score(secondary) > score(primary)) suppress(secondary, primary)
    else suppress(primary, secondary)
  }

  def resource[R, A](resource: R)(body: R => A)(implicit releasable: Releasable[R]): A = {
    if (resource == null) throw new NullPointerException("null resource")

    var toThrow: Throwable = null
    try {
      body(resource)
    } catch {
      case t: Throwable =>
        toThrow = t
        null.asInstanceOf[A]
    } finally {
      if (toThrow eq null) releasable.release(resource)
      else {
        try releasable.release(resource)
        catch { case other: Throwable => toThrow = preferentiallySuppress(toThrow, other) }
        finally throw toThrow
      }
    }
  }

  def resources[R1: Releasable, R2: Releasable, A](
    resource1: R1,
    resource2: => R2
  )(
    body: (R1, R2) => A
  ): A =
    resource(resource1) { r1 =>
      resource(resource2) { r2 =>
        body(r1, r2)
      }
    }

  def resources[R1: Releasable, R2: Releasable, R3: Releasable, A](
    resource1: R1,
    resource2: => R2,
    resource3: => R3
  )(
    body: (R1, R2, R3) => A
  ): A =
    resource(resource1) { r1 =>
      resource(resource2) { r2 =>
        resource(resource3) { r3 =>
          body(r1, r2, r3)
        }
      }
    }

  def resources[R1: Releasable, R2: Releasable, R3: Releasable, R4: Releasable, A](
    resource1: R1,
    resource2: => R2,
    resource3: => R3,
    resource4: => R4
  )(
    body: (R1, R2, R3, R4) => A
  ): A =
    resource(resource1) { r1 =>
      resource(resource2) { r2 =>
        resource(resource3) { r3 =>
          resource(resource4) { r4 =>
            body(r1, r2, r3, r4)
          }
        }
      }
    }

  trait Releasable[-R] {

    def release(resource: R): Unit
  }

  object Releasable {

    implicit object AutoCloseableIsReleasable extends Releasable[AutoCloseable] {
      def release(resource: AutoCloseable): Unit = resource.close()
    }
  }

}
