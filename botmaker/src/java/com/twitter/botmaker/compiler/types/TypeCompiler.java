package com.twitter.botmaker.compiler.types;

import com.google.common.base.Optional;
import com.google.common.collect.ImmutableList;

import org.antlr.runtime.tree.Tree;
import org.apache.thrift.TBase;

import com.twitter.botmaker.antlr.BotMakerLexer;
import com.twitter.botmaker.compiler.ClassCache;
import com.twitter.botmaker.compiler.Parser;
import com.twitter.botmaker.compiler.exceptions.ParseFailure;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.tuples.Pair;
import com.twitter.scrooge.ThriftStruct;

public final class TypeCompiler {

  private TypeCompiler() {

  }

  public static Type getType(
      String type,
      TypeContext context
  ) throws ParseFailure, SemanticCheckFailure {
    Tree tree = Parser.parseType(type);
    return getType(tree, context);
  }

  public static Type getType(
      Tree tree,
      TypeContext context
  ) throws ParseFailure, SemanticCheckFailure {
    if (tree.getType() == BotMakerLexer.GENERIC_TYPE) {
      return getCompositeType(tree, context);
    } else if (tree.getChildCount() > 1) {
      String className = getFullClassName(tree);
      return getTypeFromClassName(className);
    } else {
      String typeName = tree.getChild(0).getText();
      Optional<Type> typeFromContextOpt = context.getTypeFromName(typeName);
      Optional<Type> primitiveTypeOpt = getPrimitiveType(typeName);
      if (typeFromContextOpt.isPresent()) {
        return typeFromContextOpt.get();
      } else if (primitiveTypeOpt.isPresent()) {
        return primitiveTypeOpt.get();
      } else {
        throw new ParseFailure(String.format(
            "col %d: Unrecognized type: %s",
            tree.getCharPositionInLine() + 1, typeName));
      }
    }
  }

  private static String getFullClassName(Tree tree) {
    StringBuilder builder = new StringBuilder();
    builder.append(tree.getChild(0).getText());
    for (int index = 1; index < tree.getChildCount(); index++) {
      builder.append('.').append(tree.getChild(index).getText());
    }
    return builder.toString();
  }

  private static Type getCompositeType(
      Tree tree,
      TypeContext context
  ) throws ParseFailure, SemanticCheckFailure {
    String tname = tree.getChild(0).getText();
    Optional<Pair<Class<?>, Integer>> infoOpt = Type.getCompositeClassInfoFromName(tname);
    if (!infoOpt.isPresent()) {
      throw new ParseFailure(String.format(
          "col %d: Unrecognized type: %s",
          tree.getCharPositionInLine() + 1, tname));
    }
    Pair<Class<?>, Integer> info = infoOpt.get();
    if (info.getSecond() != tree.getChildCount() - 1 && info.getSecond() != Type.ANY_NUM_PARAMS) {
      throw new ParseFailure(String.format(
          "col %d: Type %s received %d parameters, %d expected",
          tree.getCharPositionInLine() + 1, tname, tree.getChildCount() - 1, info.getSecond()));
    }
    ImmutableList.Builder<Type> paramTypesBuilder = ImmutableList.builder();
    for (int i = 1; i < tree.getChildCount(); i++) {
      paramTypesBuilder.add(getType(tree.getChild(i), context));
    }
    return Type.of(info.getFirst(), paramTypesBuilder.build());
  }

  public static Optional<Type> getPrimitiveType(String name) {
    return Type.getPrimitiveTypeFromName(name);
  }

  public static Type getTypeFromClassName(String className) throws SemanticCheckFailure {

    Class<?> clazz;
    try {
      clazz = ClassCache.forName(className);
    } catch (Exception e) {
      throw new SemanticCheckFailure(
          String.format("cannot find class %s", className)
      );
    }

    if (TBase.class.isAssignableFrom(clazz)) {
      return Type.thriftOf((Class<? extends TBase>) clazz);
    } else if (ThriftStruct.class.isAssignableFrom(clazz)) {
      return Type.thriftStructOf((Class<? extends ThriftStruct>) clazz);
    } else {
      throw new SemanticCheckFailure(
          String.format("class has to be either a TBase or ThriftStruct: %s.", className)
      );
    }
  }
}
