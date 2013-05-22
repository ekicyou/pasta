using System;
using NeoDatis.Tool.Wrappers;
using NeoDatis.Odb.Impl.Core.Oid;
namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>Contains the list for the ODB types</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public sealed class ODBType
	{
		public static int nb;

		private const long serialVersionUID = 341217747918380780L;

		private bool isPrimitive;

		private readonly int id;

		private string name;

		private int size;

		private System.Type superClass;

		/// <summary>Used to instantiate the class when complex subclass is referenced.</summary>
		/// <remarks>
		/// Used to instantiate the class when complex subclass is referenced. example, when a Collection$SynchronizedMap is referenced
		/// ODB, will use HashMap instead
		/// </remarks>
		private System.Type defaultInstanciationClass;

		private long position;

		[System.NonSerialized]
		private static NeoDatis.Odb.Core.Layers.Layer2.Instance.IClassPool classPool = null;

		/// <summary>For array element type</summary>
		private ODBType subType;

		public const int NullId = 0;

		public const int NativeBooleanId = 10;

		/// <summary>1 byte</summary>
		public const int NativeByteId = 20;

		public const int NativeSignedByteId = 21;

		public const int NativeCharId = 30;

		/// <summary>2 byte</summary>
		public const int NativeShortId = 40;

		/// <summary>4 byte</summary>
		public const int NativeIntId = 50;

		/// <summary>8 bytes</summary>
		public const int NativeLongId = 60;

		/// <summary>4 byte</summary>
		public const int NativeFloatId = 70;

		/// <summary>8 byte</summary>
		public const int NativeDoubleId = 80;

		public const int ByteId = 90;

		public const int SignedByteId = 91;

		public const int ShortId = 100;

		public const int IntegerId = 110;

		public const int LongId = 120;

		public const int FloatId = 130;

		public const int DoubleId = 140;

		public const int CharacterId = 150;

		public const int BooleanId = 160;

		public const int DateId = 170;

		public const int DateSqlId = 171;

		public const int DateTimestampId = 172;

		public const int OidId = 180;

		public const int ObjectOidId = 181;

		public const int ClassOidId = 182;

		public const int BigIntegerId = 190;

		public const int BigDecimalId = 200;

		public const int StringId = 210;

		/// <summary>Enums are internally stored as String: the enum name</summary>
		public const int EnumId = 211;

		public const int NativeFixSizeMaxId = ClassOidId;

		public const int NativeMaxId = StringId;

		public const int CollectionId = 250;
        public const int CollectionGenericId = 251;

		public const int ArrayId = 260;

		public const int MapId = 270;

		public const int NonNativeId = 300;

		public static readonly ODBType Null = new ODBType(true, NullId, "null", 1);

		/// <summary>1 byte</summary>
		public static readonly ODBType NativeBoolean = new ODBType(true, NativeBooleanId, OdbClassUtil.GetFullName(typeof(bool)), 1);

		/// <summary>1 byte</summary>
		public static readonly ODBType NativeByte = new ODBType(true, NativeByteId, OdbClassUtil.GetFullName(typeof(byte)), 1);

		/// <summary>2 byte</summary>
		public static readonly ODBType NativeChar = new ODBType(true, NativeCharId, OdbClassUtil.GetFullName(typeof(char)), 2);

		/// <summary>2 byte</summary>
		public static readonly ODBType NativeShort = new ODBType(true, NativeShortId, OdbClassUtil.GetFullName(typeof(short)), 2);

		/// <summary>4 byte</summary>
		public static readonly ODBType NativeInt = new ODBType(true, NativeIntId, OdbClassUtil.GetFullName(typeof(int)), 4);

		/// <summary>8 bytes</summary>
		public static readonly ODBType NativeLong = new ODBType(true, NativeLongId, OdbClassUtil.GetFullName(typeof(long	)), 8);

		/// <summary>4 byte</summary>
		public static readonly ODBType NativeFloat = new ODBType(true, NativeFloatId, OdbClassUtil.GetFullName(typeof(float)), 4);

		/// <summary>8 byte</summary>
		public static readonly ODBType NativeDouble = new ODBType(true, NativeDoubleId, OdbClassUtil.GetFullName(typeof(double)), 8);

		public static readonly ODBType Byte = new ODBType(false, ByteId, OdbClassUtil.GetFullName(typeof(byte)), 1);

		public static readonly ODBType Short = new ODBType(false, ShortId, OdbClassUtil.GetFullName(typeof(short)), 2);

		public static readonly ODBType Integer = new ODBType(false, IntegerId, OdbClassUtil.GetFullName(typeof(int))	, 4);

		public static readonly ODBType BigInteger = new ODBType(false, BigIntegerId, OdbClassUtil.GetFullName(typeof(System.Decimal)), 1);

		public static readonly ODBType Long = new ODBType(false, LongId, OdbClassUtil.GetFullName(typeof(long)), 8);

		public static readonly ODBType Float = new ODBType(false, FloatId, OdbClassUtil.GetFullName(typeof(float)), 4);

		public static readonly ODBType Double = new ODBType(false, DoubleId, OdbClassUtil.GetFullName(typeof(double)), 8);

		public static readonly ODBType BigDecimal = new ODBType(false, BigDecimalId, OdbClassUtil.GetFullName(typeof(System.Decimal)), 1);

		public static readonly ODBType Character = new ODBType(false, CharacterId, OdbClassUtil.GetFullName(typeof(char)), 2);

		public static readonly ODBType Boolean = new ODBType(false, BooleanId, OdbClassUtil.GetFullName(typeof(bool)), 1);

		public static readonly ODBType Date = new ODBType(false, DateId, OdbClassUtil.GetFullName(typeof(System.DateTime)), 8);

		public static readonly ODBType DateSql = new ODBType(false, DateSqlId, OdbClassUtil.GetFullName(typeof(System.DateTime)), 8);

		public static readonly ODBType DateTimestamp = new ODBType(false, DateTimestampId, OdbClassUtil.GetFullName(typeof(System.DateTime)), 8);

		public static readonly ODBType String = new ODBType(false, StringId,OdbClassUtil.GetFullName(typeof(string)), 1);

		public static readonly ODBType Enum = new ODBType(false, EnumId, OdbClassUtil.GetFullName(typeof(System.Enum)), 1);

		public static readonly ODBType Collection = new ODBType(false, CollectionId,OdbClassUtil.GetFullName( typeof(System.Collections.ICollection
			)), 0, typeof(System.Collections.ICollection), typeof(System.Collections.ArrayList));
        public static readonly ODBType CollectionGeneric =
                new ODBType(false, CollectionGenericId, OdbClassUtil.GetFullName(typeof(System.Collections.Generic.ICollection<object>)), 0, typeof(System.Collections.Generic.ICollection<object>), typeof(System.Collections.Generic.List<object>));

		public static readonly ODBType Array = new ODBType(false, ArrayId, "array", 0);

		public static readonly ODBType Map = new ODBType(false, MapId, OdbClassUtil.GetFullName(typeof(System.Collections.IDictionary)), 0, typeof(System.Collections.IDictionary), typeof(System.Collections.Hashtable));

		public static readonly ODBType Oid = new ODBType(false, OidId, OdbClassUtil.GetFullName(typeof(OID)), 0, typeof(NeoDatis.Odb.OID));

		public static readonly ODBType ObjectOid = new ODBType(false, ObjectOidId, OdbClassUtil.GetFullName(typeof(OdbObjectOID)), 0, typeof(OdbObjectOID));

		public static readonly ODBType ClassOid = new ODBType(false, ClassOidId, OdbClassUtil.GetFullName(typeof(OdbClassOID)), 0, typeof(OdbClassOID));

		public static readonly ODBType NonNative = new ODBType(false, NonNativeId, "non native", 0);

		private static readonly System.Collections.Generic.IDictionary<int, ODBType> typesById = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<int, ODBType	>();

		private static readonly System.Collections.Generic.IDictionary<string, ODBType> typesByName = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, ODBType>();

		/// <summary>This cache is used to cache non default types.</summary>
		/// <remarks>This cache is used to cache non default types. Instead or always testing if a class is an array or a collection or any other, we put the odbtype in this cache
		/// 	</remarks>
		private static readonly System.Collections.Generic.IDictionary<string, ODBType
			> cacheOfTypesByName = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, ODBType
			>();

		public static readonly string DefaultCollectionClassName = OdbClassUtil.GetFullName( typeof(System.Collections.ArrayList));

		public static readonly string DefaultMapClassName = OdbClassUtil.GetFullName(typeof(System.Collections.Hashtable));

		public static readonly string DefaultArrayComponentClassName = OdbClassUtil.GetFullName(typeof(object));

		public static readonly int SizeOfInt = ODBType
			.Integer.GetSize();

		public static readonly int SizeOfLong = ODBType
			.Long.GetSize();

		public static readonly int SizeOfBool = ODBType
			.Boolean.GetSize();

		public static readonly int SizeOfByte = ODBType
			.Byte.GetSize();

		static ODBType()
		{
			// Not used in Java, for .Net compatibility
			// Not used in Java, for .Net compatibility
			//public final static ODBType MAP = new ODBType(false,MAP_ID, "java.util.AbstractMap", 0, AbstractMap.class);
			NeoDatis.Tool.Wrappers.List.IOdbList<ODBType
				> allTypes = new NeoDatis.Tool.Wrappers.List.OdbArrayList<ODBType
				>(100);
			//// DO NOT FORGET DO ADD THE TYPE IN THIS LIST WHEN CREATING A NEW ONE!!!
			allTypes.Add(Null);
			allTypes.Add(NativeBoolean);
			allTypes.Add(NativeByte);
			allTypes.Add(NativeChar);
			allTypes.Add(NativeShort);
			allTypes.Add(NativeInt);
			allTypes.Add(NativeLong);
			allTypes.Add(NativeFloat);
			allTypes.Add(NativeDouble);
			allTypes.Add(Byte);
			allTypes.Add(Short);
			allTypes.Add(Integer);
			allTypes.Add(Long);
			allTypes.Add(Float);
			allTypes.Add(Double);
			allTypes.Add(BigDecimal);
			allTypes.Add(BigInteger);
			allTypes.Add(Character);
			allTypes.Add(Boolean);
			allTypes.Add(Date);
			allTypes.Add(DateSql);
			allTypes.Add(DateTimestamp);
			allTypes.Add(String);
			allTypes.Add(Enum);
			allTypes.Add(Collection);
            allTypes.Add(CollectionGeneric);
			allTypes.Add(Array);
			allTypes.Add(Map);
			allTypes.Add(Oid);
			allTypes.Add(ObjectOid);
			allTypes.Add(ClassOid);
			allTypes.Add(NonNative);
			ODBType type = null;
			for (int i = 0; i < allTypes.Count; i++)
			{
				type = allTypes[i];
				typesByName[type.GetName()] = type;
				typesById[type.GetId()] = type;
			}
		}

		protected ODBType(bool isPrimitive, int id, string name, int size)
		{
			this.isPrimitive = isPrimitive;
			this.id = id;
			this.name = name;
			this.size = size;
		}

		protected ODBType(bool isPrimitive, int id, string name, int size, System.Type superclass
			)
		{
			this.isPrimitive = isPrimitive;
			this.id = id;
			this.name = name;
			this.size = size;
			this.superClass = superclass;
		}

		protected ODBType(bool isPrimitive, int id, string name, int size, System.Type superclass
			, System.Type defaultClass) : this(isPrimitive, id, name, size, superclass)
		{
			this.defaultInstanciationClass = defaultClass;
		}

		private void InitClassPool()
		{
			lock (this)
			{
				ODBType.classPool = NeoDatis.Odb.OdbConfiguration
					.GetCoreProvider().GetClassPool();
			}
		}

		public ODBType Copy()
		{
			ODBType newType = new ODBType
				(isPrimitive, id, name, size);
			newType.subType = GetSubType();
			return newType;
		}

		public static ODBType GetFromId(int id)
		{
			ODBType odbType = typesById[id];
			if (odbType == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbTypeIdDoesNotExist
					.AddParameter(id));
			}
			return odbType;
		}

		public static string GetNameFromId(int id)
		{
			return GetFromId(id).GetName();
		}

		public static ODBType GetFromName(string fullName
			)
		{
            ODBType tc = null;

            typesByName.TryGetValue(fullName, out tc);
			if (tc != null)
			{
				return tc;
			}
			ODBType nonNative = new ODBType
				(ODBType.NonNative.isPrimitive, NonNativeId
				, fullName, 0);
			return nonNative;
		}

		public static ODBType GetFromClass(System.Type clazz)
		{
            string className = OdbClassUtil.GetFullName(clazz);
			if (NeoDatis.Tool.Wrappers.OdbClassUtil.IsEnum(clazz))
			{
				ODBType type = new ODBType(ODBType.Enum.isPrimitive, ODBType.EnumId, ODBType.Enum.GetName(), 0);
				type.SetName(OdbClassUtil.GetFullName(clazz));
				return type;
			}
			// First check if it is a 'default type'
            ODBType tc = null;

            typesByName.TryGetValue(className, out tc);
			if (tc != null)
			{
				return tc;
			}
			// Then check if it is a 'non default type'
			cacheOfTypesByName.TryGetValue(className,out tc);
			if (tc != null)
			{
				return tc;
			}
			if (IsArray(clazz))
			{
				ODBType type = new ODBType(ODBType.Array.isPrimitive, ODBType.ArrayId, ODBType.Array.GetName(), 0);
				type.subType = GetFromClass(clazz.GetElementType());
				cacheOfTypesByName.Add(className, type);
				return type;
			}
			if (IsMap(clazz))
			{
				cacheOfTypesByName.Add(className, Map);
				return Map;
			}
			// check if it is a list
			if (IsCollection(clazz))
			{
				cacheOfTypesByName.Add(className, Collection);
				return Collection;
			}
			nb++;
            ODBType nonNative = new ODBType(ODBType.NonNative.isPrimitive, NonNativeId	, OdbClassUtil.GetFullName(clazz), 0);
			cacheOfTypesByName.Add(className, nonNative);
			return nonNative;
		}

		public static bool IsArray(System.Type clazz)
		{
			return clazz.IsArray;
		}

		public static bool IsMap(System.Type clazz)
		{
            bool isNonGenericMap = Map.superClass.IsAssignableFrom(clazz);

            if (isNonGenericMap)
            {
                return true;
            }
            Type[] types = clazz.GetInterfaces();
            for (int i = 0; i < types.Length; i++)
            {
                //Console.WriteLine(types[i].ToString()+ " / "  + types[i].FullName);
                int ind = types[i].FullName.IndexOf("System.Collections.Generic.IDictionary");
                if (ind != -1)
                {
                    return true;
                }


            }
            //return Collection.superClass.IsAssignableFrom(clazz) || CollectionGeneric.superClass.IsAssignableFrom(clazz);
            return false;
        }

		public static bool IsCollection(System.Type clazz)
		{
            bool isNonGenericCollection = Collection.superClass.IsAssignableFrom(clazz);
            if (isNonGenericCollection)
            {
                return true;
            }
            Type[] types = clazz.GetInterfaces();
            for (int i = 0; i < types.Length; i++)
            {
                //Console.WriteLine(types[i].ToString()+ " / "  + types[i].FullName);
                int ind = types[i].FullName.IndexOf("System.Collections.Generic.ICollection");
                if ( ind != -1)
                {
                    return true;
                }
   

            }
            //return Collection.superClass.IsAssignableFrom(clazz) || CollectionGeneric.superClass.IsAssignableFrom(clazz);
            return false;
		}

		public static bool IsNative(System.Type clazz)
		{
            ODBType tc = null;
            
            typesByName.TryGetValue(OdbClassUtil.GetFullName(clazz),out tc);
			if (tc != null)
			{
				return true;
			}
			if (clazz.IsArray)
			{
				//ODBType type = new ODBType(ODBType.ARRAY.isPrimitive,ODBType.ARRAY_ID,ODBType.ARRAY.getName(),0);
				//type.subType = getFromClass(clazz.getComponentType());
				return true;
			}
			if (Map.superClass.IsAssignableFrom(clazz))
			{
				return true;
			}
			// check if it is a list
			if (Collection.superClass.IsAssignableFrom(clazz))
			{
				return true;
			}
			return false;
		}

		public static bool Exist(string name)
		{
			return typesByName.ContainsKey(name);
		}

		public int GetId()
		{
			return id;
		}

		public string GetName()
		{
			return name;
		}

		public void SetName(string name)
		{
			this.name = name;
		}

		public int GetSize()
		{
			return size;
		}

		public bool IsCollection()
		{
			return id == CollectionId;
		}

		public static bool IsCollection(int odbTypeId)
		{
			return odbTypeId == CollectionId;
		}

		public bool IsArray()
		{
			return id == ArrayId;
		}

		public static bool IsArray(int odbTypeId)
		{
			return odbTypeId == ArrayId;
		}

		public bool IsMap()
		{
			return id == MapId;
		}

		public static bool IsMap(int odbTypeId)
		{
			return odbTypeId == MapId;
		}

		public bool IsArrayOrCollection()
		{
			return IsArray() || IsCollection();
		}

		public static bool IsNative(int odbtypeId)
		{
			return odbtypeId != NonNativeId;
		}

		public bool IsNative()
		{
			return id != NonNativeId;
		}

		public ODBType GetSubType()
		{
			return subType;
		}

		public System.Type GetSuperClass()
		{
			return superClass;
		}

		public void SetSuperClass(System.Type superClass)
		{
			this.superClass = superClass;
		}

		public override string ToString()
		{
			return id + " - " + name;
		}

		public void SetSubType(ODBType subType)
		{
			this.subType = subType;
		}

		public override bool Equals(object obj)
		{
			if (obj == null || obj.GetType() != typeof(ODBType
				))
			{
				return false;
			}
			ODBType type = (ODBType
				)obj;
			return GetId() == type.GetId();
		}

		public System.Type GetNativeClass()
		{
			switch (id)
			{
				case NativeBooleanId:
				{
					return typeof(bool);
				}

				case NativeByteId:
				{
					return typeof(byte);
				}

				case NativeCharId:
				{
					return typeof(char);
				}

				case NativeDoubleId:
				{
					return typeof(double);
				}

				case NativeFloatId:
				{
					return typeof(float);
				}

				case NativeIntId:
				{
					return typeof(int);
				}

				case NativeLongId:
				{
					return typeof(long);
				}

				case NativeShortId:
				{
					return typeof(short);
				}

				case ObjectOidId:
				{
					return typeof(NeoDatis.Odb.Impl.Core.Oid.OdbObjectOID);
				}

				case ClassOidId:
				{
					return typeof(NeoDatis.Odb.Impl.Core.Oid.OdbClassOID);
				}

				case OidId:
				{
					return typeof(NeoDatis.Odb.OID);
				}
			}
			if (classPool == null)
			{
				InitClassPool();
			}
			return classPool.GetClass(GetName());
		}

		public bool IsNonNative()
		{
			return id == NonNativeId;
		}

		public static bool IsNonNative(int odbtypeId)
		{
			return odbtypeId == NonNativeId;
		}

		public bool IsNull()
		{
			return id == NullId;
		}

		public static bool IsNull(int odbTypeId)
		{
			return odbTypeId == NullId;
		}

		public bool HasFixSize()
		{
			return HasFixSize(id);
		}

		public static bool HasFixSize(int odbId)
		{
			return odbId > 0 && odbId <= NativeFixSizeMaxId;
		}

		//return odbId != BIG_INTEGER_ID && odbId != BIG_DECIMAL_ID && odbId != STRING_ID && odbId != COLLECTION_ID && odbId!=ARRAY_ID && odbId!= MAP_ID && odbId!=NON_NATIVE_ID;
		public bool IsStringOrBigDicemalOrBigInteger()
		{
			return IsStringOrBigDicemalOrBigInteger(id);
		}

		public static bool IsStringOrBigDicemalOrBigInteger(int odbTypeId)
		{
			return odbTypeId == StringId || odbTypeId == BigDecimalId || odbTypeId == BigIntegerId;
		}

		public static bool IsAtomicNative(int odbTypeId)
		{
			return (odbTypeId > 0 && odbTypeId <= NativeMaxId);
		}

		public bool IsAtomicNative()
		{
			return IsAtomicNative(id);
		}

		public static bool IsEnum(int odbTypeId)
		{
			return odbTypeId == EnumId;
		}

		public bool IsEnum()
		{
			return IsEnum(id);
		}

		public static bool IsPrimitive(int odbTypeId)
		{
			return ODBType.GetFromId(odbTypeId).isPrimitive;
		}

		public static bool TypesAreCompatible(ODBType
			 type1, ODBType type2)
		{
			if (type1.IsArray() && type2.IsArray())
			{
				return TypesAreCompatible(type1.GetSubType(), type2.GetSubType());
			}
			if (type1.GetName().Equals(type2.GetName()))
			{

				return true;
			}
			if (type1.IsNative() && type2.IsNative())
			{
				if (type1.IsEquivalent(type2))
				{
					return true;
				}
				return false;
			}
			if (type1.IsNonNative() && type2.IsNonNative())
			{
				return (type1.GetNativeClass() == type2.GetNativeClass()) || (type1.GetNativeClass
					().IsAssignableFrom(type2.GetNativeClass()));
			}
			return false;
		}

		public bool IsBoolean()
		{
			return id == BooleanId || id == NativeBooleanId;
		}

		private bool IsEquivalent(ODBType type2)
		{
			return (id == IntegerId && type2.id == NativeIntId) || (type2.id == IntegerId && 
				id == NativeIntId);
		}

		public System.Type GetDefaultInstanciationClass()
		{
			return defaultInstanciationClass;
		}

		public void Init2()
		{
		}

		// TODO Auto-generated method stub
		/// <returns></returns>
		public bool IsDate()
		{
			return id == DateId || id == DateSqlId || id == DateTimestampId;
		}
	}
}
