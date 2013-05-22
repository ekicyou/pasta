
using NeoDatis.Tool.Wrappers.List;
using NeoDatis.Odb.Core.Trigger;
using NeoDatis.Odb.Core.Layers.Layer3;
using NeoDatis.Odb.Core.Layers.Layer1.Introspector;
using NeoDatis.Odb.Core.Layers.Layer2.Instance;
using NeoDatis.Odb;
using NeoDatis.Odb.Core.Transaction;
using NeoDatis.Odb.Impl.Core.Transaction;
using NeoDatis.Odb.Core.Layers.Layer2.Meta;
using NeoDatis.Odb.Core;
using System.Reflection;
using System;
using System.Collections;
using NeoDatis.Tool;
using NeoDatis.Odb.Core.Oid;
using NeoDatis.Tool.Wrappers;

namespace NeoDatis.Odb.Impl.Core.Layers.Layer2.Instance
{
	/// <summary>Class used to build instance from Meta Object representation.</summary>
	/// <remarks>
	/// Class used to build instance from Meta Object representation. Layer 2 to
	/// Layer 1 conversion.
	/// </remarks>
	/// <author>osmadja</author>
	public abstract class InstanceBuilder : NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder
	{
		private static readonly string LogId = "InstanceBuilder";

		private static readonly string LogIdDebug = "InstanceBuilder.debug";

		private ITriggerManager triggerManager;

		protected IStorageEngine engine;

		private IClassIntrospector classIntrospector;

		private IClassPool classPool;

		public InstanceBuilder(IStorageEngine engine)
		{
			this.triggerManager = OdbConfiguration.GetCoreProvider().GetLocalTriggerManager(engine);
			this.classIntrospector = OdbConfiguration.GetCoreProvider().GetClassIntrospector();
			this.classPool = OdbConfiguration.GetCoreProvider().GetClassPool();
			this.engine = engine;
		}

		/// <summary>Local and server InstanceBuilder must define their own getSession()</summary>
		protected abstract ISession GetSession();

		public virtual object BuildOneInstance(AbstractObjectInfo objectInfo)
		{
			object o = null;
			if (objectInfo is NonNativeNullObjectInfo)
			{
				return null;
			}
			
			if (objectInfo.GetType() == typeof(NonNativeObjectInfo))
			{
				o = BuildOneInstance((NonNativeObjectInfo)objectInfo);
			}
			else
			{
				// instantiation cache is not used for native objects
				o = BuildOneInstance((NativeObjectInfo)objectInfo);
			}
			return o;
		}
        public virtual object BuildCollectionInstance(CollectionObjectInfo coi)
		{
            Type t = classPool.GetClass(coi.GetRealCollectionClassName());

            if(t.IsGenericType){
                return BuildGenericCollectionInstance(coi,t);
            }else{
                return BuildNonGenericCollectionInstance(coi,t);
            }
           
		}
		public virtual object BuildGenericCollectionInstance(CollectionObjectInfo coi, Type t)
		{
            Type genericType = t.GetGenericTypeDefinition();
            object newCollection = null;
            try
            {
                newCollection = System.Activator.CreateInstance(t);
            }
            catch (System.Exception e)
            {
                Console.WriteLine(e);
                throw new ODBRuntimeException(NeoDatisError.CollectionInstanciationError.AddParameter(coi.GetRealCollectionClassName()));
            }
            System.Collections.IEnumerator iterator = coi.GetCollection().GetEnumerator();
            AbstractObjectInfo aoi = null;
            MethodInfo method = t.GetMethod("Add");
            while (iterator.MoveNext())
            {
                aoi = (AbstractObjectInfo)iterator.Current;
                if (!aoi.IsDeletedObject())
                {
                    method.Invoke(newCollection, new object[] { BuildOneInstance(aoi) });
                }
            }
            return newCollection;
           
		}
        public virtual object BuildNonGenericCollectionInstance(CollectionObjectInfo coi, Type t)
		{
            System.Collections.IList newCollection = (System.Collections.IList)System.Activator.CreateInstance(classPool.GetClass(coi.GetRealCollectionClassName()));
            System.Collections.IEnumerator iterator = coi.GetCollection().GetEnumerator();
            AbstractObjectInfo aoi = null;
            
            while (iterator.MoveNext())
            {
                aoi = (AbstractObjectInfo)iterator.Current;
                if (!aoi.IsDeletedObject())
                {
                    newCollection.Add(BuildOneInstance(aoi));
                }
            }
            return newCollection;
		}

		/// <summary>Builds an instance of an enum</summary>
		/// <param name="enumClass"></param>
		public virtual object BuildEnumInstance(EnumNativeObjectInfo enoi, System.Type enumClass)
		{
            object theEnum = System.Enum.Parse(enumClass, enoi.GetEnumName());
			
			return theEnum;
		}

		/// <summary>Builds an instance of an array</summary>
		public virtual object BuildArrayInstance(ArrayObjectInfo aoi)
		{
			// first check if array element type is native (int,short, for example)
			ODBType type = ODBType.GetFromName(aoi.GetRealArrayComponentClassName());
			
			System.Type arrayClazz = type.GetNativeClass();
			object array = System.Array.CreateInstance(arrayClazz, aoi.GetArray().Length);
			
			object o = null;
			AbstractObjectInfo aboi = null;
			for (int i = 0; i < aoi.GetArrayLength(); i++)
			{
				aboi = (AbstractObjectInfo)aoi.GetArray()[i];
				if (aboi != null && !aboi.IsDeletedObject() && !aboi.IsNull())
				{
					o = BuildOneInstance(aboi);
					((Array)array).SetValue(o,i);
				}
			}
			return array;
		}

        public virtual object BuildMapInstance(MapObjectInfo moi)
        {
            Type t = classPool.GetClass(moi.GetRealMapClassName());

            if (t.IsGenericType)
            {
                return BuildGenericMapInstance(moi, t);
            }
            else
            {
                return BuildNonGenericMapInstance(moi,t);
            }

        }
		public virtual IDictionary BuildNonGenericMapInstance(MapObjectInfo mapObjectInfo, Type t)
		{
			System.Collections.Generic.IDictionary<AbstractObjectInfo,AbstractObjectInfo> map = mapObjectInfo.GetMap();
			IDictionary newMap;
			try
			{
				newMap = (IDictionary) System.Activator.CreateInstance(t);
			}
			catch (System.Exception e)
			{
				throw new ODBRuntimeException(NeoDatisError.MapInstanciationError.AddParameter(map.GetType().FullName),e);
			}
			int i = 0;
			System.Collections.Generic.IEnumerator<AbstractObjectInfo> iterator = map.Keys.GetEnumerator();
			AbstractObjectInfo key = null;
			while (iterator.MoveNext())
			{
				key = iterator.Current;
				object realKey = BuildOneInstance(key);
				object realValue = BuildOneInstance(map[key]);
                newMap[realKey] = realValue;
			}
			return newMap;
		}
        public virtual object BuildGenericMapInstance(MapObjectInfo mapObjectInfo, Type t)
        {
            System.Collections.Generic.IDictionary<AbstractObjectInfo, AbstractObjectInfo> map = mapObjectInfo.GetMap();
            Type genericType = t.GetGenericTypeDefinition();
            object newMap = null;
            try
            {
                newMap = System.Activator.CreateInstance(t);
            }
            catch (System.Exception e)
            {
                Console.WriteLine(e);
                throw new ODBRuntimeException(NeoDatisError.MapInstanciationError.AddParameter(map.GetType().FullName), e);
            }
           
            int i = 0;
            System.Collections.Generic.IEnumerator<AbstractObjectInfo> iterator = map.Keys.GetEnumerator();
            AbstractObjectInfo key = null;
            MethodInfo method = t.GetMethod("Add", t.GetGenericArguments());
            while (iterator.MoveNext())
            {
                key = iterator.Current;
                object realKey = BuildOneInstance(key);
                object realValue = BuildOneInstance(map[key]);
                method.Invoke(newMap, new object[] { realKey, realValue });
            }
            return newMap;
        }

		public virtual object BuildOneInstance(NonNativeObjectInfo objectInfo)
		{
			ICache cache = GetSession().GetCache();
			
			// verify if the object is check to delete
			if (objectInfo.IsDeletedObject())
			{
				throw new ODBRuntimeException(NeoDatisError.ObjectIsMarkedAsDeletedForOid.AddParameter(objectInfo.GetOid()));
			}
			// Then check if object is in cache
			object o = cache.GetObjectWithOid(objectInfo.GetOid());
			if (o != null)
			{
				return o;
			}
			Type instanceClazz = null;
			instanceClazz = classPool.GetClass(objectInfo.GetClassInfo().GetFullClassName());
			try
			{
				o = classIntrospector.NewInstanceOf(instanceClazz);
			}
			catch (System.Exception e)
			{
				throw new ODBRuntimeException(NeoDatisError.InstanciationError.AddParameter(objectInfo.GetClassInfo().GetFullClassName()), e);
			}
			// This can happen if ODB can not create the instance
			// TODO Check if returning null is correct
			if (o == null)
			{
				return null;
			}
			// Keep the initial hash code. In some cases, when the class redefines
			// the hash code method
			// Hash code can return wrong values when attributes are not set (when
			// hash code depends on attribute values)
			// Hash codes are used as the key of the map,
			// So at the end of this method, if hash codes are different, object
			// will be removed from the cache and inserted back
			bool hashCodeIsOk = true;
			int initialHashCode = 0;
			try
			{
				initialHashCode = o.GetHashCode();
			}
			catch (System.Exception)
			{
				hashCodeIsOk = false;
			}
			// Adds this incomplete instance in the cache to manage cyclic reference
			if (hashCodeIsOk)
			{
				cache.AddObject(objectInfo.GetOid(), o, objectInfo.GetHeader());
			}
			ClassInfo ci = objectInfo.GetClassInfo();
			IOdbList<FieldInfo> fields = classIntrospector.GetAllFields(ci.GetFullClassName());
			
			FieldInfo field = null;
			AbstractObjectInfo aoi = null;
			object value = null;
			
			for (int i = 0; i < fields.Count; i++)
			{
				field = fields[i];
				// Gets the id of this field
				int attributeId = ci.GetAttributeId(field.Name);
				if (OdbConfiguration.IsDebugEnabled(LogIdDebug))
				{
					DLogger.Debug("getting field with name " + field.Name + ", attribute id is "+ attributeId);
				}
				aoi = objectInfo.GetAttributeValueFromId(attributeId);
				// Check consistency
				// ensureClassCompatibily(field,
				// instanceInfo.getClassInfo().getAttributeinfo(i).getFullClassname());
				if (aoi != null && (!aoi.IsNull()))
				{
					if (aoi.IsNative())
					{
						if (aoi.IsAtomicNativeObject())
						{
							if (aoi.IsNull())
							{
								value = null;
							}
							else
							{
								value = aoi.GetObject();
							}
						}
						if (aoi.IsCollectionObject())
						{
							value = BuildCollectionInstance((CollectionObjectInfo)aoi);
							// Manage a specific case of Set
							/*
							if (typeof(Java.Util.Set).IsAssignableFrom(field.GetType()) && typeof(ICollection).IsAssignableFrom(value.GetType()))
							{
								Java.Util.Set s = new Java.Util.HashSet();
								s.AddAll((System.Collections.ICollection)value);
								value = s;
							}*/
						}
						if (aoi.IsArrayObject())
						{
							value = BuildArrayInstance((ArrayObjectInfo)aoi);
						}
						if (aoi.IsMapObject())
						{
							value = BuildMapInstance((MapObjectInfo)aoi);
						}
						if (aoi.IsEnumObject())
						{
							value = BuildEnumInstance((EnumNativeObjectInfo)aoi, field.FieldType);
						}
					}
					else
					{
						if (aoi.IsNonNativeObject())
						{
							if (aoi.IsDeletedObject())
							{
								if (NeoDatis.Odb.OdbConfiguration.DisplayWarnings())
								{
									IError warning = NeoDatisError.AttributeReferencesADeletedObject
										.AddParameter(objectInfo.GetClassInfo().GetFullClassName())
										.AddParameter(objectInfo.GetOid()).AddParameter(field.Name);
									DLogger.Info(warning.ToString());
								}
								value = null;
							}
							else
							{
								value = BuildOneInstance((NonNativeObjectInfo)aoi);
							}
						}
					}
					if (value != null)
					{
						if (OdbConfiguration.IsDebugEnabled(LogIdDebug))
						{
							DLogger.Debug("Setting field " + field.Name + "(" + field.GetType().FullName + ") to " + value + " / " + value.GetType().FullName);
						}
						try
						{
							field.SetValue(o, value);
						}
						catch (System.Exception e)
						{
							throw new ODBRuntimeException(NeoDatisError.InstanceBuilderWrongObjectContainerType
								.AddParameter(objectInfo.GetClassInfo().GetFullClassName())
								.AddParameter(value.GetType().FullName).AddParameter(field.GetType().FullName), e);
						}
					}
				}
			}
			if (o != null && !OdbClassUtil.GetFullName(o.GetType()).Equals(objectInfo.GetClassInfo().GetFullClassName()))
			{
				new ODBRuntimeException(NeoDatisError.InstanceBuilderWrongObjectType
						.AddParameter(objectInfo.GetClassInfo().GetFullClassName())
						.AddParameter(o.GetType().FullName));
			}
			if (hashCodeIsOk || initialHashCode != o.GetHashCode())
			{
				// Bug (sf bug id=1875544 )detected by glsender
				// This can happen when an object has redefined its own hashcode
				// method and depends on the field values
				// Then, we have to remove object from the cache and re-insert to
				// correct map hash code
				cache.RemoveObjectWithOid(objectInfo.GetOid());
				// re-Adds instance in the cache
				cache.AddObject(objectInfo.GetOid(), o, objectInfo.GetHeader());
			}
			if (triggerManager != null)
			{
				triggerManager.ManageSelectTriggerAfter(objectInfo.GetClassInfo().GetFullClassName
					(), objectInfo, objectInfo.GetOid());
			}
			if (OdbConfiguration.ReconnectObjectsToSession()) {

			   ICrossSessionCache crossSessionCache = CacheFactory.GetCrossSessionCache(engine.GetBaseIdentification().GetIdentification());
			   crossSessionCache.AddObject(o, objectInfo.GetOid());

		   }
			
			return o;
		}

		public virtual object BuildOneInstance(NativeObjectInfo objectInfo)
		{
			if (objectInfo.IsAtomicNativeObject())
			{
				return BuildOneInstance((AtomicNativeObjectInfo)objectInfo);
			}
			if (objectInfo.IsCollectionObject())
			{
				CollectionObjectInfo coi = (CollectionObjectInfo)objectInfo;
				object value = BuildCollectionInstance(coi);
				/* Manage a specific case of Set
				if (typeof(Java.Util.Set).IsAssignableFrom(value.GetType()) && typeof(System.Collections.ICollection).IsAssignableFrom(value.GetType()))
				{
					Java.Util.Set s = new Java.Util.HashSet();
					s.AddAll((System.Collections.ICollection)value);
					value = s;
				}
				*/
				return value;
			}
			if (objectInfo.IsArrayObject())
			{
				return BuildArrayInstance((ArrayObjectInfo)objectInfo);
			}
			if (objectInfo.IsMapObject())
			{
				return BuildMapInstance((MapObjectInfo)objectInfo);
			}
			if (objectInfo.IsNull())
			{
				return null;
			}
			throw new ODBRuntimeException(NeoDatisError.InstanceBuilderNativeType.AddParameter(ODBType.GetNameFromId(objectInfo.GetOdbTypeId())));
		}

		public virtual object BuildOneInstance(AtomicNativeObjectInfo objectInfo)
		{
			int odbTypeId = objectInfo.GetOdbTypeId();
			long l = -1;
			switch (odbTypeId)
			{
				case ODBType.NullId:
				{
					return null;
				}

				case ODBType.StringId:
				{
					return objectInfo.GetObject();
				}

				case ODBType.DateId:
				{
					return objectInfo.GetObject();
				}

				case ODBType.LongId:
				case ODBType.NativeLongId:
				{
					if (objectInfo.GetObject().GetType() == typeof(System.Int64))
					{
						return objectInfo.GetObject();
					}
					return System.Convert.ToInt64(objectInfo.GetObject().ToString());
				}

				case ODBType.IntegerId:
				case ODBType.NativeIntId:
				{
					if (objectInfo.GetObject().GetType() == typeof(int))
					{
						return objectInfo.GetObject();
					}
					return System.Convert.ToInt32(objectInfo.GetObject().ToString());
				}

				case ODBType.BooleanId:
				case ODBType.NativeBooleanId:
				{
					if (objectInfo.GetObject().GetType() == typeof(bool))
					{
						return objectInfo.GetObject();
					}
					return System.Convert.ToBoolean(objectInfo.GetObject().ToString());
				}

				case ODBType.ByteId:
				case ODBType.NativeByteId:
				{
					if (objectInfo.GetObject().GetType() == typeof(byte))
					{
						return objectInfo.GetObject();
					}
					return System.Convert.ToByte(objectInfo.GetObject().ToString());
				}

				case ODBType.ShortId:
				case ODBType.NativeShortId:
				{
					if (objectInfo.GetObject().GetType() == typeof(short))
					{
						return objectInfo.GetObject();
					}
					return System.Convert.ToInt16(objectInfo.GetObject().ToString());
				}

				case ODBType.FloatId:
				case ODBType.NativeFloatId:
				{
					if (objectInfo.GetObject().GetType() == typeof(float))
					{
						return objectInfo.GetObject();
					}
					return System.Convert.ToSingle(objectInfo.GetObject().ToString());
				}

				case ODBType.DoubleId:
				case ODBType.NativeDoubleId:
				{
					if (objectInfo.GetObject().GetType() == typeof(double))
					{
						return objectInfo.GetObject();
					}
					return System.Convert.ToDouble(objectInfo.GetObject().ToString());
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.BigDecimalId:
				{
					return System.Decimal.Parse(objectInfo.GetObject().ToString(), System.Globalization.NumberStyles.Any);
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.BigIntegerId:
				{
					return System.Decimal.Parse(objectInfo.GetObject().ToString(), System.Globalization.NumberStyles.Any);
				}

				case ODBType.CharacterId:
				case ODBType.NativeCharId:
				{
					if (objectInfo.GetObject().GetType() == typeof(char))
					{
						return objectInfo.GetObject();
					}
					return objectInfo.GetObject().ToString()[0];
				}

				case ODBType.ObjectOidId:
				{
					if (objectInfo.GetObject().GetType() == typeof(long))
					{
						l = (long)objectInfo.GetObject();
					}
					else
					{
						OID oid = (OID)objectInfo.GetObject();
						l = oid.GetObjectId();
					}
					return OIDFactory.BuildObjectOID(l);
				}

				case ODBType.ClassOidId:
				{
					if (objectInfo.GetObject().GetType() == typeof(long))
					{
						l = (long)objectInfo.GetObject();
					}
					else
					{
						l = System.Convert.ToInt64(objectInfo.GetObject().ToString());
					}
					return OIDFactory.BuildClassOID(l);
				}

				default:
				{
					throw new ODBRuntimeException(NeoDatisError.InstanceBuilderNativeTypeInCollectionNotSupported
							.AddParameter(ODBType.GetNameFromId(odbTypeId)));
					break;
				}
			}
		}

		public virtual string GetSessionId()
		{
			return engine.GetSession(true).GetId();
		}

		public virtual bool IsLocal()
		{
			return engine.IsLocal();
		}
	}
}
