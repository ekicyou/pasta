namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>
	/// To keep info about a non native object
	/// <pre>
	/// - Keeps its class info : a meta information to describe its type
	/// - All its attributes values
	/// - Its Pointers : its position, the previous object OID, the next object OID
	/// - The Object being represented by The meta information
	/// </pre>
	/// </summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class NonNativeObjectInfo : NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
	{
		/// <summary>The object being represented</summary>
		[System.NonSerialized]
		protected object @object;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader objectHeader;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo[] attributeValues;

		/// <summary>To keep track of all non native objects , not used for instance</summary>
		private NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			> allNonNativeObjects;

		private int maxNbattributes;

		public NonNativeObjectInfo() : base(null)
		{
		}

		public NonNativeObjectInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader 
			oip, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo) : base(null)
		{
			// private List attributeValues;
			this.classInfo = classInfo;
			this.objectHeader = oip;
			if (classInfo != null)
			{
				this.maxNbattributes = classInfo.GetMaxAttributeId();
				this.attributeValues = new NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
					[maxNbattributes];
			}
			this.allNonNativeObjects = null;
		}

		public NonNativeObjectInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo
			) : base(null)
		{
			//new OdbArrayList<NonNativeObjectInfo>();
			this.classInfo = classInfo;
			this.objectHeader = new NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader(-1, 
				null, null, (classInfo != null ? classInfo.GetId() : null), null, null);
			if (classInfo != null)
			{
				this.maxNbattributes = classInfo.GetMaxAttributeId();
				this.attributeValues = new NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
					[maxNbattributes];
			}
			this.allNonNativeObjects = null;
		}

		public NonNativeObjectInfo(object @object, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 info, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo[] values, long[] 
			attributesIdentification, int[] attributeIds) : base(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.GetFromName(info.GetFullClassName()))
		{
			//new OdbArrayList<NonNativeObjectInfo>();
			this.@object = @object;
			this.classInfo = info;
			this.attributeValues = values;
			this.maxNbattributes = classInfo.GetMaxAttributeId();
			if (attributeValues == null)
			{
				this.attributeValues = new NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
					[maxNbattributes];
			}
			this.objectHeader = new NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader(-1, 
				null, null, (classInfo != null ? classInfo.GetId() : null), attributesIdentification
				, attributeIds);
			this.allNonNativeObjects = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				>();
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader GetHeader()
		{
			return objectHeader;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo GetAttributeValueFromId
			(int attributeId)
		{
			return attributeValues[attributeId - 1];
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo GetClassInfo()
		{
			return classInfo;
		}

		public virtual void SetClassInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo
			)
		{
			if (classInfo != null)
			{
				this.classInfo = classInfo;
				this.objectHeader.SetClassInfoId(classInfo.GetId());
			}
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder(classInfo.GetFullClassName
				()).Append("(").Append(GetOid()).Append(")=");
			if (attributeValues == null)
			{
				buffer.Append("null attribute values");
				return buffer.ToString();
			}
			for (int i = 0; i < attributeValues.Length; i++)
			{
				if (i != 0)
				{
					buffer.Append(",");
				}
				string attributeName = (classInfo != null ? (classInfo.GetAttributeInfo(i)).GetName
					() : "?");
				buffer.Append(attributeName).Append("=");
				object @object = attributeValues[i];
				if (@object == null)
				{
					buffer.Append(" null java object - should not happen , ");
				}
				else
				{
					NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType type = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
						.GetFromClass(attributeValues[i].GetType());
					if (@object is NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeNullObjectInfo)
					{
						buffer.Append("null");
						continue;
					}
					if (@object is NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeDeletedObjectInfo)
					{
						buffer.Append("deleted object");
						continue;
					}
					if (@object is NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo)
					{
						NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo noi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo
							)@object;
						buffer.Append(noi.ToString());
						continue;
					}
					if (@object is NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)
					{
						NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
							)@object;
						buffer.Append("@").Append(nnoi.GetClassInfo().GetFullClassName()).Append("(id=").
							Append(nnoi.GetOid()).Append(")");
						continue;
					}
					if (@object is NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectReference)
					{
						buffer.Append(@object.ToString());
						continue;
					}
					buffer.Append("@").Append(NeoDatis.Tool.Wrappers.OdbClassUtil.GetClassName(type.GetName
						()));
				}
			}
			return buffer.ToString();
		}

		public virtual NeoDatis.Odb.OID GetNextObjectOID()
		{
			return objectHeader.GetNextObjectOID();
		}

		public virtual void SetNextObjectOID(NeoDatis.Odb.OID nextObjectOID)
		{
			this.objectHeader.SetNextObjectOID(nextObjectOID);
		}

		public virtual NeoDatis.Odb.OID GetPreviousObjectOID()
		{
			return objectHeader.GetPreviousObjectOID();
		}

		public virtual void SetPreviousInstanceOID(NeoDatis.Odb.OID previousObjectOID)
		{
			this.objectHeader.SetPreviousObjectOID(previousObjectOID);
		}

		public override long GetPosition()
		{
			return objectHeader.GetPosition();
		}

		public override void SetPosition(long position)
		{
			objectHeader.SetPosition(position);
		}

		public override object GetObject()
		{
			return @object;
		}

		public virtual object GetValueOf(string attributeName)
		{
			int attributeId = -1;
			bool isRelation = attributeName.IndexOf(".") != -1;
			if (!isRelation)
			{
				attributeId = GetClassInfo().GetAttributeId(attributeName);
				return GetAttributeValueFromId(attributeId).GetObject();
			}
			int firstDotIndex = attributeName.IndexOf(".");
			string firstAttributeName = NeoDatis.Tool.Wrappers.OdbString.Substring(attributeName
				, 0, firstDotIndex);
			attributeId = GetClassInfo().GetAttributeId(firstAttributeName);
			object @object = attributeValues[attributeId];
			if (@object is NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
					)@object;
				return nnoi.GetValueOf(NeoDatis.Tool.Wrappers.OdbString.Substring(attributeName, 
					firstDotIndex + 1, attributeName.Length));
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ClassInfoDoNotHaveTheAttribute
				.AddParameter(GetClassInfo().GetFullClassName()).AddParameter(attributeName));
		}

		/// <summary>Used to change the value of an attribute</summary>
		/// <param name="attributeName"></param>
		/// <param name="aoi"></param>
		public virtual void SetValueOf(string attributeName, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			 aoi)
		{
			int attributeId = -1;
			bool isRelation = attributeName.IndexOf(".") != -1;
			if (!isRelation)
			{
				attributeId = GetClassInfo().GetAttributeId(attributeName);
				SetAttributeValue(attributeId, aoi);
				return;
			}
			int firstDotIndex = attributeName.IndexOf(".");
			string firstAttributeName = NeoDatis.Tool.Wrappers.OdbString.Substring(attributeName
				, 0, firstDotIndex);
			attributeId = GetClassInfo().GetAttributeId(firstAttributeName);
			object @object = attributeValues[attributeId];
			if (@object is NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
					)@object;
				nnoi.SetValueOf(NeoDatis.Tool.Wrappers.OdbString.Substring(attributeName, firstDotIndex
					 + 1, attributeName.Length), aoi);
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ClassInfoDoNotHaveTheAttribute
				.AddParameter(GetClassInfo().GetFullClassName()).AddParameter(attributeName));
		}

		public virtual NeoDatis.Odb.OID GetOid()
		{
			if (GetHeader() == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.UnexpectedSituation
					.AddParameter("Null Object Info Header"));
			}
			return GetHeader().GetOid();
		}

		public virtual void SetOid(NeoDatis.Odb.OID oid)
		{
			if (GetHeader() != null)
			{
				GetHeader().SetOid(oid);
			}
		}

		public override bool IsNonNativeObject()
		{
			return true;
		}

		public override bool IsNull()
		{
			return false;
		}

		public virtual void Clear()
		{
			attributeValues = null;
		}

		/// <summary>Create a copy oh this meta object</summary>
		/// <param name="onlyData">if true, only copy attributes values</param>
		/// <returns></returns>
		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo CreateCopy
			(System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			> cache, bool onlyData)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)cache[objectHeader.GetOid()];
			if (nnoi != null)
			{
				return nnoi;
			}
			if (onlyData)
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = new NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
					();
				nnoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo(@object, classInfo
					, null, oih.GetAttributesIdentification(), oih.GetAttributeIds());
			}
			else
			{
				nnoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo(@object, classInfo
					, null, objectHeader.GetAttributesIdentification(), objectHeader.GetAttributeIds
					());
				nnoi.GetHeader().SetOid(GetHeader().GetOid());
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo[] newAttributeValues = new 
				NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo[attributeValues.Length];
			for (int i = 0; i < attributeValues.Length; i++)
			{
				newAttributeValues[i] = attributeValues[i].CreateCopy(cache, onlyData);
			}
			nnoi.attributeValues = newAttributeValues;
			cache.Add(objectHeader.GetOid(), nnoi);
			return nnoi;
		}

		public virtual void SetAttributeValue(int attributeId, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			 aoi)
		{
			attributeValues[attributeId - 1] = aoi;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo[] GetAttributeValues
			()
		{
			return attributeValues;
		}

		public virtual int GetMaxNbattributes()
		{
			return maxNbattributes;
		}

		/// <summary>The performance of this method is bad.</summary>
		/// <remarks>The performance of this method is bad. But it is not used by the engine, only in the ODBExplorer
		/// 	</remarks>
		/// <param name="aoi"></param>
		/// <returns></returns>
		public virtual int GetAttributeId(NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			 aoi)
		{
			for (int i = 0; i < attributeValues.Length; i++)
			{
				if (aoi == attributeValues[i])
				{
					return i + 1;
				}
			}
			return -1;
		}

		/// <summary>Return the position where the position of an attribute is stored.</summary>
		/// <remarks>
		/// Return the position where the position of an attribute is stored.
		/// <pre>
		/// If a object has 3 attributes and if it is stored at position x
		/// Then the number of attributes (3) is stored at x+StorageEngineConstant.OBJECT_OFFSET_NB_ATTRIBUTES
		/// and first attribute id definition is stored at x+StorageEngineConstant.OBJECT_OFFSET_NB_ATTRIBUTES+size-of(int)
		/// and first attribute position is stored at x+StorageEngineConstant.OBJECT_OFFSET_NB_ATTRIBUTES+size-of(int)+size-of(int)
		/// the second attribute id is stored at x+StorageEngineConstant.OBJECT_OFFSET_NB_ATTRIBUTES+size-of(int)+size-of(int)+size-of(long)
		/// the second attribute position is stored at x+StorageEngineConstant.OBJECT_OFFSET_NB_ATTRIBUTES+size-of(int)+size-of(int)+size-of(long)+size-of(int)
		/// <pre>
		/// FIXME Remove dependency of StorageEngineConstant!
		/// </remarks>
		/// <param name="attributeId"></param>
		/// <returns>The position where this attribute is stored</returns>
		public virtual long GetAttributeDefinitionPosition(int attributeId)
		{
			long offset = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.ObjectOffsetNbAttributes;
			// delta =
			// Skip NbAttribute (int) +
			// Delta attribute (attributeId-1) * attribute definition size =
			// INT+LONG
			// Skip attribute Id (int)
			long delta = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Integer.GetSize() + (attributeId
				 - 1) * (NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Integer.GetSize() + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.Long.GetSize()) + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Integer.GetSize(
				);
			return GetPosition() + offset + delta;
		}

		public override void SetObject(object @object)
		{
			this.@object = @object;
		}

		public override bool Equals(object obj)
		{
			return base.Equals(obj);
		}

		public override int GetHashCode()
		{
			// This happens when the object is deleted
			if (objectHeader == null)
			{
				return -1;
			}
			return objectHeader.GetHashCode();
		}

		/// <param name="header"></param>
		public virtual void SetHeader(NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
			 header)
		{
			this.objectHeader = header;
		}
	}
}
