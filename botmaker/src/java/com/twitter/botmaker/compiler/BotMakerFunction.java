package com.twitter.botmaker.compiler;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

@Target(ElementType.TYPE)
@Retention(RetentionPolicy.RUNTIME)
public @interface BotMakerFunction {
  String[] name() default {};

  String description() default "";

  ActionLevel actionLevel();

  boolean internal() default false;

  boolean deprecated() default false;

  String[] argTypes() default {};

  String[] arguments() default {};

  String returnType() default "";

  String[] examples() default {};
}
