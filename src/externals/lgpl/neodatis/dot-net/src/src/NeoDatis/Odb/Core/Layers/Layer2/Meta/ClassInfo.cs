using NeoDatis.Tool.Wrappers.List;
using NeoDatis.Tool.Wrappers.Map;
using System;
namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>A meta representation of a class</summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class ClassInfo
	{
		/// <summary>Constant used for the classCategory variable to indicate a system class</summary>
		public const byte CategorySystemClass = 1;

		/// <summary>Constant used for the classCategory variable to indicate a user class</summary>
		public const byte CategoryUserClass = 2;

		private const long serialVersionUID = 1L;

		/// <summary>To specify the type of the class : system class or user class</summary>
		private byte classCategory;

		/// <summary>The full class name with package</summary>
		private string fullClassName;

		/// <summary>Extra info of the class - no used in java version</summary>
		private string extraInfo;

		private NeoDatis.Tool.Wrappers.List.IOdbList<ClassAttributeInfo
			> attributes;

		/// <summary>
		/// This map is redundant with the field 'attributes', but it is to enable
		/// fast access to attributes by name TODO use only the map and remove list
		/// key=attribute name, key =ClassInfoattribute
		/// </summary>
		private System.Collections.Generic.IDictionary<string, ClassAttributeInfo
			> attributesByName;

		/// <summary>
		/// This map is redundant with the field 'attributes', but it is to enable
		/// fast access to attributes by id key=attribute Id(Integer), key
		/// =ClassAttributeInfo
		/// </summary>
		private System.Collections.Generic.IDictionary<int, ClassAttributeInfo
			> attributesById;

		/// <summary>
		/// To keep session original numbers, original number of committed
		/// objects,first and last object position
		/// </summary>
		private CommittedCIZoneInfo original;

		/// <summary>
		/// To keep session numbers, number of committed objects,first and last
		/// object position
		/// </summary>
		private CommittedCIZoneInfo committed;

		/// <summary>
		/// To keep session uncommitted numbers, number of uncommitted objects,first
		/// and last object position
		/// </summary>
		private CIZoneInfo uncommitted;

		/// <summary>Physical location of this class in the file (in byte)</summary>
		private long position;

		private NeoDatis.Odb.OID id;

		/// <summary>Where is the previous class.</summary>
		/// <remarks>Where is the previous class. -1, if it does not exist</remarks>
		private NeoDatis.Odb.OID previousClassOID;

		/// <summary>Where is the next class, -1, if it does not exist</summary>
		private NeoDatis.Odb.OID nextClassOID;

		/// <summary>Where starts the block of attributes definition of this class ?</summary>
		private long attributesDefinitionPosition;

		/// <summary>The size (in bytes) of the class block</summary>
		private int blockSize;

		/// <summary>Infos about the last object of this class</summary>
		private ObjectInfoHeader lastObjectInfoHeader;

		/// <summary>
		/// The max id is used to give a unique id for each attribute and allow
		/// refactoring like new field and/or removal
		/// </summary>
		private int maxAttributeId;

		private NeoDatis.Tool.Wrappers.List.IOdbList<ClassInfoIndex
			> indexes;

		[System.NonSerialized]
		private NeoDatis.Tool.Wrappers.List.IOdbList<object> history;

		public ClassInfo()
		{
			this.original = new CommittedCIZoneInfo(this
				, null, null, 0);
			this.committed = new CommittedCIZoneInfo(this
				, null, null, 0);
			this.uncommitted = new CIZoneInfo(this, null
				, null, 0);
			this.previousClassOID = null;
			this.nextClassOID = null;
			this.blockSize = -1;
			this.position = -1;
			this.maxAttributeId = -1;
			this.classCategory = CategoryUserClass;
			this.history = new NeoDatis.Tool.Wrappers.List.OdbArrayList<object>();
		}

		public ClassInfo(string className) : this(className, string.Empty, null)
		{
		}

		public ClassInfo(string className, string extraInfo) : this(className, extraInfo, 
			null)
		{
		}

		protected ClassInfo(string fullClassName, string extraInfo, NeoDatis.Tool.Wrappers.List.IOdbList
			<ClassAttributeInfo> attributes) : this()
		{
			this.fullClassName = fullClassName;
			this.extraInfo = extraInfo;
			this.attributes = attributes;
			this.attributesByName = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, ClassAttributeInfo
				>();
			this.attributesById = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<int, ClassAttributeInfo
				>();
			if (attributes != null)
			{
				FillAttributesMap();
			}
			this.maxAttributeId = (attributes == null ? 1 : attributes.Count + 1);
		}

		private void FillAttributesMap()
		{
			ClassAttributeInfo cai = null;
			if (attributesByName == null)
			{
				attributesByName = new OdbHashMap<string, ClassAttributeInfo>();
				attributesById = new OdbHashMap<int, ClassAttributeInfo>();
			}
			// attributesMap.clear();
			for (int i = 0; i < attributes.Count; i++)
			{
				cai = attributes[i];
				attributesByName[cai.GetName()] = cai;
				attributesById[cai.GetId()] = cai;
			}
		}

		public override bool Equals(object obj)
		{
			if (obj == null || obj.GetType() != typeof(ClassInfo
				))
			{
				return false;
			}
			ClassInfo classInfo = (ClassInfo
				)obj;
			return classInfo.fullClassName.Equals(fullClassName);
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append(" [ ").Append(fullClassName).Append(" - id=").Append(id);
			buffer.Append(" - previousClass=").Append(previousClassOID).Append(" - nextClass="
				).Append(nextClassOID).Append(" - attributes=(");
			// buffer.append(" | position=").append(position);
			// buffer.append(" | class=").append(className).append(" | attributes=[");
			if (attributes != null)
			{
				for (int i = 0; i < attributes.Count; i++)
				{
					ClassAttributeInfo cai = (ClassAttributeInfo
						)attributes[i];
					buffer.Append(cai.GetName()).Append(",");
				}
			}
			else
			{
				buffer.Append("not yet defined");
			}
			buffer.Append(") ]");
			return buffer.ToString();
		}

		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<ClassAttributeInfo
			> GetAttributes()
		{
			return attributes;
		}

		public virtual void SetAttributes(NeoDatis.Tool.Wrappers.List.IOdbList<ClassAttributeInfo
			> attributes)
		{
			this.attributes = attributes;
			this.maxAttributeId = attributes.Count;
			FillAttributesMap();
		}

		public virtual CommittedCIZoneInfo GetCommitedZoneInfo
			()
		{
			return committed;
		}

		public virtual long GetAttributesDefinitionPosition()
		{
			return attributesDefinitionPosition;
		}

		public virtual void SetAttributesDefinitionPosition(long definitionPosition)
		{
			this.attributesDefinitionPosition = definitionPosition;
		}

		public virtual NeoDatis.Odb.OID GetNextClassOID()
		{
			return nextClassOID;
		}

		public virtual void SetNextClassOID(NeoDatis.Odb.OID nextClassOID)
		{
			this.nextClassOID = nextClassOID;
		}

		public virtual NeoDatis.Odb.OID GetPreviousClassOID()
		{
			return previousClassOID;
		}

		public virtual void SetPreviousClassOID(NeoDatis.Odb.OID previousClassOID)
		{
			this.previousClassOID = previousClassOID;
		}

		public virtual long GetPosition()
		{
			return position;
		}

		public virtual void SetPosition(long position)
		{
			this.position = position;
		}

		public virtual int GetBlockSize()
		{
			return blockSize;
		}

		public virtual void SetBlockSize(int blockSize)
		{
			this.blockSize = blockSize;
		}

		/// <returns>the fullClassName</returns>
		public virtual string GetFullClassName()
		{
			return fullClassName;
		}

		/// <summary>
		/// This method could be optimized, but it is only on Class creation, one
		/// time in the database life time...
		/// </summary>
		/// <remarks>
		/// This method could be optimized, but it is only on Class creation, one
		/// time in the database life time... This is used to get all (non native)
		/// attributes a class info have to store them in the meta model before
		/// storing the class itself
		/// </remarks>
		/// <returns></returns>
		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<ClassAttributeInfo
			> GetAllNonNativeAttributes()
		{
			NeoDatis.Tool.Wrappers.List.IOdbList<ClassAttributeInfo
				> result = new NeoDatis.Tool.Wrappers.List.OdbArrayList<ClassAttributeInfo
				>(attributes.Count);
			ClassAttributeInfo cai = null;
			for (int i = 0; i < attributes.Count; i++)
			{
				cai = attributes[i];
				if (!cai.IsNative() || cai.GetAttributeType().IsEnum())
				{
					result.Add(cai);
				}
				else
				{
					if (cai.GetAttributeType().IsArray() && !cai.GetAttributeType().GetSubType().IsNative
						())
					{
						result.Add(new ClassAttributeInfo(-1, "subtype"
							, cai.GetAttributeType().GetSubType().GetName(), null));
					}
				}
			}
			return result;
		}

		public virtual NeoDatis.Odb.OID GetId()
		{
			return id;
		}

		public virtual void SetId(NeoDatis.Odb.OID id)
		{
			this.id = id;
		}

		public virtual ClassAttributeInfo GetAttributeInfoFromId
			(int id)
		{
			return attributesById[id];
		}

		public virtual int GetAttributeId(string name)
		{
			ClassAttributeInfo cai = attributesByName[name
				];
			if (cai == null)
			{
				return -1;
			}
			return cai.GetId();
		}

		public virtual ClassAttributeInfo GetAttributeInfoFromName
			(string name)
		{
			return attributesByName[name];
		}

		public virtual ClassAttributeInfo GetAttributeInfo
			(int index)
		{
			return attributes[index];
		}

		public virtual int GetMaxAttributeId()
		{
			return maxAttributeId;
		}

		public virtual void SetMaxAttributeId(int maxAttributeId)
		{
			this.maxAttributeId = maxAttributeId;
		}

		public virtual ClassInfoCompareResult ExtractDifferences(ClassInfo newCI, bool update)
		{
			string attributeName = null;
			ClassAttributeInfo cai1 = null;
			ClassAttributeInfo cai2 = null;
			Meta.ClassInfoCompareResult result = new ClassInfoCompareResult(GetFullClassName());
			bool isCompatible = true;
			IOdbList<ClassAttributeInfo> attributesToRemove = new OdbArrayList<ClassAttributeInfo>(10);
			IOdbList<ClassAttributeInfo> attributesToAdd = new OdbArrayList<ClassAttributeInfo>(10);
			int nbAttributes = attributes.Count;
			for (int id = 0; id < nbAttributes; id++)
			{
				// !!!WARNING : ID start with 1 and not 0
				cai1 = attributes[id];
				if (cai1 == null)
				{
					continue;
				}
				attributeName = cai1.GetName();
				cai2 = newCI.GetAttributeInfoFromId(cai1.GetId());
				if (cai2 == null)
				{
					result.AddCompatibleChange("Field '" + attributeName + "' has been removed");
					if (update)
					{
						// Simply remove the attribute from meta-model
						attributesToRemove.Add(cai1);
					}
				}
				else
				{
					if (!ODBType.TypesAreCompatible(cai1.GetAttributeType(), cai2.GetAttributeType()))
					{
						result.AddIncompatibleChange("Type of Field '" + attributeName + "' has changed : old='"
							 + cai1.GetFullClassname() + "' - new='" + cai2.GetFullClassname() + "'");
						isCompatible = false;
					}
				}
			}
			int nbNewAttributes = newCI.attributes.Count;
			for (int id = 0; id < nbNewAttributes; id++)
			{
				// !!!WARNING : ID start with 1 and not 0
				cai2 = newCI.attributes[id];
				if (cai2 == null)
				{
					continue;
				}
				attributeName = cai2.GetName();
				cai1 = GetAttributeInfoFromId(cai2.GetId());
				if (cai1 == null)
				{
					result.AddCompatibleChange("Field '" + attributeName + "' has been added");
					if (update)
					{
						// Sets the right id of attribute
						cai2.SetId(maxAttributeId + 1);
						maxAttributeId++;
						// Then adds the new attribute to the meta-model
						attributesToAdd.Add(cai2);
					}
				}
			}
			attributes.RemoveAll(attributesToRemove);
			attributes.AddAll(attributesToAdd);
			FillAttributesMap();
			return result;
		}

		public virtual int GetNumberOfAttributes()
		{
			return attributes.Count;
		}

		public virtual ClassInfoIndex AddIndexOn(string
			 name, string[] indexFields, bool acceptMultipleValuesForSameKey)
		{
			if (indexes == null)
			{
				indexes = new NeoDatis.Tool.Wrappers.List.OdbArrayList<ClassInfoIndex
					>();
			}
			ClassInfoIndex cii = new ClassInfoIndex
				();
			cii.SetClassInfoId(id);
			cii.SetCreationDate(NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs());
			cii.SetLastRebuild(cii.GetCreationDate());
			cii.SetName(name);
			cii.SetStatus(ClassInfoIndex.Enabled);
			cii.SetUnique(!acceptMultipleValuesForSameKey);
			int[] attributeIds = new int[indexFields.Length];
			for (int i = 0; i < indexFields.Length; i++)
			{
				attributeIds[i] = GetAttributeId(indexFields[i]);
			}
			cii.SetAttributeIds(attributeIds);
			indexes.Add(cii);
			return cii;
		}

		/// <summary>Removes an index</summary>
		/// <param name="cii"></param>
		public virtual void RemoveIndex(ClassInfoIndex
			 cii)
		{
			indexes.Remove(cii);
		}

		public virtual int GetNumberOfIndexes()
		{
			if (indexes == null)
			{
				return 0;
			}
			return indexes.Count;
		}

		public virtual ClassInfoIndex GetIndex(int index
			)
		{
			if (indexes == null || index >= indexes.Count)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.IndexNotFound
					.AddParameter(GetFullClassName()).AddParameter(index));
			}
			return indexes[index];
		}

		public virtual void SetIndexes(NeoDatis.Tool.Wrappers.List.IOdbList<ClassInfoIndex
			> indexes2)
		{
			this.indexes = indexes2;
		}

		/// <summary>To detect if a class has cyclic reference</summary>
		/// <returns>true if this class info has cyclic references</returns>
		public virtual bool HasCyclicReference()
		{
			return HasCyclicReference(new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, ClassInfo
				>());
		}

		/// <summary>To detect if a class has cyclic reference</summary>
		/// <param name="alreadyVisitedClasses">A hashmap containg all the already visited classes
		/// 	</param>
		/// <returns>true if this class info has cyclic references</returns>
		private bool HasCyclicReference(System.Collections.Generic.IDictionary<string, ClassInfo
			> alreadyVisitedClasses)
		{
			ClassAttributeInfo cai = null;
			bool hasCyclicRef = false;
			if (alreadyVisitedClasses[fullClassName] != null)
			{
				return true;
			}
			System.Collections.Generic.IDictionary<string, ClassInfo
				> localMap = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, ClassInfo
				>();
			alreadyVisitedClasses.Add(fullClassName, this);
			for (int i = 0; i < attributes.Count; i++)
			{
				cai = GetAttributeInfo(i);
				if (!cai.IsNative())
				{
					localMap = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, ClassInfo
						>(alreadyVisitedClasses);
					hasCyclicRef = cai.GetClassInfo().HasCyclicReference(localMap);
					if (hasCyclicRef)
					{
						return true;
					}
				}
			}
			return false;
		}

		public virtual byte GetClassCategory()
		{
			return classCategory;
		}

		public virtual void SetClassCategory(byte classInfoType)
		{
			this.classCategory = classInfoType;
		}

		public virtual ObjectInfoHeader GetLastObjectInfoHeader
			()
		{
			return lastObjectInfoHeader;
		}

		public virtual void SetLastObjectInfoHeader(ObjectInfoHeader
			 lastObjectInfoHeader)
		{
			this.lastObjectInfoHeader = lastObjectInfoHeader;
		}

		public virtual CIZoneInfo GetUncommittedZoneInfo
			()
		{
			return uncommitted;
		}

		/// <summary>Get number of objects: committed and uncommitted</summary>
		/// <returns>The number of committed and uncommitted objects</returns>
		public virtual long GetNumberOfObjects()
		{
			return committed.GetNbObjects() + uncommitted.GetNbObjects();
		}

		public virtual CommittedCIZoneInfo GetOriginalZoneInfo
			()
		{
			return original;
		}

		public virtual bool IsSystemClass()
		{
			return classCategory == CategorySystemClass;
		}

		public virtual ClassInfoIndex GetIndexWithName
			(string name)
		{
			ClassInfoIndex cii = null;
			if (indexes == null)
			{
				return null;
			}
			for (int i = 0; i < indexes.Count; i++)
			{
				cii = indexes[i];
				if (cii.GetName().Equals(name))
				{
					return cii;
				}
			}
			return null;
		}

		public virtual ClassInfoIndex GetIndexForAttributeId
			(int attributeId)
		{
			ClassInfoIndex cii = null;
			if (indexes == null)
			{
				return null;
			}
			for (int i = 0; i < indexes.Count; i++)
			{
				cii = indexes[i];
				if (cii.GetAttributeIds().Length == 1 && cii.GetAttributeId(0) == attributeId)
				{
					return cii;
				}
			}
			return null;
		}

		public virtual ClassInfoIndex GetIndexForAttributeIds
			(int[] attributeIds)
		{
			ClassInfoIndex cii = null;
			if (indexes == null)
			{
				return null;
			}
			for (int i = 0; i < indexes.Count; i++)
			{
				cii = indexes[i];
				if (cii.MatchAttributeIds(attributeIds))
				{
					return cii;
				}
			}
			return null;
		}

		public virtual string[] GetAttributeNames(int[] attributeIds)
		{
			int nbIds = attributeIds.Length;
			string[] names = new string[nbIds];
			for (int i = 0; i < nbIds; i++)
			{
				names[i] = GetAttributeInfoFromId(attributeIds[i]).GetName();
			}
			return names;
		}

		public virtual System.Collections.Generic.IList<string> GetAttributeNamesAsList(int
			[] attributeIds)
		{
			int nbIds = attributeIds.Length;
			System.Collections.Generic.IList<string> names = new System.Collections.Generic.List
				<string>(attributeIds.Length);
			for (int i = 0; i < nbIds; i++)
			{
				names.Add(GetAttributeInfoFromId(attributeIds[i]).GetName());
			}
			return names;
		}

		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<ClassInfoIndex
			> GetIndexes()
		{
			if (indexes == null)
			{
				return new NeoDatis.Tool.Wrappers.List.OdbArrayList<ClassInfoIndex
					>();
			}
			return indexes;
		}

		public virtual void RemoveAttribute(ClassAttributeInfo
			 cai)
		{
			attributes.Remove(cai);
			attributesByName.Remove(cai.GetName());
		}

		public virtual void AddAttribute(ClassAttributeInfo
			 cai)
		{
			cai.SetId(maxAttributeId++);
			attributes.Add(cai);
			attributesByName.Add(cai.GetName(), cai);
		}

		public virtual void SetFullClassName(string fullClassName)
		{
			this.fullClassName = fullClassName;
		}

		public virtual void AddHistory(object o)
		{
			if (history == null)
			{
				history = new NeoDatis.Tool.Wrappers.List.OdbArrayList<object>(1);
			}
			history.Add(o);
		}

		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<object> GetHistory()
		{
			return history;
		}

		public virtual bool HasIndex(string indexName)
		{
			ClassInfoIndex cii = null;
			if (indexes == null)
			{
				return false;
			}
			for (int i = 0; i < indexes.Count; i++)
			{
				cii = indexes[i];
				if (indexName.Equals(cii.GetName()))
				{
					return true;
				}
			}
			return false;
		}

		public virtual bool HasIndex()
		{
			return indexes != null && !indexes.IsEmpty();
		}

		public virtual void SetExtraInfo(string extraInfo)
		{
			this.extraInfo = extraInfo;
		}

		public virtual string GetExtraInfo()
		{
			return extraInfo;
		}

		public virtual ClassInfo Duplicate(bool onlyData
			)
		{
			ClassInfo ci = new ClassInfo
				(fullClassName);
			ci.extraInfo = extraInfo;
			ci.SetAttributes(attributes);
			ci.SetClassCategory(classCategory);
			ci.SetMaxAttributeId(maxAttributeId);
			if (onlyData)
			{
				return ci;
			}
			ci.SetAttributesDefinitionPosition(attributesDefinitionPosition);
			ci.SetBlockSize(blockSize);
			ci.SetExtraInfo(extraInfo);
			ci.SetId(id);
			ci.SetPreviousClassOID(previousClassOID);
			ci.SetNextClassOID(nextClassOID);
			ci.SetLastObjectInfoHeader(lastObjectInfoHeader);
			ci.SetPosition(position);
			ci.SetIndexes(indexes);
			return ci;
		}
	}
}
