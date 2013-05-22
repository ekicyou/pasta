	using System;
	using NeoDatis.Odb.Core.Layers.Layer2.Meta;
	using NeoDatis.Tool.Wrappers.List;
	using System.Reflection;
    using NeoDatis.Tool.Wrappers;
    using NeoDatis.Odb.Impl.Core.Layers.Layer2.Instance;
    using NeoDatis.Tool;
	using NeoDatis.Odb.Core;


using System.Collections.Generic;
using NeoDatis.Tool.Wrappers.Map;
using NeoDatis.Odb.Core.Layers.Layer2.Instance;
using NeoDatis.Odb.Impl.Core.Oid;
using NeoDatis.Btree;
using NeoDatis.Btree.Impl;
using NeoDatis.Odb.Impl.Core.Btree;

		namespace NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector
		{
		
		/// <summary>The ClassIntrospector is used to introspect classes.</summary>
		/// <remarks>
		/// The ClassIntrospector is used to introspect classes. It uses Reflection to
		/// extract class information. It transforms a native Class into a ClassInfo (a
		/// meta representation of the class) that contains all informations about the
		/// class.
		/// </remarks>
		/// <author>osmadja</author>
		public abstract class AbstractClassIntrospector : NeoDatis.Odb.Core.Layers.Layer1.Introspector.IClassIntrospector
		{
			private static readonly string LogId = "ClassIntrospector";
	
			private IDictionary<string,IOdbList<FieldInfo>> fields = new OdbHashMap<string,IOdbList<FieldInfo>>();

            private IDictionary<string,Type> systemClasses = new OdbHashMap<string, Type>();

            private IDictionary<String, FullInstantiationHelper> fullInstantiationHelpers = new OdbHashMap<String, FullInstantiationHelper>();
            private IDictionary<String, InstantiationHelper> instantiationHelpers = new OdbHashMap<String, InstantiationHelper>();
            private IDictionary<String, ParameterHelper> parameterHelpers = new OdbHashMap<String, ParameterHelper>();
	
			private NeoDatis.Odb.Core.Layers.Layer2.Instance.IClassPool classPool;


			/// <summary> </summary>
				/// <param name="clazz">The class to instrospect
				/// </param>
				/// <param name="recursive">If true, goes does the hierarchy to try to analyse all classes
				/// </param>
				/// <param name="The">list of class info detected while introspecting the class
				/// </param>
				/// <returns>
				/// </returns>
				public ClassInfoList Introspect(System.Type clazz, bool recursive)
				{
					return InternalIntrospect(clazz, recursive, null);
				}
								 			
			public virtual void AddInstanciationHelper(System.Type clazz, NeoDatis.Odb.Core.Layers.Layer2.Instance.InstantiationHelper
				 helper)
		{
		AddInstantiationHelper(clazz.FullName, helper);
			}
	
			public virtual void AddParameterHelper(System.Type clazz, NeoDatis.Odb.Core.Layers.Layer2.Instance.ParameterHelper
				 helper)
			{
				AddParameterHelper(clazz.FullName, helper);
			}
	
			public virtual void AddFullInstanciationHelper(System.Type clazz, NeoDatis.Odb.Core.Layers.Layer2.Instance.FullInstantiationHelper
				 helper)
			{
				AddFullInstantiationHelper(clazz.FullName, helper);
			}
	
			public virtual void AddInstantiationHelper(string clazz, NeoDatis.Odb.Core.Layers.Layer2.Instance.InstantiationHelper
				 helper)
			{
				instantiationHelpers.Add(clazz, helper);
			}
	
			public virtual void AddParameterHelper(string clazz, NeoDatis.Odb.Core.Layers.Layer2.Instance.ParameterHelper
				 helper)
			{
				parameterHelpers.Add(clazz, helper);
			}
	
			public virtual void AddFullInstantiationHelper(string clazz, NeoDatis.Odb.Core.Layers.Layer2.Instance.FullInstantiationHelper
				helper)
			{
				fullInstantiationHelpers.Add(clazz, helper);
			}
				
				public virtual void RemoveInstantiationHelper(System.Type clazz)
			{
				RemoveInstantiationHelper(clazz.FullName);
			}
	
			public virtual void RemoveInstantiationHelper(string canonicalName)
			{
				instantiationHelpers.Remove(canonicalName);
			}
	
			public virtual void RemoveParameterHelper(System.Type clazz)
			{
				RemoveParameterHelper(clazz.FullName);
			}
	
			public virtual void RemoveParameterHelper(string canonicalName)
			{
				parameterHelpers.Remove(canonicalName);
			}
	
			public virtual void RemoveFullInstantiationHelper(System.Type clazz)
			{
				RemoveFullInstantiationHelper(clazz.FullName);
			}
	
			public virtual void RemoveFullInstantiationHelper(string canonicalName)
			{
				fullInstantiationHelpers.Remove(canonicalName);
			}

				
				/// <summary> </summary>
				/// <param name="clazz">The class to instrospect
				/// </param>
				/// <param name="recursive">If true, goes does the hierarchy to try to analyse all classes
				/// </param>
				/// <param name="A">map with classname that are being introspected, to avoid recursive calls
				/// 
				/// </param>
				/// <returns>
				/// </returns>
				private ClassInfoList InternalIntrospect(System.Type clazz, bool recursive, ClassInfoList classInfoList)
				{
					if (classInfoList != null)
					{
						ClassInfo existingCi = (ClassInfo) classInfoList.GetClassInfoWithName(OdbClassUtil.GetFullName( clazz));
						if (existingCi != null)
						{
							return classInfoList;
						}
					}
					
		            ClassInfo classInfo = new ClassInfo(OdbClassUtil.GetFullName(clazz));
		            classInfo.SetClassCategory( GetClassCategory(OdbClassUtil.GetFullName(clazz)) );
					if (classInfoList == null)
					{
						classInfoList = new ClassInfoList(classInfo);
					}
					else
					{
						classInfoList.AddClassInfo(classInfo);
					}
					
					// Field[] fields = clazz.getDeclaredFields();
					//UPGRADE_TODO: The equivalent in .NET for method 'java.lang.Class.getName' may return a different value. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1043'"
					//m by cristi
		            IOdbList<FieldInfo> fields = GetAllFields(OdbClassUtil.GetFullName(clazz));
					IOdbList<ClassAttributeInfo> attributes = new OdbArrayList<ClassAttributeInfo>(fields.Count);
					
					ClassInfo ci = null;
					for (int i = 0; i < fields.Count; i++)
					{
						System.Reflection.FieldInfo field = (System.Reflection.FieldInfo) fields[i];
                        //Console.WriteLine("Field " + field.Name + " , type = " + field.FieldType);
						if (!ODBType.GetFromClass(field.FieldType).IsNative())
						{
							if (recursive)
							{
								classInfoList = InternalIntrospect(field.FieldType, recursive, classInfoList);
		                        ci = classInfoList.GetClassInfoWithName(OdbClassUtil.GetFullName(field.FieldType));
							}
							else
							{
		                        ci = new ClassInfo(OdbClassUtil.GetFullName(field.FieldType));
							}
						}
						else
						{
							ci = null;
						}
						attributes.Add(new ClassAttributeInfo((i + 1), field.Name, field.FieldType, OdbClassUtil.GetFullName(field.FieldType), ci));
					}
					classInfo.SetAttributes( attributes );
					classInfo.SetMaxAttributeId( fields.Count);
					return classInfoList;
				}
				
				/// <summary>Builds a class info from a class and an existing class info
				/// 
				/// <pre>
				/// The existing class info is used to make sure that fields with the same name will have
				/// the same id
				/// </pre>
				/// 
				/// </summary>
				/// <param name="fullClassName">The name of the class to get info
				/// </param>
				/// <param name="existingClassInfo">
				/// </param>
				/// <returns> A ClassInfo -  a meta representation of the class
				/// </returns>
				public ClassInfo GetClassInfo(System.String fullClassName, ClassInfo existingClassInfo)
				{
					
					
					ClassInfo classInfo = new ClassInfo(fullClassName);
					classInfo.SetClassCategory( GetClassCategory(fullClassName) );
		;
					IOdbList<FieldInfo> fields = GetAllFields(fullClassName);
					IOdbList<ClassAttributeInfo> attributes = new OdbArrayList<ClassAttributeInfo>(fields.Count);
			
					int attributeId = - 1;
					int maxAttributeId = existingClassInfo.GetMaxAttributeId();
					ClassInfo ci = null;
					for (int i = 0; i < fields.Count; i++)
					{
						FieldInfo field = fields[i];
						// Gets the attribute id from the existing class info
						attributeId = existingClassInfo.GetAttributeId(field.Name);
						if (attributeId == - 1)
						{
							maxAttributeId++;
							// The attibute with field.getName() does not exist in existing class info
							//  create a new id
							attributeId = maxAttributeId;
						}
						if (!ODBType.GetFromClass(field.FieldType).IsNative())
						{
							ci = new ClassInfo(OdbClassUtil.GetFullName(field.FieldType));
						}
						else
						{
							ci = null;
						}
						
						attributes.Add(new ClassAttributeInfo(attributeId, field.Name, field.FieldType, OdbClassUtil.GetFullName(field.FieldType), ci));
					}
					classInfo.SetAttributes( attributes );
					classInfo.SetMaxAttributeId( maxAttributeId);
					return classInfo;
				}
				
				/// <summary> </summary>
				/// <param name="fullClassName">
				/// </param>
				/// <param name="includingThis">
				/// </param>
				/// <returns> The list of super classes
				/// </returns>
				public System.Collections.IList GetSuperClasses( string fullClassName, bool includingThis)
				{
					System.Collections.IList result = new System.Collections.ArrayList();
					
					System.Type clazz = classPool.GetClass(fullClassName);
					
					if (clazz.IsInterface)
					{
						//throw new ODBRuntimeException(clazz.getName() + " is an interface");
					}
					if (includingThis)
					{
						result.Add(clazz);
					}
					
					System.Type superClass = clazz.BaseType;
					while (superClass != null && superClass != typeof(System.Object))
					{
						result.Add(superClass);
						superClass = superClass.BaseType;
					}
					return result;
				}
				
			
			public FieldInfo GetField(Type type, string fieldName)
		{
			return type.GetField(fieldName);
		}

            public IOdbList<FieldInfo> GetAllFields(string fullClassName)
            {
                IOdbList<FieldInfo> result = null;
                fields.TryGetValue(fullClassName, out result);

                if (result != null)
                {
                    return result;
                }
                System.Collections.IDictionary attributesNames = new System.Collections.Hashtable();
                result = new OdbArrayList<FieldInfo>(50);
                System.Reflection.FieldInfo[] superClassfields = null;
                System.Collections.IList classes = GetSuperClasses(fullClassName, true);
                for (int i = 0; i < classes.Count; i++)
                {
                    System.Type clazz1 = (System.Type)classes[i];

                    superClassfields = clazz1.GetFields(System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.DeclaredOnly | System.Reflection.BindingFlags.Static);
                    for (int j = 0; j < superClassfields.Length; j++)
                    {
                        // Only adds the attribute if it does not exist one with same name
                        if (attributesNames[superClassfields[j].Name] == null)
                        {
                            result.Add(superClassfields[j]);
                            attributesNames[superClassfields[j].Name] = superClassfields[j].Name;
                        }
                    }
                }
                result = RemoveUnnecessaryFields(result);
                fields[fullClassName] = result;
                attributesNames.Clear();
                attributesNames = null;
                return result;
            }
				
				public IOdbList<FieldInfo> RemoveUnnecessaryFields(IOdbList<FieldInfo> fields)
				{
					IOdbList<FieldInfo> fieldsToRemove = new OdbArrayList<FieldInfo> (fields.Count);
					
					// Remove static fields
					for (int i = 0; i < fields.Count; i++)
					{
						System.Reflection.FieldInfo field = (System.Reflection.FieldInfo) fields[i];
		
		                // by osmadja
		                if (field.IsNotSerialized || field.IsStatic )
		                {
		                    fieldsToRemove.Add(field);
		                }
		                //by cristi
		                if (field.FieldType == typeof(IntPtr))
		                {
		                    fieldsToRemove.Add(field);
		                }
		                object[] oattr = field.GetCustomAttributes(true);
		                bool isNonPersistent = false;
		                foreach (object at in oattr)
		                {
		                    NonPersistentAttribute npat = at as NonPersistentAttribute;
		                    if (npat != null)
		                    {
		                        isNonPersistent = true;
		                        break;
		                    }
		                }
		                if (isNonPersistent || field.IsStatic)
		                {
		                    fieldsToRemove.Add(field);
		                }
						// Remove inner class fields
						if (field.Name.StartsWith("this$"))
						{
							fieldsToRemove.Add(field);
						}
					}
					
					fields.RemoveAll(fieldsToRemove);
					return fields;
				}
				
	/// <summary>
		/// introspect a list of classes
		/// This method return the current meta model based on the classes that currently exist in the
		/// execution classpath.
		/// </summary>
		/// <remarks>
		/// introspect a list of classes
		/// This method return the current meta model based on the classes that currently exist in the
		/// execution classpath. The result will be used to check meta model compatiblity between
		/// the meta model that is currently persisted in the database and the meta model
		/// currently executing in JVM. This is used b the automatic meta model refactoring
		/// </remarks>
		/// <returns></returns>
		/// <returns>A map where the key is the class name and the key is the ClassInfo: the class meta representation
		/// 	</returns>
		public virtual IDictionary<string, ClassInfo> Instrospect(IOdbList<ClassInfo> classInfos)
		{
			ClassInfo persistedCI = null;
			ClassInfo currentCI = null;
			IDictionary<string, ClassInfo> cis = new Dictionary<string, ClassInfo>();
			// re introspect classes
			IEnumerator<ClassInfo> iterator = classInfos.GetEnumerator();
			while (iterator.MoveNext())
			{
				persistedCI = iterator.Current;
				currentCI = GetClassInfo(persistedCI.GetFullClassName(), persistedCI);
				cis.Add(currentCI.GetFullClassName(), currentCI);
			}
			return cis;
		}
				
				public ClassInfoList Introspect(System.String fullClassName, bool recursive)
				{
					return Introspect(classPool.GetClass(fullClassName), true);
				}
				
				public System.Reflection.ConstructorInfo GetConstructorOf(System.String fullClassName)
				{
					System.Type clazz = classPool.GetClass(fullClassName);
					try
					{
						// Checks if exist a default constructor - with no parameters
						System.Reflection.ConstructorInfo constructor = clazz.GetConstructor(new System.Type[0]);
						return constructor;
					}
					catch (System.MethodAccessException e)
					{
						// else take the constructer with the smaller number of parameters
						// and call it will null values
						// @TODO Put this inf oin cache !
						if (OdbConfiguration.IsDebugEnabled(LogId))
						{
							DLogger.Debug(clazz + " does not have default constructor! using a 'with parameter' constructor will null values");
						}
						System.Reflection.ConstructorInfo[] constructors = clazz.GetConstructors();
						int numberOfParameters = 1000;
						int bestConstructorIndex = 0;
						for (int i = 0; i < constructors.Length; i++)
						{
							//UPGRADE_TODO: The equivalent in .NET for method 'java.lang.reflect.Constructor.getParameterTypes' may return a different value. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1043'"
							if (constructors[i].GetParameters().Length < numberOfParameters)
							{
								bestConstructorIndex = i;
							}
						}
						System.Reflection.ConstructorInfo constructor = constructors[bestConstructorIndex];
						return constructor;
					}
				}

	public virtual void Reset()
		{
			fields.Clear();
			fullInstantiationHelpers.Clear();
			instantiationHelpers.Clear();
			parameterHelpers.Clear();
		}

		/// <summary>Two phase init method</summary>
		public virtual void Init2()
		{
			this.classPool = OdbConfiguration.GetCoreProvider().GetClassPool();
		}

public virtual object NewFullInstanceOf(System.Type clazz, NonNativeObjectInfo
			 nnoi)
		{
			string className = clazz.FullName;
			NeoDatis.Odb.Core.Layers.Layer2.Instance.FullInstantiationHelper helper = (NeoDatis.Odb.Core.Layers.Layer2.Instance.FullInstantiationHelper
				)fullInstantiationHelpers[className];
			if (helper != null)
			{
				object o = helper.Instantiate(nnoi);
				if (o != null)
				{
					return o;
				}
			}
			return null;
		}

				public System.Object NewInstanceOf(System.Type clazz)
		        {
		            
		                System.Reflection.ConstructorInfo constructor = null;
		                constructor = classPool.GetConstrutor(OdbClassUtil.GetFullName(clazz));
		                if (constructor == null)
		                {
		                    // Checks if exist a default constructor - with no parameters
		                    constructor = clazz.GetConstructor(Type.EmptyTypes);
		                    //UPGRADE_ISSUE: Method 'java.lang.reflect.AccessibleObject.setAccessible' was not converted. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1000_javalangreflectAccessibleObject'"
		                    //c by cristi
		                    //constructor.setAccessible(true);
		                    if(constructor!=null)
		                    {
		                    	classPool.AddConstrutor(OdbClassUtil.GetFullName( clazz), constructor);
		                    }
		                }
		                if (constructor != null)
		                {
		                    System.Object o = constructor.Invoke(new System.Object[0]);
		                    return o;
		                }
		
		                if (clazz.IsValueType)
		                {
		                    return Activator.CreateInstance(clazz);
		                }
		                else
		                {
		                    // else take the constructer with the smaller number of parameters
		                    // and call it will null values
		                    // @TODO Put this info in cache !
		                    if (OdbConfiguration.IsDebugEnabled(LogId))
		                    {
		                        DLogger.Debug(clazz + " does not have default constructor! using a 'with parameter' constructor will null values");
		                    }
		                    System.Reflection.ConstructorInfo[] constructors = clazz.GetConstructors(System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.DeclaredOnly);
		
		                    if (clazz.IsInterface)
		                    {
		                        //@TODO This is not a good solution to manage interface
		                        return null;
		                    }
		
		                    if (constructors.Length == 0)
		                    {
		                            throw new ODBRuntimeException(NeoDatisError.ClassWithoutConstructor.AddParameter(clazz.AssemblyQualifiedName));
		                    }
		                    int numberOfParameters = 1000;
		                    int bestConstructorIndex = 0;
		                    for (int i = 0; i < constructors.Length; i++)
		                    {
		                        //UPGRADE_TODO: The equivalent in .NET for method 'java.lang.reflect.Constructor.getParameterTypes' may return a different value. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1043'"
		                        if (constructors[i].GetParameters().Length < numberOfParameters)
		                        {
		                            bestConstructorIndex = i;
		                        }
		                    }
		                    constructor = constructors[bestConstructorIndex];
		                    //UPGRADE_TODO: The equivalent in .NET for method 'java.lang.reflect.Constructor.getParameterTypes' may return a different value. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1043'"
		                    System.Object[] nulls = new System.Object[constructor.GetParameters().Length];
		                    for (int i = 0; i < nulls.Length; i++)
		                    {
		                        //UPGRADE_TODO: The equivalent in .NET for method 'java.lang.reflect.Constructor.getParameterTypes' may return a different value. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1043'"
		                        //m by cristi
		                        if (constructor.GetParameters()[i].ParameterType == System.Type.GetType("System.Int32"))
		                        {
		                            nulls[i] = 0;
		                        }
		                        else
		                        {
		                            //UPGRADE_TODO: The equivalent in .NET for method 'java.lang.reflect.Constructor.getParameterTypes' may return a different value. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1043'"
		                            if (constructor.GetParameters()[i].ParameterType == System.Type.GetType("System.Int64"))
		                            {
		                                nulls[i] = 0;
		                            }
		                            else
		                            {
		                                //UPGRADE_TODO: The equivalent in .NET for method 'java.lang.reflect.Constructor.getParameterTypes' may return a different value. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1043'"
		                                if (constructor.GetParameters()[i].ParameterType == System.Type.GetType("System.Int16"))
		                                {
		                                    nulls[i] = System.Int16.Parse("0");
		                                }
		                                else
		                                {
		                                    //UPGRADE_TODO: The equivalent in .NET for method 'java.lang.reflect.Constructor.getParameterTypes' may return a different value. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1043'"
		                                    //main by cristi
		                                    if (constructor.GetParameters()[i].ParameterType == System.Type.GetType("System.SByte"))
		                                    {
		                                        nulls[i] = System.SByte.Parse("0");
		                                    }
		                                    else
		                                    {
		                                        //UPGRADE_TODO: The equivalent in .NET for method 'java.lang.reflect.Constructor.getParameterTypes' may return a different value. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1043'"
		                                        //m by cristi
		                                        if (constructor.GetParameters()[i].ParameterType == System.Type.GetType("System.Single"))
		                                        {
		                                            //UPGRADE_WARNING: Data types in Visual C# might be different.  Verify the accuracy of narrowing conversions. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1042'"
		                                            nulls[i] = System.Single.Parse("0");
		                                        }
		                                        else
		                                        {
		                                            //UPGRADE_TODO: The equivalent in .NET for method 'java.lang.reflect.Constructor.getParameterTypes' may return a different value. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1043'"
		                                            //m by cristi
		                                            if (constructor.GetParameters()[i].ParameterType == System.Type.GetType("System.Double"))
		                                            {
		                                                //UPGRADE_TODO: The differences in the format  of parameters for constructor 'java.lang.Double.Double'  may cause compilation errors.  "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1092'"
		                                                nulls[i] = System.Double.Parse("0");
		                                            }
		                                            else
		                                            {
		                                                nulls[i] = null;
		                                            }
		                                        }
		                                    }
		                                }
		                            }
		                        }
		                    }
		                    System.Object object_Renamed = null;
		
		                    //UPGRADE_ISSUE: Method 'java.lang.reflect.AccessibleObject.setAccessible' was not converted. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1000_javalangreflectAccessibleObject'"
		                    //c by cristi
		                    //constructor.setAccessible(true);
		                    try
		                    {
		                        object_Renamed = constructor.Invoke(nulls);
		                    }
		                    catch (System.Exception e2)
		                    {
		                             throw new ODBRuntimeException(NeoDatisError.NoNullableConstructor.AddParameter("[" + DisplayUtility.ObjectArrayToString(constructor.GetParameters()) + "]").AddParameter(clazz.AssemblyQualifiedName), e2);
		                    }
		                    return object_Renamed;
		
		
		                }
		        
		        }
				
				
	
			// FIXME put the list of the classes elsewhere!
			private void FillSystemClasses()
			{
				systemClasses.Add(typeof(ClassInfoIndex).FullName, typeof(ClassInfoIndex));
				systemClasses.Add(typeof(OID).FullName, typeof(NeoDatis.Odb.OID));
				systemClasses.Add(typeof(OdbObjectOID).FullName, typeof(OdbObjectOID));
				systemClasses.Add(typeof(OdbClassOID).FullName, typeof(OdbClassOID));
				systemClasses.Add(typeof(ODBBTreeNodeSingle).FullName	, typeof(ODBBTreeNodeSingle));
				systemClasses.Add(typeof(ODBBTreeNodeMultiple).FullName, typeof(ODBBTreeNodeMultiple));
				systemClasses.Add(typeof(ODBBTreeSingle).FullName, typeof(ODBBTreeSingle));
				systemClasses.Add(typeof(NeoDatis.Btree.IBTree).FullName, typeof(IBTree));
				systemClasses.Add(typeof(IBTreeNodeOneValuePerKey).FullName, typeof(IBTreeNodeOneValuePerKey));
				systemClasses.Add(typeof(IKeyAndValue).FullName, typeof(IKeyAndValue));
				systemClasses.Add(typeof(KeyAndValue).FullName, typeof(KeyAndValue));
			}

		public virtual bool IsSystemClass(string fullClassName)
		{
			return systemClasses.ContainsKey(fullClassName);
        }

		public  byte GetClassCategory(string fullClassName)
				{
					if ((systemClasses.Count == 0))
					{
						FillSystemClasses();
					}
					if (systemClasses.ContainsKey(fullClassName))
					{
						return ClassInfo.CategorySystemClass;
					}
					return ClassInfo.CategoryUserClass;
				}

		
		
		protected virtual bool TryToCreateAnEmptyConstructor(System.Type clazz)
		{
					return false;
		}
				
			protected virtual void AddConstructor(string className, System.Reflection.ConstructorInfo
				 constructor)
			{
				classPool.AddConstrutor(className, constructor);
			}
		}
	}
	
