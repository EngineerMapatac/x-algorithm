package com.twitter.botmaker.compiler.types;

import java.nio.ByteBuffer;
import java.util.Collection;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.function.BiPredicate;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableMap;
import com.google.common.collect.ImmutableSet;
import com.google.common.hash.Hasher;

import org.apache.thrift.TBase;
import org.apache.thrift.TFieldIdEnum;

import com.twitter.botmaker.compiler.Fingerprint;
import com.twitter.botmaker.compiler.Serializer;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.thrift.FieldMetaData;
import com.twitter.botmaker.compiler.thrift.FieldValueMetaData;
import com.twitter.botmaker.compiler.thrift.ListMetaData;
import com.twitter.botmaker.compiler.thrift.MapMetaData;
import com.twitter.botmaker.compiler.thrift.PrimitiveValueMetaData;
import com.twitter.botmaker.compiler.thrift.SetMetaData;
import com.twitter.botmaker.compiler.thrift.TBaseStructMetaData;
import com.twitter.botmaker.compiler.thrift.TEnumMetaData;
import com.twitter.botmaker.compiler.thrift.TType;
import com.twitter.util.Function;

public final class ThriftType extends Type implements FieldAccessibleType<TBase> {

  public static final BiPredicate<ThriftType, ThriftType> EQUALITY_CHECK = (l, r) ->
          Objects.equals(l.getFingerprint(), r.getFingerprint());

  private final Map<String, FieldMetaData> metadataMap;
  private final Fingerprint fingerprint;

  private ThriftType(Class<? extends TBase> thriftClass) throws SemanticCheckFailure {
    super(thriftClass, thriftClass.getName(), Serializer.thriftOf(thriftClass));
    this.metadataMap = getMetaData(thriftClass);
    Hasher hasher = Fingerprint.newHasher();
    hasher.putUnencodedChars(thriftClass.getName());
    computerFingerprint(hasher);
    this.fingerprint = new Fingerprint(0, hasher.hash().asBytes());
  }

  public Fingerprint getFingerprint() {
    return fingerprint;
  }

  static ThriftType ofThrift(Class<? extends TBase> typeBase) {

    return createType(typeBase, Function.exfunc0(() ->
        new ThriftType(typeBase)
    ));
  }

  public static ImmutableMap<String, FieldMetaData> getMetaData(
      Class<? extends TBase> thriftClass
  ) throws SemanticCheckFailure {

    Map<? extends TFieldIdEnum, org.apache.thrift.meta_data.FieldMetaData> metaDataMap =
        org.apache.thrift.meta_data.FieldMetaData.getStructMetaDataMap(thriftClass);

    Set<org.apache.thrift.meta_data.FieldValueMetaData> binaryFieldValueMetaDatas;
    try {
      java.lang.reflect.Field field = thriftClass.getField("binaryFieldValueMetaDatas");
      binaryFieldValueMetaDatas =
          (Set<org.apache.thrift.meta_data.FieldValueMetaData>) field.get(null);
    } catch (Exception e) {
      binaryFieldValueMetaDatas = ImmutableSet.of();
    }

    ImmutableMap.Builder<String, FieldMetaData> builder = ImmutableMap.builder();
    for (Map.Entry<? extends TFieldIdEnum, org.apache.thrift.meta_data.FieldMetaData>
        fieldAndMetaData : metaDataMap.entrySet()) {
      org.apache.thrift.meta_data.FieldMetaData metaData = fieldAndMetaData.getValue();
      FieldValueMetaData valueMetaData =
          getValueMetaData(metaData.valueMetaData, binaryFieldValueMetaDatas);
      builder.put(
          fieldAndMetaData.getKey().getFieldName(),
          new FieldMetaData(
              fieldAndMetaData.getKey().getFieldName(),
              fieldAndMetaData.getKey().getThriftFieldId(),
              metaData.requirementType,
              valueMetaData)
      );
    }

    return builder.build();
  }

  private static FieldValueMetaData getValueMetaData(
      org.apache.thrift.meta_data.FieldValueMetaData valueMetaData,
      Set<org.apache.thrift.meta_data.FieldValueMetaData> binaryFieldValueMetaDatas
  ) throws SemanticCheckFailure {
    switch (valueMetaData.type) {
      case TType.STRING:
        return binaryFieldValueMetaDatas.contains(valueMetaData)
            ? new PrimitiveValueMetaData(TType.BINARY)
            : new PrimitiveValueMetaData(TType.STRING);

      case TType.BOOL:
      case TType.BYTE:
      case TType.I16:
      case TType.I32:
      case TType.I64:
      case TType.DOUBLE:
        return new PrimitiveValueMetaData(valueMetaData.type);

      case TType.ENUM:
        org.apache.thrift.meta_data.EnumMetaData enumMetaData =
            (org.apache.thrift.meta_data.EnumMetaData) valueMetaData;
        return new TEnumMetaData(enumMetaData.enumClass);

      case TType.LIST:
        org.apache.thrift.meta_data.ListMetaData listMetaData =
            (org.apache.thrift.meta_data.ListMetaData) valueMetaData;
        FieldValueMetaData listElemMetaData =
            getValueMetaData(listMetaData.elemMetaData, binaryFieldValueMetaDatas);
        return new ListMetaData(listElemMetaData);

      case TType.SET:
        org.apache.thrift.meta_data.SetMetaData setMetaData =
            (org.apache.thrift.meta_data.SetMetaData) valueMetaData;
        FieldValueMetaData setElemMetaData =
            getValueMetaData(setMetaData.elemMetaData, binaryFieldValueMetaDatas);
        return new SetMetaData(setElemMetaData);

      case TType.MAP:
        org.apache.thrift.meta_data.MapMetaData mapMetaData =
            (org.apache.thrift.meta_data.MapMetaData) valueMetaData;
        FieldValueMetaData keyElemMetaData =
            getValueMetaData(mapMetaData.keyMetaData, binaryFieldValueMetaDatas);
        FieldValueMetaData valueElemMetaData =
            getValueMetaData(mapMetaData.valueMetaData, binaryFieldValueMetaDatas);
        return new MapMetaData(keyElemMetaData, valueElemMetaData);

      case TType.STRUCT:
        org.apache.thrift.meta_data.StructMetaData structMetaData =
            (org.apache.thrift.meta_data.StructMetaData) valueMetaData;
        return new TBaseStructMetaData(structMetaData.structClass);

      default:
        throw new SemanticCheckFailure(
            String.format("Unsupported field type %s", valueMetaData.type));
    }
  }

  @Override
  public Set<String> getFieldNames() {
    return metadataMap.keySet();
  }

  public FieldMetaData getFieldMetadata(String fieldName) {
    return metadataMap.get(fieldName);
  }

  @Override
  public Type getFieldType(String fieldName) throws SemanticCheckFailure {
    FieldMetaData metaData = metadataMap.get(fieldName);
    if (metaData == null) {
      throw new SemanticCheckFailure(
          String.format("field %s not found in the metadata of thrift class %s",
              fieldName, typeBase.getName()));
    }

    FieldValueMetaData valueMetaData = metaData.valueMetaData;
    return valueMetaData.getType();
  }

  @SuppressWarnings("unchecked")
  @Override
  public boolean isSetValue(TBase thriftObj, String fieldName) {
    FieldMetaData metaData = metadataMap.get(fieldName);
    return thriftObj.isSet(thriftObj.fieldForId(metaData.fieldId));
  }

  @SuppressWarnings("unchecked")
  @Override
  public Object getFieldValue(TBase thriftObj, String fieldName) {
    FieldMetaData metaData = metadataMap.get(fieldName);
    Object fieldValue = thriftObj.getFieldValue(thriftObj.fieldForId(metaData.fieldId));
    return getFieldValue(metaData.valueMetaData, fieldValue);
  }

  public TBase newInstance() throws Exception {
    TBase thriftObj = (TBase) typeBase.newInstance();
    return thriftObj;
  }

  public TBase newInstance(Map<Object, Object> map) throws Exception {
    TBase thriftObj = newInstance();
    for (Map.Entry<Object, Object> entry : map.entrySet()) {
      String fn = entry.getKey().toString();
      setFieldValue(thriftObj, fn, entry.getValue());
    }
    return thriftObj;
  }

  @SuppressWarnings("unchecked")
  public void setFieldValue(
      TBase thriftObj, String fieldName, Object fieldValue) throws Exception {
    FieldMetaData metaData = metadataMap.get(fieldName);
    if (metaData == null) {
      throw new SemanticCheckFailure(
          String.format("Field %s is not defined in thrift type %s", fieldName, typeBase.getName())
      );
    }
    Object value = extractFieldValue(metaData.valueMetaData, fieldValue, fieldName);
    thriftObj.setFieldValue(thriftObj.fieldForId(metaData.fieldId), value);
  }

  private static Object getListFieldValue(FieldValueMetaData valueMetaData, Object fieldValue) {
    ListMetaData listMetaData = (ListMetaData) valueMetaData;
    FieldValueMetaData elemMetaData = listMetaData.elemMetaData;
    if (!isConversionRequired(elemMetaData)) {
      return fieldValue;
    }

    ImmutableList.Builder<Object> builder = ImmutableList.builder();
    for (Object val : (Collection) fieldValue) {
      Object converted = getFieldValue(elemMetaData, val);
      builder.add(converted);
    }
    return builder.build();
  }

  private static Object getSetFieldValue(FieldValueMetaData valueMetaData, Object fieldValue) {
    SetMetaData setMetaData = (SetMetaData) valueMetaData;
    FieldValueMetaData elemMetaData = setMetaData.elemMetaData;
    if (!isConversionRequired(elemMetaData)) {
      return fieldValue;
    }

    ImmutableSet.Builder<Object> builder = ImmutableSet.builder();
    for (Object val : (Collection) fieldValue) {
      Object converted = getFieldValue(elemMetaData, val);
      builder.add(converted);
    }
    return builder.build();
  }

  @SuppressWarnings("unchecked")
  private static Object getMapFieldValue(FieldValueMetaData valueMetaData, Object fieldValue) {
    MapMetaData mapMetaData = (MapMetaData) valueMetaData;
    FieldValueMetaData keyMetaData = mapMetaData.keyMetaData;
    FieldValueMetaData valMetaData = mapMetaData.valueMetaData;

    boolean isKeyConversionRequired = isConversionRequired(keyMetaData);
    boolean isValueConversionRequired = isConversionRequired(valMetaData);
    if (!isKeyConversionRequired && !isValueConversionRequired) {
      return fieldValue;
    }

    ImmutableMap.Builder<Object, Object> builder = ImmutableMap.builder();
    Map<Object, Object> map = (Map<Object, Object>) fieldValue;
    for (Map.Entry<Object, Object> entry : map.entrySet()) {
      Object key = isKeyConversionRequired
          ? getFieldValue(keyMetaData, entry.getKey()) : entry.getKey();
      Object val = isValueConversionRequired
          ? getFieldValue(valMetaData, entry.getValue()) : entry.getValue();
      builder.put(key, val);
    }
    return builder.build();
  }

  public static Object getFieldValue(FieldValueMetaData valueMetaData, Object fieldValue) {
    switch (valueMetaData.ttype) {
      case TType.STRING:
      case TType.BOOL:
      case TType.DOUBLE:
      case TType.I64:
        return fieldValue;

      case TType.BINARY:
        if (fieldValue instanceof ByteBuffer) {
          return fieldValue;
        } else {
          byte[] bytes = (byte[]) fieldValue;
          return ByteBuffer.wrap(bytes);
        }

      case TType.BYTE:
      case TType.I16:
      case TType.I32:
        return ((Number) fieldValue).longValue();

      case TType.ENUM:
        return fieldValue.toString();

      case TType.LIST:
        return getListFieldValue(valueMetaData, fieldValue);

      case TType.SET:
        return getSetFieldValue(valueMetaData, fieldValue);

      case TType.MAP:
        return getMapFieldValue(valueMetaData, fieldValue);

      default:
        return fieldValue;
    }
  }

  private static boolean isConversionRequired(FieldValueMetaData valueMetaData) {
    switch (valueMetaData.ttype) {
      case TType.BYTE:
      case TType.I16:
      case TType.I32:
        return true;

      case TType.ENUM:
        return true;

      case TType.BINARY:
        return true;

      case TType.LIST:
        ListMetaData listMetaData = (ListMetaData) valueMetaData;
        return isConversionRequired(listMetaData.elemMetaData);

      case TType.SET:
        SetMetaData setMetaData = (SetMetaData) valueMetaData;
        return isConversionRequired(setMetaData.elemMetaData);

      case TType.MAP:
        MapMetaData mapMetaData = (MapMetaData) valueMetaData;
        return isConversionRequired(mapMetaData.keyMetaData)
            || isConversionRequired(mapMetaData.valueMetaData);

      default:
        return false;
    }
  }

  @SuppressWarnings("unchecked")
  private Object extractFieldValue(
      FieldValueMetaData valueMetaData, Object fieldValue, String fieldName) throws Exception {

    switch (valueMetaData.ttype) {
      case TType.STRING:
        checkIfAssignable(String.class, fieldValue.getClass(), fieldName);
        return fieldValue;

      case TType.BOOL:
        checkIfAssignable(Boolean.class, fieldValue.getClass(), fieldName);
        return fieldValue;

      case TType.DOUBLE:
        checkIfAssignable(Number.class, fieldValue.getClass(), fieldName);
        return ((Number) fieldValue).doubleValue();

      case TType.BINARY:
        checkIfAssignable(ByteBuffer.class, fieldValue.getClass(), fieldName);
        return fieldValue;

      case TType.I64:
        checkIfAssignable(Number.class, fieldValue.getClass(), fieldName);
      return ((Number) fieldValue).longValue();

      case TType.BYTE:
        checkIfAssignable(Number.class, fieldValue.getClass(), fieldName);
        return ((Number) fieldValue).byteValue();

      case TType.I16:
        checkIfAssignable(Number.class, fieldValue.getClass(), fieldName);
        return ((Number) fieldValue).shortValue();

      case TType.I32:
        checkIfAssignable(Number.class, fieldValue.getClass(), fieldName);
        return ((Number) fieldValue).intValue();

      case TType.ENUM:
        checkIfAssignable(String.class, fieldValue.getClass(), fieldName);
        TEnumMetaData enumMetaData = (TEnumMetaData) valueMetaData;
        return Enum.valueOf((Class<? extends Enum>) enumMetaData.enumClass, (String) fieldValue);

      case TType.LIST:
        checkIfAssignable(List.class, fieldValue.getClass(), fieldName);
        ListMetaData listMetaData = (ListMetaData) valueMetaData;
        return extractListFieldValue(listMetaData, (List<?>) fieldValue, fieldName);

      case TType.SET:
        checkIfAssignable(Set.class, fieldValue.getClass(), fieldName);
        SetMetaData setMetaData = (SetMetaData) valueMetaData;
        return extractSetFieldValue(setMetaData, (Set<?>) fieldValue, fieldName);

      case TType.MAP:
        checkIfAssignable(Map.class, fieldValue.getClass(), fieldName);
        MapMetaData mapMetaData = (MapMetaData) valueMetaData;
        return extractMapFieldValue(mapMetaData, (Map<?, ?>) fieldValue, fieldName);

      case TType.STRUCT:
        TBaseStructMetaData structMetaData = (TBaseStructMetaData) valueMetaData;
        if (structMetaData.structClass.isAssignableFrom(fieldValue.getClass())) {
          return fieldValue;
        }

        checkIfAssignable(Map.class, fieldValue.getClass(), fieldName);
        return extractStructFieldValue(structMetaData, (Map<Object, Object>) fieldValue);

      default:
        throw new ClassCastException();
    }
  }

  private void checkIfAssignable(Class ca, Class cb, String fieldName) {
    if (!ca.isAssignableFrom(cb)) {
      throw new ClassCastException(
          String.format("Class %s is not assignable from %s for field %s", ca, cb, fieldName)
      );
    }
  }

  private List<Object> extractListFieldValue(
      ListMetaData listMetaData, List<?> fieldValue, String fieldName) throws Exception {
    FieldValueMetaData elemMetaData = listMetaData.elemMetaData;

    ImmutableList.Builder<Object> builder = ImmutableList.builder();
    for (Object val : fieldValue) {
      Object converted = extractFieldValue(elemMetaData, val, fieldName);
      builder.add(converted);
    }
    return builder.build();
  }

  private Set<Object> extractSetFieldValue(
      SetMetaData setMetaData, Set<?> fieldValue, String fieldName) throws Exception {
    FieldValueMetaData elemMetaData = setMetaData.elemMetaData;

    ImmutableSet.Builder<Object> builder = ImmutableSet.builder();
    for (Object val : fieldValue) {
      Object converted = extractFieldValue(elemMetaData, val, fieldName);
      builder.add(converted);
    }
    return builder.build();
  }

  private Map<Object, Object> extractMapFieldValue(
      MapMetaData mapMetaData, Map<?, ?> fieldValue, String fieldName) throws Exception {
    FieldValueMetaData keyMetaData = mapMetaData.keyMetaData;
    FieldValueMetaData valueMetaData = mapMetaData.valueMetaData;

    ImmutableMap.Builder<Object, Object> builder = ImmutableMap.builder();
    Map<Object, Object> map = (Map<Object, Object>) fieldValue;
    for (Map.Entry<Object, Object> entry : map.entrySet()) {
      Object key = extractFieldValue(keyMetaData, entry.getKey(), fieldName);
      Object val = extractFieldValue(valueMetaData, entry.getValue(), fieldName);
      builder.put(key, val);
    }
    return builder.build();
  }

  private TBase extractStructFieldValue(
      TBaseStructMetaData structMetaData, Map<Object, Object> fieldValue) throws Exception {

    ThriftType thriftType = (ThriftType) ThriftType.ofThrift(structMetaData.structClass);
    return thriftType.newInstance(fieldValue);
  }
}
