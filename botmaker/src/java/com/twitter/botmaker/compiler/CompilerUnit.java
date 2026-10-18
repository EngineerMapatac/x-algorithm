package com.twitter.botmaker.compiler;

import java.lang.reflect.Constructor;
import java.util.List;
import java.util.Objects;
import java.util.function.BiPredicate;

import com.google.common.base.Throwables;
import com.google.common.collect.ImmutableList;

import org.apache.thrift.TBase;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.StructTupleType;
import com.twitter.botmaker.compiler.types.ThriftStructType;
import com.twitter.botmaker.compiler.types.ThriftType;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.Constant;
import com.twitter.botmaker.function.collection.ToStructTuple;
import com.twitter.botmaker.function.conversion.ToPair;
import com.twitter.botmaker.function.thrift.ThriftOperator;
import com.twitter.botmaker.function.thrift.ThriftStructOperator;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.scrooge.ThriftStruct;

public abstract class CompilerUnit {


  public static final BiPredicate<CompilerUnit, CompilerUnit> EQUALITY_CHECK = (l, r) ->
          Objects.equals(l.fingerprint(), r.fingerprint());

  public abstract String funcName();

  public boolean enabled() {
    return true;
  }

  public String toString() {
    return String.format(
        "funcName: %s, description: %s",
        this.funcName(),
        this.botMakerDoc().description()
    );
  }

  public abstract BotMakerDoc botMakerDoc();

  public abstract Fingerprint fingerprint();

  public abstract ASTNode createASTNode(
      ImmutableList<ASTNode> children
  ) throws SemanticCheckFailure;

  static List<CompilerUnit> of(Class<? extends ASTNode> clazz) {

    Constructor<? extends ASTNode> constructor;
    try {
      constructor = clazz.getConstructor(String.class, ImmutableList.class);
    } catch (final Exception e) {
      throw new RuntimeException(
          String.format("Failed to find suitable constructor for '%s'",
              clazz.getSimpleName()), e);
    }

    final List<String> funcNames;
    final BotMakerFunction annotation = clazz.getAnnotation(BotMakerFunction.class);
    if (annotation.name().length > 0) {
      funcNames = ImmutableList.copyOf(annotation.name());
    } else {
      funcNames = ImmutableList.of(clazz.getSimpleName());
    }

    ImmutableList.Builder<CompilerUnit> compileUnits = ImmutableList.builder();
    for (String funcName : funcNames) {

      final String fn = funcName;
      compileUnits.add(new CompilerUnit() {
        @Override
        public String funcName() {
          return fn;
        }

        @Override
        public boolean enabled() {
          return true;
        }

        @Override
        public BotMakerDoc botMakerDoc() {
          return annotation.internal() ? BotMakerDoc.EMPTY : BotMakerDoc.of(fn, annotation);
        }

        @Override
        public Fingerprint fingerprint() {
          return Fingerprint.EMPTY;
        }

        @Override
        public ASTNode createASTNode(ImmutableList<ASTNode> children)
            throws SemanticCheckFailure {
          try {
            return constructor.newInstance(fn, children);
          } catch (Exception e) {
            Throwables.propagateIfInstanceOf(e.getCause(), SemanticCheckFailure.class);

            throw new RuntimeException(
                String.format("Failed to create instance of '%s'", fn), e);
          }
        }
      });
    }
    return compileUnits.build();
  }

  private static final Type TPA = Type.newGenericType();
  private static final Type TPB = Type.newGenericType();

  static List<CompilerUnit> of(String fn, ThriftType thriftType) {

    return ImmutableList.of(new CompilerUnit() {

      private final ASTNode.Signature signature = new ASTNode.Signature(
          Type.pairOf(TPA, TPB),
          thriftType
      );

      private final BotMakerDoc description = BotMakerDoc.of(
          fn,
          signature,
          String.format("Create a new %s thrift object from the name value pairs", thriftType),
          ActionLevel.NO_ACTION,
          "name value pairs to initialize the new thrift object."
      );

      @Override
      public String funcName() {
        return fn;
      }

      @Override
      public boolean enabled() {
        return true;
      }

      @Override
      public BotMakerDoc botMakerDoc() {
        return description;
      }

      @Override
      public Fingerprint fingerprint() {
        return Fingerprint.EMPTY;
      }

      @Override
      public ASTNode createASTNode(ImmutableList<ASTNode> children) throws SemanticCheckFailure {
        return new ThriftOperator<Runtime, TBase>(
            (Class<TBase>) thriftType.typeBase, fn, children) {
          @Override
          public Signature getSignature() {
            return signature;
          }

          @Override
          protected Object apply(
              Context<Runtime> context, TBase thriftObject) {
            return thriftObject;
          }
        };
      }
    });
  }

  static List<CompilerUnit> of(String fn, ThriftStructType thriftStructType) {

    return ImmutableList.of(new CompilerUnit() {
      private final ASTNode.Signature signature = new ASTNode.Signature(
          Type.pairOf(TPA, TPB),
          thriftStructType
      );

      private final BotMakerDoc description = BotMakerDoc.of(
          fn,
          signature,
          String.format(
              "Create a new %s thrift object from the name value pairs", thriftStructType),
          ActionLevel.NO_ACTION,
          "name value pairs to initialize the new thrift object."
      );

      @Override
      public String funcName() {
        return fn;
      }

      @Override
      public boolean enabled() {
        return true;
      }

      @Override
      public BotMakerDoc botMakerDoc() {
        return description;
      }

      @Override
      public Fingerprint fingerprint() {
        return Fingerprint.EMPTY;
      }

      @Override
      public ASTNode createASTNode(ImmutableList<ASTNode> children) throws SemanticCheckFailure {
        return new ThriftStructOperator<Runtime, ThriftStruct>(
            (Class<ThriftStruct>) thriftStructType.typeBase, fn, children) {
          @Override
          public Signature getSignature() {
            return signature;
          }

          @Override
          protected Object apply(
              Context<Runtime> context, ThriftStruct thriftObject) {
            return thriftObject;
          }
        };
      }
    });
  }

  static List<CompilerUnit> of(String fn, StructTupleType structTupleType) {

    return ImmutableList.of(new CompilerUnit() {
      private final ASTNode.Signature signature = new ASTNode.Signature(
          structTupleType.fieldTypes,
          structTupleType
      );

      private final BotMakerDoc description = BotMakerDoc.of(
          fn,
          signature,
          String.format("Create a new %s object from the name value pairs", structTupleType),
          ActionLevel.NO_ACTION,
          "name value pairs to initialize the new struct object."
      );

      @Override
      public String funcName() {
        return fn;
      }

      @Override
      public boolean enabled() {
        return true;
      }

      @Override
      public BotMakerDoc botMakerDoc() {
        return description;
      }

      @Override
      public Fingerprint fingerprint() {
        return Fingerprint.EMPTY;
      }

      @Override
      public ASTNode createASTNode(ImmutableList<ASTNode> children) throws SemanticCheckFailure {

        ImmutableList<String> fieldNames = structTupleType.fieldNames;
        ASTNode[] fieldValues = new ASTNode[fieldNames.size()];

        for (ASTNode child : children) {
          if (!(child instanceof ToPair)) {
            throw new SemanticCheckFailure(
                String.format(
                    "StructTuple expects pairs of name and values, but a %s received", child)
            );
          }

          String fieldName = ((Constant) child.getChildren().get(0)).getValue().toString();
          if (!fieldNames.contains(fieldName)) {
            throw new SemanticCheckFailure(
                String.format("Field %s does not belong to %s", fieldName, toString())
            );
          }
          fieldValues[fieldNames.indexOf(fieldName)] = (ASTNode) child.getChildren().get(1);
        }
        return new ToStructTuple(fn, fieldNames, ImmutableList.copyOf(fieldValues)) {
          @Override
          public Signature getSignature() {
            return signature;
          }
        };
      }
    });
  }
}
