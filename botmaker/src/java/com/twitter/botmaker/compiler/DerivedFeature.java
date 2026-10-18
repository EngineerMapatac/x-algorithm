package com.twitter.botmaker.compiler;

import java.util.Objects;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import com.google.common.base.Optional;
import com.google.common.base.Strings;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableSet;
import com.google.common.hash.Hasher;

import org.antlr.runtime.tree.Tree;

import com.twitter.botmaker.compiler.exceptions.ParseFailure;
import com.twitter.botmaker.compiler.tuples.Tuple3;

public interface DerivedFeature {

  String getName();

  String getDescription();

  String getExpr();

  boolean isLogged();

  Fingerprint getFingerprint();

  Tree getTree();

  ImmutableList<Tuple3<Tree, String, Optional<Tree>>> getParams();

  Optional<Tree> getReturnType();

  String getFixedReturnType();

  boolean isOverrideFunction();

  ImmutableSet<String> getHashTags();

  Pattern HASHTAG_PATTERN = Pattern.compile("#[a-z0-9A-Z]+\\b");

  static ImmutableSet<String> extractHashTags(String description) {
    Matcher matcher = HASHTAG_PATTERN.matcher(Strings.nullToEmpty(description));
    ImmutableSet.Builder<String> builder = ImmutableSet.builder();
    while (matcher.find()) {
      builder.add(matcher.group());
    }

    return builder.build();
  }

  static DerivedFeature of(
      String name,
      String description,
      String expr,
      boolean logged) throws ParseFailure {
    return of(name, description, expr, logged, "Auto", false);
  }

  static DerivedFeature of(
      String name,
      String description,
      String expr,
      boolean logged,
      String fixedReturnType,
      boolean overrideFunction) throws ParseFailure {

    Tuple3<Tree, ImmutableList<Tuple3<Tree, String, Optional<Tree>>>, Optional<Tree>> result =
        Parser.createParseParamsTree(expr);

    int cachedHashCode = Objects.hash(name, expr, description, logged);

    Hasher hasher = Fingerprint.newHasher();
    hasher.putUnencodedChars(name);
    hasher.putUnencodedChars(Strings.nullToEmpty(description));
    hasher.putUnencodedChars(Strings.nullToEmpty(expr));
    hasher.putBoolean(logged);
    Fingerprint fingerprint = new Fingerprint(0, hasher.hash().asBytes());

    ImmutableSet<String> hashTags = extractHashTags(description);

    return new DerivedFeature() {
      @Override
      public boolean isOverrideFunction() {
        return overrideFunction;
      }

      @Override
      public ImmutableSet<String> getHashTags() {
        return hashTags;
      }

      @Override
      public String getName() {
        return name;
      }

      @Override
      public String getDescription() {
        return description;
      }

      @Override
      public String getExpr() {
        return expr;
      }

      @Override
      public boolean isLogged() {
        return logged;
      }

      @Override
      public Fingerprint getFingerprint() {
        return fingerprint;
      }

      @Override
      public Tree getTree() {
        return result.getFirst();
      }

      @Override
      public ImmutableList<Tuple3<Tree, String, Optional<Tree>>> getParams() {
        return result.getSecond();
      }

      @Override
      public Optional<Tree> getReturnType() {
        return result.getThird();
      }

      @Override
      public String getFixedReturnType() {
        return fixedReturnType;
      }

      @Override
      public boolean equals(Object o) {
        if (this == o) {
          return true;
        } else if (o == null || getClass() != o.getClass()) {
          return false;
        }

        DerivedFeature that = (DerivedFeature) o;

        return cachedHashCode == that.hashCode()
            && Objects.equals(name, that.getName())
            && Objects.equals(description, that.getDescription())
            && Objects.equals(logged, that.isLogged())
            && Objects.equals(expr, that.getExpr());
      }

      @Override
      public String toString() {
        return name;
      }

      @Override
      public int hashCode() {
        return cachedHashCode;
      }
    };
  }
}
