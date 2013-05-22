namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>To keep meta informations about an object</summary>
	/// <author>olivier smadja</author>
	[System.Serializable]
	public abstract class AbstractObjectInfo
	{
		/// <summary>The Type Id of the object</summary>
		protected int odbTypeId;

		/// <summary>The Type of the object</summary>
		protected NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType odbType;

		/// <summary>The position of the object</summary>
		protected long position;

		public AbstractObjectInfo(int typeId)
		{
			this.odbTypeId = typeId;
		}

		public AbstractObjectInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType type)
		{
			if (type != null)
			{
				this.odbTypeId = type.GetId();
			}
			this.odbType = type;
		}

		public virtual bool IsNative()
		{
			return IsAtomicNativeObject() || IsArrayObject() || IsCollectionObject() || IsMapObject
				();
		}

		public virtual bool IsGroup()
		{
			return IsCollectionObject() || IsMapObject() || IsArrayObject();
		}

		public virtual bool IsNull()
		{
			return GetObject() == null;
		}

		public abstract object GetObject();

		public abstract void SetObject(object @object);

		public virtual int GetOdbTypeId()
		{
			return odbTypeId;
		}

		public virtual void SetOdbTypeId(int odbTypeId)
		{
			this.odbTypeId = odbTypeId;
		}

		public virtual long GetPosition()
		{
			return position;
		}

		public virtual void SetPosition(long position)
		{
			this.position = position;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType GetOdbType()
		{
			if (odbType == null)
			{
				odbType = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.GetFromId(odbTypeId);
			}
			return odbType;
		}

		public virtual void SetOdbType(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType odbType
			)
		{
			this.odbType = odbType;
		}

		public virtual bool IsNonNativeObject()
		{
			return false;
		}

		public virtual bool IsAtomicNativeObject()
		{
			return false;
		}

		public virtual bool IsCollectionObject()
		{
			return false;
		}

		public virtual bool IsMapObject()
		{
			return false;
		}

		public virtual bool IsArrayObject()
		{
			return false;
		}

		public virtual bool IsDeletedObject()
		{
			return false;
		}

		public virtual bool IsObjectReference()
		{
			return false;
		}

		public virtual bool IsEnumObject()
		{
			return false;
		}

		public abstract NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo CreateCopy
			(System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			> cache, bool onlyData);
	}
}
