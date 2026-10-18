package com.twitter.botmaker.compiler.types;

import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.nio.ByteBuffer;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.Stack;
import java.util.function.BiPredicate;

import scala.Option;
import scala.Product;
import scala.collection.JavaConverters;
import scala.reflect.Manifest;

import com.google.common.base.Preconditions;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableMap;
import com.google.common.collect.ImmutableSet;
import com.google.common.hash.Hasher;

import org.apache.thrift.TException;
import org.apache.thrift.protocol.TField;
import org.apache.thrift.protocol.TList;
import org.apache.thrift.protocol.TMap;
import org.apache.thrift.protocol.TMessage;
import org.apache.thrift.protocol.TProtocol;
import org.apache.thrift.protocol.TSet;
import org.apache.thrift.protocol.TStruct;

import com.twitter.botmaker.compiler.ClassCache;
import com.twitter.botmaker.compiler.Fingerprint;
import com.twitter.botmaker.compiler.Serializer;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.thrift.FieldMetaData;
import com.twitter.botmaker.compiler.thrift.FieldValueMetaData;
import com.twitter.botmaker.compiler.thrift.ListMetaData;
import com.twitter.botmaker.compiler.thrift.MapMetaData;
import com.twitter.botmaker.compiler.thrift.PrimitiveValueMetaData;
import com.twitter.botmaker.compiler.thrift.SetMetaData;
import com.twitter.botmaker.compiler.thrift.TFieldRequirementType;
import com.twitter.botmaker.compiler.thrift.TType;
import com.twitter.botmaker.compiler.thrift.ThriftEnumMetaData;
import com.twitter.botmaker.compiler.thrift.ThriftStructMetaData;
import com.twitter.scrooge.ThriftEnum;
import com.twitter.scrooge.ThriftStruct;
import com.twitter.scrooge.ThriftStructCodec;
import com.twitter.scrooge.ThriftStructFieldInfo;
import com.twitter.scrooge.ThriftUnion;
import com.twitter.scrooge.ThriftUnionFieldInfo;
import com.twitter.util.Function;

public final class ThriftStructType extends Type implements FieldAccessibleType<ThriftStruct> {

  public static final BiPredicate<ThriftStructType, ThriftStructType> EQUALITY_CHECK = (l, r) ->
          Objects.equals(l.getFingerprint(), r.getFingerprint());

  private final boolean isUnion;
  private final ThriftStructCodec codec;
  private final ImmutableMap<String, Integer> indexMap;
  private final ImmutableMap<String, FieldMetaData> metadataMap;
  private final Fingerprint fingerprint;

  private ThriftStructType(Class<? extends ThriftStruct> thriftClass) throws SemanticCheckFailure {
    super(thriftClass, thriftClass.getName(), Serializer.thriftStructOf(thriftClass));

    this.codec = getThriftStructCodec(thriftClass);

    List<ThriftStructFieldInfo> fieldInfos;
    if (ThriftUnion.class.isAssignableFrom(thriftClass)) {
      fieldInfos = getUnionFieldInfos(this.codec);
      this.isUnion = true;
    } else {
      fieldInfos = getStructFieldInfos(this.codec);
      this.isUnion = false;
    }

    ImmutableMap.Builder<String, Integer> indexBuilder = ImmutableMap.builder();
    ImmutableMap.Builder<String, FieldMetaData> metadataBuilder = ImmutableMap.builder();

    int index = 0;
    for (ThriftStructFieldInfo fieldInfo : fieldInfos) {
      indexBuilder.put(fieldInfo.tfield().name, index++);

      FieldValueMetaData fieldValueMetaData = getFieldValueMetadata(fieldInfo.manifest());
      FieldMetaData fieldMetaData = new FieldMetaData(
          fieldInfo.tfield().name,
          fieldInfo.tfield().id,
          fieldInfo.isRequired() ? TFieldRequirementType.REQUIRED
              : fieldInfo.isOptional() ? TFieldRequirementType.OPTIONAL
              : TFieldRequirementType.DEFAULT,
          fieldValueMetaData
      );
      metadataBuilder.put(fieldInfo.tfield().name, fieldMetaData);
    }

    this.indexMap = indexBuilder.build();
    this.metadataMap = metadataBuilder.build();
    Hasher hasher = Fingerprint.newHasher();
    hasher.putUnencodedChars(thriftClass.getName());
    computerFingerprint(hasher);
    this.fingerprint = new Fingerprint(0, hasher.hash().asBytes());
  }

  public Fingerprint getFingerprint() {
    return fingerprint;
  }

  static ThriftStructType ofThriftStruct(
      Class<? extends ThriftStruct> typeBase) {

    return createType(typeBase, Function.exfunc0(() ->
        new ThriftStructType(typeBase)
    ));
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
    FieldMetaData fieldMetaData = metadataMap.get(fieldName);
    if (fieldMetaData == null) {
      throw new SemanticCheckFailure(
          String.format("field %s not found in the metadata of thrift class %s",
              fieldName, typeBase.getName()));
    }

    return fieldMetaData.valueMetaData.getType();
  }

  @SuppressWarnings("unchecked")
  @Override
  public boolean isSetValue(ThriftStruct thriftObj, String fieldName) {
    if (this.isUnion) {
      ThriftUnion thriftUnion = (ThriftUnion) thriftObj;
      Option<ThriftStructFieldInfo> fieldInfo = thriftUnion.unionStructFieldInfo();
      if (fieldInfo.isEmpty()) {
        return false;
      } else {
        return fieldInfo.get().tfield().name.equals(fieldName);
      }
    } else {
      FieldMetaData fieldMetaData = metadataMap.get(fieldName);
      if (fieldMetaData.requirementType == TFieldRequirementType.REQUIRED) {
        return true;
      } else if (fieldMetaData.requirementType == TFieldRequirementType.OPTIONAL) {
        int index = indexMap.get(fieldName);
        Product product = (Product) thriftObj;
        Option option = (Option) product.productElement(index);
        return option.isDefined();
      } else {
        return true;
      }
    }
  }

  @SuppressWarnings("unchecked")
  @Override
  public Object getFieldValue(ThriftStruct thriftObj, String fieldName) {
    if (isUnion) {
      ThriftUnion thriftUnion = (ThriftUnion) thriftObj;
      FieldMetaData fieldMetadata = metadataMap.get(fieldName);

      Option<ThriftStructFieldInfo> fieldInfo = thriftUnion.unionStructFieldInfo();
      if (fieldInfo.isEmpty()) {
        return null;
      } else if (!fieldInfo.get().tfield().name.equals(fieldName)) {
        return null;
      }
      Object fieldValue = thriftUnion.containedValue();
      return getFieldValue(fieldMetadata.valueMetaData, fieldValue);

    } else {
      FieldMetaData fieldMetadata = metadataMap.get(fieldName);
      int index = indexMap.get(fieldName);
      Product product = (Product) thriftObj;
      Object fieldValue = product.productElement(index);
      if (fieldMetadata.requirementType == TFieldRequirementType.OPTIONAL) {
        Option option = (Option) fieldValue;
        if (option.isEmpty()) {
          return null;
        }
        fieldValue = option.get();
      }

      return getFieldValue(fieldMetadata.valueMetaData, fieldValue);
    }
  }

  public ThriftStruct newInstance(Map<Object, Object> map) throws Exception {

    TBotMakerMapProtocol protocol = new TBotMakerMapProtocol(metadataMap, map);
    return (ThriftStruct) codec.decode(protocol);
  }

  private Object getListFieldValue(FieldValueMetaData valueMetaData, Object fieldValue) {
    ListMetaData listMetaData = (ListMetaData) valueMetaData;
    FieldValueMetaData elemMetaData = listMetaData.elemMetaData;

    scala.collection.Seq seq = (scala.collection.Seq) fieldValue;
    scala.collection.Iterator it = seq.iterator();
    ImmutableList.Builder<Object> builder = ImmutableList.builder();
    while (it.hasNext()) {
      builder.add(getFieldValue(elemMetaData, it.next()));
    }
    return builder.build();
  }

  private Object getSetFieldValue(FieldValueMetaData valueMetaData, Object fieldValue) {
    SetMetaData setMetaData = (SetMetaData) valueMetaData;
    FieldValueMetaData elemMetaData = setMetaData.elemMetaData;

    scala.collection.Set set = (scala.collection.Set) fieldValue;
    scala.collection.Iterator it = set.iterator();
    ImmutableSet.Builder<Object> builder = ImmutableSet.builder();
    while (it.hasNext()) {
      builder.add(getFieldValue(elemMetaData, it.next()));
    }
    return builder.build();
  }

  @SuppressWarnings("unchecked")
  private Object getMapFieldValue(FieldValueMetaData valueMetaData, Object fieldValue) {

    MapMetaData mapMetaData = (MapMetaData) valueMetaData;
    FieldValueMetaData keyMetaData = mapMetaData.keyMetaData;
    FieldValueMetaData valMetaData = mapMetaData.valueMetaData;

    scala.collection.Map map = (scala.collection.Map) fieldValue;
    scala.collection.Iterator it = map.iterator();

    ImmutableMap.Builder<Object, Object> builder = ImmutableMap.builder();
    while (it.hasNext()) {
      scala.Tuple2 tuple = (scala.Tuple2) it.next();
      builder.put(
          getFieldValue(keyMetaData, tuple._1()),
          getFieldValue(valMetaData, tuple._2())
      );
    }
    return builder.build();
  }

  private Object getFieldValue(FieldValueMetaData valueMetaData, Object fieldValue) {
    switch (valueMetaData.ttype) {
      case TType.STRING:
      case TType.BOOL:
      case TType.DOUBLE:
      case TType.I64:
      case TType.BINARY:
        return fieldValue;

      case TType.BYTE:
      case TType.I16:
      case TType.I32:
        return ((Number) fieldValue).longValue();

      case TType.ENUM:
        ThriftEnum thriftEnum = (ThriftEnum) fieldValue;
        return thriftEnum.name();

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

  private static FieldValueMetaData getFieldValueMetadata(
      Manifest manifest) throws SemanticCheckFailure {
    Class runtimeClass = manifest.runtimeClass();
    if (String.class.equals(runtimeClass)) {
      return new PrimitiveValueMetaData(TType.STRING);
    } else if (boolean.class.equals(runtimeClass)) {
      return new PrimitiveValueMetaData(TType.BOOL);
    } else if (scala.Boolean.class.equals(runtimeClass)) {
      return new PrimitiveValueMetaData(TType.BOOL);
    } else if (byte.class.equals(runtimeClass)) {
      return new PrimitiveValueMetaData(TType.BYTE);
    } else if (scala.Byte.class.equals(runtimeClass)) {
      return new PrimitiveValueMetaData(TType.BYTE);
    } else if (short.class.equals(runtimeClass)) {
      return new PrimitiveValueMetaData(TType.I16);
    } else if (scala.Short.class.equals(runtimeClass)) {
      return new PrimitiveValueMetaData(TType.I16);
    } else if (int.class.equals(runtimeClass)) {
      return new PrimitiveValueMetaData(TType.I32);
    } else if (scala.Int.class.equals(runtimeClass)) {
      return new PrimitiveValueMetaData(TType.I32);
    } else if (long.class.equals(runtimeClass)) {
      return new PrimitiveValueMetaData(TType.I64);
    } else if (scala.Long.class.equals(runtimeClass)) {
      return new PrimitiveValueMetaData(TType.I64);
    } else if (double.class.equals(runtimeClass)) {
      return new PrimitiveValueMetaData(TType.DOUBLE);
    } else if (scala.Double.class.equals(runtimeClass)) {
      return new PrimitiveValueMetaData(TType.I64);
    } else if (ByteBuffer.class.isAssignableFrom(runtimeClass)) {
      return new PrimitiveValueMetaData(TType.BINARY);
    } else if (scala.collection.Seq.class.equals(runtimeClass)) {
      Manifest valueManifest = (Manifest) manifest.typeArguments().apply(0);
      FieldValueMetaData elemMetaData = getFieldValueMetadata(valueManifest);
      return new ListMetaData(elemMetaData);
    } else if (scala.collection.Set.class.equals(runtimeClass)) {
      Manifest valueManifest = (Manifest) manifest.typeArguments().apply(0);
      FieldValueMetaData elemMetaData = getFieldValueMetadata(valueManifest);
      return new SetMetaData(elemMetaData);
    } else if (scala.collection.Map.class.equals(runtimeClass)) {
      Manifest keyManifest = (Manifest) manifest.typeArguments().apply(0);
      Manifest valueManifest = (Manifest) manifest.typeArguments().apply(1);
      FieldValueMetaData keyMetaData = getFieldValueMetadata(keyManifest);
      FieldValueMetaData valueMetaData = getFieldValueMetadata(valueManifest);
      return new MapMetaData(keyMetaData, valueMetaData);
    } else if (ThriftStruct.class.isAssignableFrom(runtimeClass)) {
      return new ThriftStructMetaData(runtimeClass);
    } else if (ThriftEnum.class.isAssignableFrom(runtimeClass)) {
      return new ThriftEnumMetaData((Class<? extends ThriftEnum>) runtimeClass);
    } else {
      throw new SemanticCheckFailure(
          String.format("Unsupported field type %s", runtimeClass.toString()));
    }
  }

  private static ThriftStructCodec getThriftStructCodec(
      Class<? extends ThriftStruct> thriftClass) throws SemanticCheckFailure {

    try {
      Class<? extends ThriftStructCodec> codecClass =
          (Class<? extends ThriftStructCodec>) ClassCache.forName(thriftClass.getName() + "$");
      Field codecField = codecClass.getField("MODULE$");
      ThriftStructCodec codec = (ThriftStructCodec) codecField.get(null);
      return codec;
    } catch (Exception e) {
      throw new SemanticCheckFailure(
          String.format("Cannot load ThriftStruct Codec for %s: $s",
              thriftClass.getName(), e.toString()));
    }
  }

  private static List<ThriftStructFieldInfo> getStructFieldInfos(
      ThriftStructCodec codec) {
    List<ThriftStructFieldInfo> fieldInfos =
        JavaConverters.seqAsJavaList(codec.metaData().fieldInfos());
    return fieldInfos;
  }

  private static List<ThriftStructFieldInfo> getUnionFieldInfos(
      ThriftStructCodec codec) throws SemanticCheckFailure {
    try {
      Method fieldInfosMethod = codec.getClass().getMethod("fieldInfos");
      List<ThriftUnionFieldInfo> fieldInfos =
          JavaConverters.seqAsJavaList(
              (scala.collection.Seq<ThriftUnionFieldInfo>) fieldInfosMethod.invoke(codec));

      ImmutableList.Builder<ThriftStructFieldInfo> builder = ImmutableList.builder();
      for (ThriftUnionFieldInfo fieldInfo : fieldInfos) {
        builder.add(fieldInfo.structFieldInfo());
      }
      return builder.build();
    } catch (Exception e) {
      throw new SemanticCheckFailure(
          String.format("Cannot load ThriftUnion fieldInfos for %s: $s",
              codec.getClass().getName(), e.toString()));
    }
  }

  private static class TBotMakerMapProtocol extends TProtocol {

    public interface Scope {

      FieldValueMetaData getMetaData();

      Object getValue();
    }

    public interface StructScope extends Scope {
      TField readField();
    }

    private static class ThriftStructScope implements StructScope {

      private final ThriftStructType thriftType;
      private final ThriftStruct thriftObject;

      private Iterator<Map.Entry<String, FieldMetaData>> iterator;
      private Map.Entry<String, FieldMetaData> entry;

      public ThriftStructScope(ThriftStructType thriftType, ThriftStruct thriftObject) {
        this.thriftType = thriftType;
        this.thriftObject = thriftObject;

        this.iterator = thriftType.metadataMap.entrySet().iterator();
        this.entry = null;
      }

      public TField readField() {

        while (iterator.hasNext()) {

          entry = iterator.next();
          if (thriftType.isSetValue(thriftObject, entry.getKey())) {
            FieldValueMetaData valueMetaData = entry.getValue().valueMetaData;
            return new TField(
                entry.getKey(),
                valueMetaData.getTType(),
                entry.getValue().fieldId);
          }
        }
        entry = null;
        return new TField(null, TType.STOP, (short) 0);
      }

      public FieldValueMetaData getMetaData() {
        Preconditions.checkNotNull(entry);
        return entry.getValue().valueMetaData;
      }

      public Object getValue() {
        Preconditions.checkNotNull(entry);
        return thriftType.getFieldValue(thriftObject, entry.getKey());
      }
    }

    private static class MapStructScope implements StructScope {

      private final ImmutableMap<String, FieldMetaData> metadataMap;
      private final Map<Object, Object> map;

      private Iterator<Map.Entry<String, FieldMetaData>> iterator;
      private Map.Entry<String, FieldMetaData> entry;

      public MapStructScope(
          ImmutableMap<String, FieldMetaData> metadataMap,
          Map<Object, Object> map
      ) {
        this.metadataMap = metadataMap;
        this.map = map;

        this.iterator = metadataMap.entrySet().iterator();
        this.entry = null;
      }

      public TField readField() {

        while (iterator.hasNext()) {

          entry = iterator.next();
          if (map.containsKey(entry.getKey())) {
            FieldValueMetaData valueMetaData = entry.getValue().valueMetaData;
            return new TField(
                entry.getKey(),
                valueMetaData.getTType(),
                entry.getValue().fieldId);
          }
        }
        entry = null;
        return new TField(null, TType.STOP, (short) 0);
      }

      public FieldValueMetaData getMetaData() {
        Preconditions.checkNotNull(entry);
        return entry.getValue().valueMetaData;
      }

      public Object getValue() {
        Preconditions.checkNotNull(entry);
        return map.get(entry.getKey());
      }
    }

    private static class ListScope implements Scope {

      private final ListMetaData metadata;
      private final List<Object> list;

      private Iterator<Object> iterator;

      public ListScope(ListMetaData metadata, List<Object> list) {
        this.metadata = metadata;
        this.list = list;

        this.iterator = list.iterator();
      }

      public FieldValueMetaData getMetaData() {
        return metadata.elemMetaData;
      }

      public Object getValue() {
        Preconditions.checkArgument(iterator.hasNext());
        return iterator.next();
      }
    }

    private static class SetScope implements Scope {

      private final SetMetaData metadata;
      private final Set<Object> set;

      private Iterator<Object> iterator;

      public SetScope(SetMetaData metadata, Set<Object> set) {
        this.metadata = metadata;
        this.set = set;

        this.iterator = set.iterator();
      }

      public FieldValueMetaData getMetaData() {
        return metadata.elemMetaData;
      }

      public Object getValue() {
        Preconditions.checkArgument(iterator.hasNext());
        return iterator.next();
      }
    }

    private static class MapScope implements Scope {

      private final MapMetaData metadata;
      private final Map<Object, Object> map;

      private Iterator<Map.Entry<Object, Object>> iterator;
      private Map.Entry<Object, Object> entry;
      private boolean isKey;

      public MapScope(MapMetaData metadata, Map<Object, Object> map) {
        this.metadata = metadata;
        this.map = map;

        this.iterator = map.entrySet().iterator();
        this.entry = null;
        this.isKey = false;
      }

      public FieldValueMetaData getMetaData() {
        return !isKey ? metadata.keyMetaData : metadata.valueMetaData;
      }

      public Object getValue() {
        if (isKey) {
          isKey = false;
          return entry.getValue();
        } else {
          Preconditions.checkArgument(iterator.hasNext());
          entry = iterator.next();
          isKey = true;
          return entry.getKey();
        }
      }
    }

    private final ImmutableMap<String, FieldMetaData> metadataMap;
    private final Map<Object, Object> data;

    private Stack<Scope> scopes = new Stack<Scope>();

    public TBotMakerMapProtocol(
        ImmutableMap<String, FieldMetaData> metadataMap,
        Map<Object, Object> data) {
      super(null);

      this.metadataMap = metadataMap;
      this.data = data;
    }

    @Override
    public void writeMessageBegin(TMessage tMessage) throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeMessageEnd() throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeStructBegin(TStruct tStruct) throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeStructEnd() throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeFieldBegin(TField tField) throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeFieldEnd() throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeFieldStop() throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeMapBegin(TMap tMap) throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeMapEnd() throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeListBegin(TList tList) throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeListEnd() throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeSetBegin(TSet tSet) throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeSetEnd() throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeBool(boolean b) throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeByte(byte b) throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeI16(short i) throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeI32(int i) throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeI64(long l) throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeDouble(double v) throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeString(String s) throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void writeBinary(ByteBuffer byteBuffer) throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public TMessage readMessageBegin() throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public void readMessageEnd() throws TException {
      throw new TException(new UnsupportedOperationException());
    }

    @Override
    public TStruct readStructBegin() throws TException {
      if (!scopes.empty()) {
        Scope scope = scopes.peek();
        ThriftStructMetaData structMetadata = (ThriftStructMetaData) scope.getMetaData();
        ThriftStructType thriftType = ofThriftStruct(structMetadata.structClass);
        Object value = readValue();
        if (value instanceof Map) {
          Map<Object, Object> mapValue = (Map<Object, Object>) value;
          scopes.push(new MapStructScope(thriftType.metadataMap, mapValue));
        } else if (value instanceof ThriftStruct) {
          ThriftStruct structValue = (ThriftStruct) value;
          scopes.push(new ThriftStructScope(thriftType, structValue));
        } else {
          throw new TException("unknown struct value");
        }
      } else {
        scopes.push(new MapStructScope(metadataMap, data));
      }

      return null;
    }

    @Override
    public void readStructEnd() {
      scopes.pop();
    }

    @Override
    public TField readFieldBegin() {
      StructScope scope = (StructScope) scopes.peek();
      return scope.readField();
    }

    @Override
    public void readFieldEnd() {

    }

    @Override
    public TMap readMapBegin() {
      Scope scope = scopes.peek();
      MapMetaData mapMetadata = (MapMetaData) scope.getMetaData();
      Map<Object, Object> mapValue = (Map<Object, Object>) readValue();
      scopes.push(new MapScope(mapMetadata, mapValue));

      return new TMap(
          mapMetadata.keyMetaData.getTType(),
          mapMetadata.valueMetaData.getTType(),
          mapValue.size()
      );
    }

    @Override
    public void readMapEnd() {
      scopes.pop();
    }

    @Override
    public TList readListBegin() {
      Scope scope = scopes.peek();
      ListMetaData listMetadata = (ListMetaData) scope.getMetaData();
      List<Object> listValue = (List<Object>) readValue();
      scopes.push(new ListScope(listMetadata, listValue));

      return new TList(
          listMetadata.elemMetaData.getTType(),
          listValue.size()
      );
    }

    @Override
    public void readListEnd() {
      scopes.pop();
    }

    @Override
    public TSet readSetBegin() {
      Scope scope = scopes.peek();
      SetMetaData setMetadata = (SetMetaData) scope.getMetaData();
      Set<Object> setValue = (Set<Object>) readValue();
      scopes.push(new SetScope(setMetadata, setValue));

      return new TSet(
          setMetadata.elemMetaData.getTType(),
          setValue.size()
      );
    }

    @Override
    public void readSetEnd() {
      scopes.pop();
    }

    public Object readValue() {
      Scope scope = scopes.peek();
      return scope.getValue();
    }

    @Override
    public boolean readBool() {
      return (Boolean) readValue();
    }

    @Override
    public byte readByte() {
      return ((Number) readValue()).byteValue();
    }

    @Override
    public short readI16() {
      return ((Number) readValue()).shortValue();
    }

    @Override
    public int readI32() {
      return ((Number) readValue()).intValue();
    }

    @Override
    public long readI64() {
      return ((Number) readValue()).longValue();
    }

    @Override
    public double readDouble() {
      return ((Number) readValue()).doubleValue();
    }

    @Override
    public String readString() {
      return (String) readValue();
    }

    @Override
    public ByteBuffer readBinary() {
      return (ByteBuffer) readValue();
    }
  }
}
