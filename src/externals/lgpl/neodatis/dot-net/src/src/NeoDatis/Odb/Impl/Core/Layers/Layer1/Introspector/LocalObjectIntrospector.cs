using NeoDatis.Tool.Wrappers;
using NeoDatis.Odb.Core.Layers.Layer2.Meta;
using NeoDatis.Odb.Core;
using System;
using System.Collections.Generic;
using NeoDatis.Odb.Core.Layers.Layer1.Introspector;
using NeoDatis.Tool.Wrappers.Map;
using NeoDatis.Odb.Core.Transaction;
namespace NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector
{
	/// <summary>The local implementation of the Object Instrospector.</summary>
	/// <remarks>The local implementation of the Object Instrospector.</remarks>
	/// <author>osmadja</author>
	public class LocalObjectIntrospector : NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector
	{
		protected NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine;

		protected NeoDatis.Odb.Core.Layers.Layer1.Introspector.IClassIntrospector classIntrospector;

		private NeoDatis.Odb.Core.Layers.Layer2.Instance.IClassPool classPool;

		public LocalObjectIntrospector(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine
			)
		{
			// private MetaModel localMetaModel;
			this.storageEngine = storageEngine;
			this.classIntrospector = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClassIntrospector
				();
			this.classPool = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClassPool();
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo GetMetaRepresentation
			(object o, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci, bool recursive
			, System.Collections.Generic.IDictionary<object, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			> alreadyReadObjects, NeoDatis.Odb.Core.Layers.Layer1.Introspector.IIntrospectionCallback
			 callback)
		{
			return GetObjectInfo(o, ci, recursive, alreadyReadObjects, callback);
		}

		/// <summary>retrieve object data</summary>
		/// <param name="o"></param>
		/// <param name="ci"></param>
		/// <param name="recursive"></param>
		/// <returns>The object info</returns>
		protected virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo GetObjectInfo
			(object o, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci, bool recursive
			, System.Collections.Generic.IDictionary<object, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			> alreadyReadObjects, NeoDatis.Odb.Core.Layers.Layer1.Introspector.IIntrospectionCallback
			 callback)
		{
			return GetObjectInfoInternal(null, o, ci, recursive, alreadyReadObjects, callback
				);
		}

		protected virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo GetNativeObjectInfoInternal
			(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType type, object o, bool recursive
			, System.Collections.Generic.IDictionary<object, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			> alreadyReadObjects, NeoDatis.Odb.Core.Layers.Layer1.Introspector.IIntrospectionCallback
			 callback)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi = null;
			if (type.IsAtomicNative())
			{
				if (o == null)
				{
					aoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NullNativeObjectInfo(type.GetId());
				}
				else
				{
					aoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo(o, type
						.GetId());
				}
			}
			else
			{
				if (type.IsCollection())
				{
					aoi = IntrospectCollection((System.Collections.ICollection)o, recursive, alreadyReadObjects
						, type, callback);
				}
				else
				{
					if (type.IsArray())
					{
						if (o == null)
						{
							aoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.ArrayObjectInfo(null);
						}
						else
						{
							// Gets the type of the elements of the array
							string realArrayClassName = OdbClassUtil.GetFullName(o.GetType().GetElementType());
							NeoDatis.Odb.Core.Layers.Layer2.Meta.ArrayObjectInfo aroi = null;
							if (recursive)
							{
								aroi = IntrospectArray(o, recursive, alreadyReadObjects, type, callback);
							}
							else
							{
								aroi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.ArrayObjectInfo((object[])o
									);
							}
							aroi.SetRealArrayComponentClassName(realArrayClassName);
							aoi = aroi;
						}
					}
					else
					{
						if (type.IsMap())
						{
							if (o == null)
							{
								aoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.MapObjectInfo(null, type, type.GetDefaultInstanciationClass
									().FullName);
							}
							else
							{
								MapObjectInfo moi = null;
								string realMapClassName = OdbClassUtil.GetFullName(o.GetType());
                                bool isGeneric = o.GetType().IsGenericType;
                                if (isGeneric)
                                {
                                    moi = new MapObjectInfo(IntrospectGenericMap((System.Collections.Generic.IDictionary<object,object>)o, recursive, alreadyReadObjects, callback), type, realMapClassName);
                                }
                                else
                                {
                                    moi = new MapObjectInfo(IntrospectNonGenericMap((System.Collections.IDictionary)o, recursive, alreadyReadObjects, callback), type, realMapClassName);
                                }
								if (realMapClassName.IndexOf("$") != -1)
								{
									moi.SetRealMapClassName(OdbClassUtil.GetFullName(type.GetDefaultInstanciationClass()));
								}
								aoi = moi;
							}
						}
						else
						{
							if (type.IsEnum())
							{
								System.Enum enumObject = (System.Enum)o;
								if (enumObject == null)
								{
									aoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NullNativeObjectInfo(type.GetSize(
										));
								}
								else
								{
                                    Type t = enumObject.GetType();
                                    string enumClassName = enumObject == null ? null : OdbClassUtil.GetFullName(enumObject.GetType());
									// Here we must check if the enum is already in the meta model. Enum must be stored in the meta
									// model to optimize its storing as we need to keep track of the enum class
									// for each enum stored. So instead of storing the enum class name, we can store enum class id, a long
									// instead of the full enum class name string
									NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = GetClassInfo(enumClassName);
									string enumValue = enumObject == null ? null : enumObject.ToString();
									aoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.EnumNativeObjectInfo(ci, enumValue
										);
								}
							}
						}
					}
				}
			}
			return aoi;
		}

		/// <summary>
		/// Build a meta representation of an object
		/// <pre>
		/// warning: When an object has two fields with the same name (a private field with the same name in a parent class, the deeper field (of the parent) is ignored!)
		/// </pre>
		/// </summary>
		/// <param name="o"></param>
		/// <param name="ci"></param>
		/// <param name="recursive"></param>
		/// <returns>The ObjectInfo</returns>
		protected virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo GetObjectInfoInternal
			(NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo nnoi, object o, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 ci, bool recursive, System.Collections.Generic.IDictionary<object, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			> alreadyReadObjects, NeoDatis.Odb.Core.Layers.Layer1.Introspector.IIntrospectionCallback
			 callback)
		{
			object value = null;
			if (o == null)
			{
				return NeoDatis.Odb.Core.Layers.Layer2.Meta.NullNativeObjectInfo.GetInstance();
			}
			System.Type clazz = o.GetType();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType type = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(clazz);
			string className = OdbClassUtil.GetFullName(clazz);
			if (type.IsNative())
			{
				return GetNativeObjectInfoInternal(type, o, recursive, alreadyReadObjects, 
					callback);
			}
			// sometimes the clazz.getName() may not match the ci.getClassName()
			// It happens when the attribute is an interface or superclass of the
			// real attribute class
			// In this case, ci must be updated to the real class info
			if (ci != null && !clazz.FullName.Equals(ci.GetFullClassName()))
			{
				ci = GetClassInfo(className);
				nnoi = null;
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo mainAoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)nnoi;
			bool isRootObject = false;
			if (alreadyReadObjects == null)
			{
				alreadyReadObjects = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<object, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
					>();
				isRootObject = true;
			}
			if (o != null)
			{
                NonNativeObjectInfo cachedNnoi = null;
                alreadyReadObjects.TryGetValue(o, out cachedNnoi);
				
                if (cachedNnoi != null)
				{
					ObjectReference or = new ObjectReference(cachedNnoi);
					return or;
				}
				if (callback != null)
				{
					callback.ObjectFound(o);
				}
			}
			if (mainAoi == null)
			{
				mainAoi = BuildNnoi(o, ci, null, null, null, alreadyReadObjects);
			}
			alreadyReadObjects[o] = mainAoi;
			NeoDatis.Tool.Wrappers.List.IOdbList<System.Reflection.FieldInfo> fields = classIntrospector.GetAllFields(className);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi = null;
			int attributeId = -1;
			// For all fields
			for (int i = 0; i < fields.Count; i++)
			{
				System.Reflection.FieldInfo field = fields[i];
				try
				{
					value = field.GetValue(o);
					attributeId = ci.GetAttributeId(field.Name);
					if (attributeId == -1)
					{
						throw new ODBRuntimeException(NeoDatisError.ObjectIntrospectorNoFieldWithName.AddParameter(ci.GetFullClassName()).AddParameter(field.Name));
					}
					ODBType valueType = null;
					if (value == null)
					{
						// If value is null, take the type from the field type
						// declared in the class
						valueType = ODBType.GetFromClass(field.FieldType);
					}
					else
					{
						// Else take the real attribute type!
						valueType = ODBType.GetFromClass(value.GetType());
					}
					// for native fields
					if (valueType.IsNative())
					{
						aoi = GetNativeObjectInfoInternal(valueType, value, recursive, alreadyReadObjects, callback);
						mainAoi.SetAttributeValue(attributeId, aoi);
					}
					else
					{
						//callback.objectFound(value);
						// Non Native Objects
						if (value == null)
						{
                            ClassInfo clai = GetClassInfo(OdbClassUtil.GetFullName(field.GetType()));

							aoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeNullObjectInfo(clai);
							mainAoi.SetAttributeValue(attributeId, aoi);
						}
						else
						{
                            ClassInfo clai = GetClassInfo(OdbClassUtil.GetFullName(value.GetType()));
							if (recursive)
							{
								aoi = GetObjectInfoInternal(null, value, clai, recursive, alreadyReadObjects, callback
									);
								mainAoi.SetAttributeValue(attributeId, aoi);
							}
							else
							{
								// When it is not recursive, simply add the object
								// values.add(value);
								throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
									.AddParameter("Should not enter here - ObjectIntrospector - 'simply add the object'"
									));
							}
						}
					}
				}
				catch (System.ArgumentException e)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
						.AddParameter("in getObjectInfoInternal"), e);
				}
				catch (System.MemberAccessException e)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
						.AddParameter("getObjectInfoInternal"), e);
				}
			}
			if (isRootObject)
			{
				alreadyReadObjects.Clear();
				alreadyReadObjects = null;
			}
			return mainAoi;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo BuildNnoi
			(object o, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			[] values, long[] attributesIdentification, int[] attributeIds, System.Collections.Generic.IDictionary
			<object, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo> alreadyReadObjects
			)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				(o, classInfo, values, attributesIdentification, attributeIds);
			if (storageEngine != null)
			{
				// for unit test purpose
				NeoDatis.Odb.Core.Transaction.ICache cache = storageEngine.GetSession(true).GetCache
					();
				// Check if object is in the cache, if so sets its oid
				NeoDatis.Odb.OID oid = cache.GetOid(o, false);
				if (oid != null)
				{
					nnoi.SetOid(oid);
					// Sets some values to the new header to keep track of the infos
					// when
					// executing NeoDatis without closing it, just committing.
					// Bug reported by Andy
					NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = cache.GetObjectInfoHeaderFromOid
						(oid, true);
					nnoi.GetHeader().SetObjectVersion(oih.GetObjectVersion());
					nnoi.GetHeader().SetUpdateDate(oih.GetUpdateDate());
					nnoi.GetHeader().SetCreationDate(oih.GetCreationDate());
				}
			}
			return nnoi;
		}

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo IntrospectCollection
			(System.Collections.ICollection collection, bool introspect, System.Collections.Generic.IDictionary
			<object, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo> alreadyReadObjects
			, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType type, NeoDatis.Odb.Core.Layers.Layer1.Introspector.IIntrospectionCallback
			 callback)
		{
			if (collection == null)
			{
				return new NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo();
			}
			// A collection that contain all meta representations of the collection
			// objects
			System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
				> collectionCopy = new System.Collections.Generic.List<NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
				>(collection.Count);
			// A collection to keep references all all non native objects of the
			// collection
			// This will be used later to get all non native objects contained in an
			// object
			System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				> nonNativesObjects = new System.Collections.Generic.List<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				>(collection.Count);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi = null;
			System.Collections.IEnumerator iterator = collection.GetEnumerator();
			while (iterator.MoveNext())
			{
				object o = iterator.Current;
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = null;
				// Null objects are not inserted in list
				if (o != null)
				{
					ci = GetClassInfo(OdbClassUtil.GetFullName(o.GetType()));
					aoi = GetObjectInfo(o, ci, introspect, alreadyReadObjects, callback);
					collectionCopy.Add(aoi);
					if (aoi.IsNonNativeObject())
					{
						// o is not null, call the callback with it
						//callback.objectFound(o);
						// This is a non native object
						nonNativesObjects.Add((NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)aoi
							);
					}
				}
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo coi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo
				(collectionCopy, nonNativesObjects);
			string realCollectionClassName = OdbClassUtil.GetFullName(collection.GetType());
			if (realCollectionClassName.IndexOf("$") != -1)
			{
				coi.SetRealCollectionClassName(type.GetDefaultInstanciationClass().FullName);
			}
			else
			{
				coi.SetRealCollectionClassName(realCollectionClassName);
			}
			return coi;
		}

		private System.Collections.Generic.IDictionary<AbstractObjectInfo, AbstractObjectInfo> IntrospectNonGenericMap(
            System.Collections.IDictionary map, 
            bool introspect, 
            IDictionary<object, NonNativeObjectInfo> alreadyReadObjects, IIntrospectionCallback callback)
		{
			System.Collections.Generic.IDictionary<AbstractObjectInfo, AbstractObjectInfo> mapCopy = new OdbHashMap<AbstractObjectInfo,AbstractObjectInfo>();
			System.Collections.ICollection keySet = map.Keys;
			System.Collections.IEnumerator keys = keySet.GetEnumerator();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ciKey = null;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ciValue = null;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoiForKey = null;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoiForValue = null;
			while (keys.MoveNext())
			{
				object key = keys.Current;
				object value = map[key];
				if (key != null)
				{
					ciKey = GetClassInfo(OdbClassUtil.GetFullName(key.GetType()));
					if (value != null)
					{
						ciValue = GetClassInfo(OdbClassUtil.GetFullName(value.GetType()));
					}
					aoiForKey = GetObjectInfo(key, ciKey, introspect, alreadyReadObjects, callback);
					aoiForValue = GetObjectInfo(value, ciValue, introspect, alreadyReadObjects, callback
						);
					mapCopy.Add(aoiForKey, aoiForValue);
				}
			}
			return mapCopy;
		}
        private System.Collections.Generic.IDictionary<AbstractObjectInfo, AbstractObjectInfo> IntrospectGenericMap(
            System.Collections.Generic.IDictionary<object,object> map,
            bool introspect,
            IDictionary<object, NonNativeObjectInfo> alreadyReadObjects, IIntrospectionCallback callback)
        {
            System.Collections.Generic.IDictionary<AbstractObjectInfo, AbstractObjectInfo> mapCopy = new OdbHashMap<AbstractObjectInfo, AbstractObjectInfo>();
            System.Collections.Generic.ICollection<object> keySet = map.Keys;
            System.Collections.IEnumerator keys = keySet.GetEnumerator();
            NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ciKey = null;
            NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ciValue = null;
            NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoiForKey = null;
            NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoiForValue = null;
            while (keys.MoveNext())
            {
                object key = keys.Current;
                object value = map[key];
                if (key != null)
                {
                    ciKey = GetClassInfo(OdbClassUtil.GetFullName(key.GetType()));
                    if (value != null)
                    {
                        ciValue = GetClassInfo(OdbClassUtil.GetFullName(value.GetType()));
                    }
                    aoiForKey = GetObjectInfo(key, ciKey, introspect, alreadyReadObjects, callback);
                    aoiForValue = GetObjectInfo(value, ciValue, introspect, alreadyReadObjects, callback);
                    mapCopy.Add(aoiForKey, aoiForValue);
                }
            }
            return mapCopy;
        }

		private ClassInfo GetClassInfo(string fullClassName
			)
		{
			if (ODBType.GetFromName(fullClassName).IsNative	())
			{
				return null;
			}
			ISession session = storageEngine.GetSession(true);
			MetaModel metaModel = session.GetMetaModel();
			if (metaModel.ExistClass(fullClassName))
			{
				return metaModel.GetClassInfo(fullClassName, true);
			}
			ClassInfo ci = null;
			ClassInfoList ciList = null;
			ciList = classIntrospector.Introspect(fullClassName, true);
			// to enable junit tests
			if (storageEngine != null)
			{
				storageEngine.AddClasses(ciList);
				// For client Server : reset meta model
				if (!storageEngine.IsLocal())
				{
					metaModel = session.GetMetaModel();
				}
			}
			else
			{
				metaModel.AddClasses(ciList);
			}
			ci = metaModel.GetClassInfo(fullClassName, true);
			return ci;
		}

		/// <summary>Used when byte code instrumentation is to check if an object has changed
		/// 	</summary>
		/// <param name="o"></param>
		/// <returns></returns>
		public virtual bool ObjectHasChanged(object o)
		{
			System.Type clazz = classPool.GetClass(o.GetType().FullName);
			System.Reflection.FieldInfo field;
			try
			{
				field = classIntrospector.GetField(clazz, "hasChanged");
				object value = field.GetValue(o);
				return ((bool)value);
			}
			catch (System.Exception e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
					.AddParameter("in objectHasChanged(Object object)"), e);
			}
		}

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.ArrayObjectInfo IntrospectArray(object
			 array, bool introspect, System.Collections.Generic.IDictionary<object, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			> alreadyReadObjects, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType valueType, NeoDatis.Odb.Core.Layers.Layer1.Introspector.IIntrospectionCallback
			 callback)
		{
			int length = NeoDatis.Tool.Wrappers.OdbReflection.GetArrayLength(array);
			System.Type elementType = array.GetType().GetElementType();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType type = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(elementType);
			if (type.IsAtomicNative())
			{
				return IntropectAtomicNativeArray(array, type);
			}
			if (!introspect)
			{
				return new NeoDatis.Odb.Core.Layers.Layer2.Meta.ArrayObjectInfo((object[])array);
			}
			object[] arrayCopy = new object[length];
			for (int i = 0; i < length; i++)
			{
				object o = NeoDatis.Tool.Wrappers.OdbReflection.GetArrayElement(array, i);
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = null;
				if (o != null)
				{
					ci = GetClassInfo(OdbClassUtil.GetFullName(o.GetType()));
					NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi = GetObjectInfo(o, ci
						, introspect, alreadyReadObjects, callback);
					arrayCopy[i] = aoi;
				}
				else
				{
					arrayCopy[i] = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeNullObjectInfo();
				}
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ArrayObjectInfo arrayOfAoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.ArrayObjectInfo
				(arrayCopy, valueType, type.GetId());
			return arrayOfAoi;
		}

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.ArrayObjectInfo IntropectAtomicNativeArray
			(object array, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType type)
		{
			int length = NeoDatis.Tool.Wrappers.OdbReflection.GetArrayLength(array);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo anoi = null;
			object[] arrayCopy = new object[length];
			int typeId = 0;
			for (int i = 0; i < length; i++)
			{
				object o = NeoDatis.Tool.Wrappers.OdbReflection.GetArrayElement(array, i);
				if (o != null)
				{
					// If object is not null, try to get the exact type
					typeId = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.GetFromClass(o.GetType()).GetId
						();
					anoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo(o, typeId);
					arrayCopy[i] = anoi;
				}
				else
				{
					// Else take the declared type
					arrayCopy[i] = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NullNativeObjectInfo(type
						.GetId());
				}
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ArrayObjectInfo aoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.ArrayObjectInfo
				(arrayCopy, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Array, type.GetId());
			return aoi;
		}

		public virtual void Clear()
		{
			storageEngine = null;
		}
	}
}
