package com.twitter.botmaker.compiler;

import com.google.common.base.Strings;
import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.compiler.types.Type;

public interface BotMakerDoc {
  String name();

  String description();

  ActionLevel actionLevel();

  boolean isDerivedFeature();

  boolean isInternal();

  String[] argTypes();

  String[] arguments();

  String returnType();

  BotMakerDoc EMPTY = new BotMakerDoc() {
    @Override
    public String name() {
      return null;
    }

    @Override
    public String description() {
      return null;
    }

    @Override
    public ActionLevel actionLevel() {
      return ActionLevel.UNDOCUMENTED;
    }

    @Override
    public boolean isDerivedFeature() {
      return false;
    }

    @Override
    public boolean isInternal() {
      return false;
    }

    @Override
    public String[] argTypes() {
      return new String[]{};
    }

    @Override
    public String[] arguments() {
      return new String[]{};
    }

    @Override
    public String returnType() {
      return null;
    }
  };

  static BotMakerDoc of(
      String name,
      String description,
      ActionLevel actionLevel,
      String[] argTypes,
      String[] arguments,
      String returnType
  ) {

    return new BotMakerDoc() {

      @Override
      public String name() {
        return name;
      }

      @Override
      public String description() {
        return description;
      }

      @Override
      public ActionLevel actionLevel() {
        return actionLevel;
      }

      @Override
      public boolean isDerivedFeature() {
        return false;
      }

      @Override
      public boolean isInternal() {
        return false;
      }

      @Override
      public String[] argTypes() {
        return argTypes;
      }

      @Override
      public String[] arguments() {
        return arguments;
      }

      @Override
      public String returnType() {
        return returnType;
      }
    };
  }

  static BotMakerDoc of(
      String name,
      ASTNode.Signature signature,
      String description,
      ActionLevel actionLevel,
      String... arguments) {

    return new BotMakerDoc() {

      @Override
      public String name() {
        return name;
      }

      @Override
      public String description() {
        return description;
      }

      @Override
      public ActionLevel actionLevel() {
        return actionLevel;
      }

      @Override
      public boolean isDerivedFeature() {
        return false;
      }

      @Override
      public boolean isInternal() {
        return false;
      }

      @Override
      public String[] argTypes() {

        ImmutableList.Builder<String> builder = ImmutableList.builder();
        for (Type type : signature.paramTypes) {
          builder.add(type.toString());
        }

        for (Type type : signature.optParamTypes) {
          builder.add("[" + type.toString() + "]");
        }

        if (signature.varargsType != null) {
          builder.add(signature.varargsType.toString() + "...");
        }

        return builder.build().toArray(new String[]{});
      }

      @Override
      public String[] arguments() {
        return arguments;
      }

      @Override
      public String returnType() {
        return signature.returnType.toString();
      }
    };
  }

  static BotMakerDoc of(String funcName, BotMakerFunction annotation) {

    return new BotMakerDoc() {

      @Override
      public String name() {
        return funcName;
      }

      @Override
      public String description() {
        return Strings.nullToEmpty(annotation.description());
      }

      @Override
      public ActionLevel actionLevel() {
        return annotation.actionLevel();
      }

      @Override
      public boolean isDerivedFeature() {
        return false;
      }

      @Override
      public boolean isInternal() {
        return annotation.internal();
      }

      @Override
      public String[] argTypes() {
        if (annotation.argTypes() != null) {
          return annotation.argTypes();
        } else {
          return new String[]{};
        }
      }

      @Override
      public String[] arguments() {
        if (annotation.arguments() != null) {
          return annotation.arguments();
        } else {
          return new String[]{};
        }
      }

      @Override
      public String returnType() {
        return Strings.nullToEmpty(annotation.returnType());
      }
    };
  }

  static BotMakerDoc of(DerivedFeature derivedFeature) {

    return new BotMakerDoc() {

      @Override
      public String name() {
        return derivedFeature.getName();
      }

      @Override
      public String description() {
        return derivedFeature.getDescription();
      }

      @Override
      public ActionLevel actionLevel() {
        return ActionLevel.RESERVED_FOR_DERIVED_FEATURE;
      }

      @Override
      public boolean isDerivedFeature() {
        return true;
      }

      @Override
      public boolean isInternal() {
        return false;
      }

      @Override
      public String[] argTypes() {
        return new String[]{};
      }

      @Override
      public String[] arguments() {
        return new String[]{};
      }

      @Override
      public String returnType() {
        return "";
      }
    };
  }

}
