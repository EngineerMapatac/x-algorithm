package com.twitter.botmaker.function.datetime;

import java.text.DateFormat;
import java.util.Date;
import java.util.List;
import java.util.TimeZone;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.util.TwitterDateFormat;

@BotMakerFunction(
    argTypes = {
        "Long",
        "[String]",
        "[String]"
    },
    arguments = {
        "timestampMs",
        "format (default = \"z_yyyy/MM/dd_HH:mm:ss.SSS\")",
        "timeZone (default = \"America/Los_Angeles\")"
    },
    returnType = "String",
    description = "Converts a UTC timestamp in milliseconds to human readable strings. See"
        + " Java SimpleDateFormat for example of format. .",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "TimeMsToString(1421351551233, \"z_yyyy/MM/dd_HH:mm:ss.SSS\")",
        "TimeMsToString(1421351551233)",
        "TimeMsToString(1421351551233, \"MM/dd/yyyy\", \"UTC\")"
    }
)
  public class TimeMsToString extends FunctionNode<Runtime> {
  private static final Signature SIGNATURE;
  static {
    SIGNATURE = new Signature(
        ImmutableList.of(Type.LONG),
        ImmutableList.of(Type.STRING, Type.STRING),
        Type.STRING);
  }

  private static final String DEFAULT_FORMAT = "z_yyyy/MM/dd_HH:mm:ss.SSS";
  private static final String DEFAULT_TIMEZONE = "America/Los_Angeles";

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  public TimeMsToString(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  protected Object apply(Context<Runtime> context, List<Object> args) {

    final String format;
    if (args.size() > 1) {
      format = (String) args.get(1);
    } else {
      format = DEFAULT_FORMAT;
    }
    final String timeZone;
    if (args.size() > 2) {
      timeZone = (String) args.get(2);
    } else {
      timeZone = DEFAULT_TIMEZONE;
    }
    DateFormat dateFormat = TwitterDateFormat.apply(format);
    dateFormat.setTimeZone(TimeZone.getTimeZone(timeZone));
    return dateFormat.format(new Date((Long) args.get(0)));
  }
}
