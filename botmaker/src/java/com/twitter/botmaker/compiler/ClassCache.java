package com.twitter.botmaker.compiler;

import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.util.Set;

import com.google.common.cache.CacheBuilder;
import com.google.common.cache.CacheLoader;
import com.google.common.cache.LoadingCache;
import com.google.common.collect.ImmutableSet;

import org.reflections.ReflectionUtils;
import org.reflections.Reflections;
import org.reflections.util.ClasspathHelper;
import org.reflections.util.ConfigurationBuilder;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.compiler.tuples.Tuple3;

import static org.reflections.ReflectionUtils.withAnnotation;

public final class ClassCache {

  private static final LoadingCache<String, Class>
      CLASS_CACHE = CacheBuilder.newBuilder().build(
      new CacheLoader<String, Class>() {
        @Override
        public Class load(String s) throws Exception {
          return Class.forName(s);
        }
      });

  public static final LoadingCache<Tuple3<String, Class, Set<String>>, Reflections>
      REFLECTIONS_LOADING_CACHE = CacheBuilder.newBuilder().build(
      new CacheLoader<Tuple3<String, Class, Set<String>>, Reflections>() {
        @Override
        public Reflections load(Tuple3<String, Class, Set<String>> key) {
          ConfigurationBuilder configurationBuilder = new ConfigurationBuilder();
          configurationBuilder
              .setUrls(ClasspathHelper.forPackage(key.getFirst()))
              .setUrls(ClasspathHelper.forClass(key.getSecond()))
              .forPackages(key.getThird().toArray(new String[0]));
          return new Reflections(configurationBuilder);
        }
      }
  );

  static ImmutableSet<Class<? extends ASTNode>> forASTNodes(Reflections reflections) {
    Set<Class<? extends ASTNode>> classes = reflections.getSubTypesOf(ASTNode.class);
    return ImmutableSet.copyOf(
        ReflectionUtils.getAll(classes, withAnnotation(BotMakerFunction.class)));
  }

  static ImmutableSet<CompilerUnit> forCompilerUnits(Reflections reflections) {
    Set<Class<? extends CompilerUnit>>
        classes = reflections.getSubTypesOf(CompilerUnit.class);
    ImmutableSet.Builder<CompilerUnit> builder = ImmutableSet.builder();
    for (Class<? extends CompilerUnit> cls : classes) {
      if (!Modifier.isAbstract(cls.getModifiers())) {
        try {
          Field field = cls.getField("MODULE$");
          Object module = field.get(null);
          if (module instanceof CompilerUnit) {
            CompilerUnit cu = (CompilerUnit) module;
            builder.add(cu);
          }
        } catch (Exception ex) {
        }
      }
    }
    return builder.build();
  }

  private ClassCache() {  }

  public static Class<?> forName(String cls) {
    return CLASS_CACHE.getUnchecked(cls);
  }
}
