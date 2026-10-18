package com.twitter.botmaker.function.datetime;

import java.text.DateFormat;
import java.text.ParseException;
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
        "String",
        "[String]"
    },
    arguments = {
        "date time string",
        "format"
    },
    returnType = "Long",
    description = "Basically it converts a human readable string to a Unix timestamp in"
        + " milliseconds. The timezone is UTC by default unless it is specified in the"
        + " format string.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "StringToTimeMs(\"01/15/2015\", \"MM/dd/yyyy\")",
        "StringToTimeMs(\"PST_2015/01/15_11:52:31.233)\")"
    }
)
public class StringToTimeMs extends FunctionNode<Runtime> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.STRING),
      ImmutableList.of(Type.STRING),
      Type.LONG
  );

  public StringToTimeMs(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
  }

  private static final String DEFAULT_FORMAT = "z_yyyy/MM/dd_HH:mm:ss.SSS";
  private static final String DEFAULT_TIMEZONE = "UTC";

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  @Override
  protected Object apply(Context<Runtime> context, List<Object> args) {

    @SuppressWarnings("unchecked")
    String text = (String) args.get(0);
    String format = (args.size() > 1) ? (String) args.get(1) : DEFAULT_FORMAT;
    DateFormat dateFormat = TwitterDateFormat.apply(format);
    dateFormat.setTimeZone(TimeZone.getTimeZone(DEFAULT_TIMEZONE));
    try {
      return dateFormat.parse(text).getTime();
    } catch (ParseException ex) {
      throw new DateStringParseFailure("Date String Format Error:", ex);
    }
  }

  private class DateStringParseFailure extends RuntimeException {
    public DateStringParseFailure(String msg, Throwable throwable) {
      super(msg, throwable);
    }
  }
}
