namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>Some basic info about an object info like position, its class info,...</summary>
	/// <remarks>Some basic info about an object info like position, its class info,...</remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class ObjectInfoHeader
	{
		private long position;

		private NeoDatis.Odb.OID previousObjectOID;

		private NeoDatis.Odb.OID nextObjectOID;

		private NeoDatis.Odb.OID classInfoId;

		/// <summary>Can be position(for native object) or id(for non native object, positions are positive e ids are negative
		/// 	</summary>
		private long[] attributesIdentification;

		private int[] attributeIds;

		private NeoDatis.Odb.OID oid;

		private long creationDate;

		private long updateDate;

		private int objectVersion;

		public ObjectInfoHeader(long position, NeoDatis.Odb.OID previousObjectOID, NeoDatis.Odb.OID
			 nextObjectOID, NeoDatis.Odb.OID classInfoId, long[] attributesIdentification, int
			[] attributeIds)
		{
			this.position = position;
			this.oid = null;
			this.previousObjectOID = previousObjectOID;
			this.nextObjectOID = nextObjectOID;
			this.classInfoId = classInfoId;
			this.attributesIdentification = attributesIdentification;
			this.attributeIds = attributeIds;
			this.objectVersion = 1;
			this.creationDate = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
		}

		public ObjectInfoHeader() : base()
		{
			this.position = -1;
			this.oid = null;
			this.objectVersion = 1;
			this.creationDate = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
		}

		public virtual int GetNbAttributes()
		{
			return attributesIdentification.Length;
		}

		public virtual NeoDatis.Odb.OID GetNextObjectOID()
		{
			return nextObjectOID;
		}

		public virtual void SetNextObjectOID(NeoDatis.Odb.OID nextObjectOID)
		{
			this.nextObjectOID = nextObjectOID;
		}

		public virtual long GetPosition()
		{
			return position;
		}

		public virtual void SetPosition(long position)
		{
			this.position = position;
		}

		//	/**
		//     * @return Returns the classInfoId.
		//     */
		//    public long getClassInfoId() {
		//        return classInfoId;
		//    }
		//    /**
		//     * @param classInfoId The classInfoId to set.
		//     */
		//    public void setClassInfoId(long classInfoId) {
		//        this.classInfoId = classInfoId;
		//    }
		public virtual NeoDatis.Odb.OID GetPreviousObjectOID()
		{
			return previousObjectOID;
		}

		public virtual void SetPreviousObjectOID(NeoDatis.Odb.OID previousObjectOID)
		{
			this.previousObjectOID = previousObjectOID;
		}

		public virtual NeoDatis.Odb.OID GetClassInfoId()
		{
			return classInfoId;
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append("oid=").Append(oid).Append(" - ");
			//.append("class info id=").append(classInfoId);
			buffer.Append(" - position=").Append(position).Append(" | prev=").Append(previousObjectOID
				);
			buffer.Append(" | next=").Append(nextObjectOID);
			buffer.Append(" attrs =[");
			if (attributesIdentification != null)
			{
				for (int i = 0; i < attributesIdentification.Length; i++)
				{
					buffer.Append(attributesIdentification[i]).Append(" ");
				}
			}
			else
			{
				buffer.Append(" nulls ");
			}
			buffer.Append(" ]");
			return buffer.ToString();
		}

		public virtual long[] GetAttributesIdentification()
		{
			return attributesIdentification;
		}

		public virtual void SetAttributesIdentification(long[] attributesIdentification)
		{
			this.attributesIdentification = attributesIdentification;
		}

		public virtual NeoDatis.Odb.OID GetOid()
		{
			return oid;
		}

		public virtual void SetOid(NeoDatis.Odb.OID oid)
		{
			this.oid = oid;
		}

		public virtual long GetCreationDate()
		{
			return creationDate;
		}

		public virtual void SetCreationDate(long creationDate)
		{
			this.creationDate = creationDate;
		}

		public virtual long GetUpdateDate()
		{
			return updateDate;
		}

		public virtual void SetUpdateDate(long updateDate)
		{
			this.updateDate = updateDate;
		}

		/// <summary>
		/// Return the attribute identification (position or id) from the attribute id
		/// FIXME Remove dependency from StorageEngineConstant
		/// </summary>
		/// <param name="attributeId"></param>
		/// <returns>-1 if attribute with this id does not exist</returns>
		public virtual long GetAttributeIdentificationFromId(int attributeId)
		{
			if (attributeIds == null)
			{
				return NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.NullObjectIdId;
			}
			for (int i = 0; i < attributeIds.Length; i++)
			{
				if (attributeIds[i] == attributeId)
				{
					return attributesIdentification[i];
				}
			}
			return NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.NullObjectIdId;
		}

		public virtual long GetAttributeId(int attributeIndex)
		{
			return attributeIds[attributeIndex];
		}

		public virtual void SetAttributesIds(int[] ids)
		{
			attributeIds = ids;
		}

		public virtual int[] GetAttributeIds()
		{
			return attributeIds;
		}

		public virtual void SetClassInfoId(NeoDatis.Odb.OID classInfoId2)
		{
			this.classInfoId = classInfoId2;
		}

		public virtual int GetObjectVersion()
		{
			return objectVersion;
		}

		public virtual void SetObjectVersion(int objectVersion)
		{
			this.objectVersion = objectVersion;
		}

		public override int GetHashCode()
		{
			int prime = 31;
			int result = 1;
			result = prime * result + (int)(position ^ ((position) >> (32 & 0x1f)));
			return result;
		}

		public override bool Equals(object obj)
		{
			if (this == obj)
			{
				return true;
			}
			if (obj == null)
			{
				return false;
			}
			if (GetType() != obj.GetType())
			{
				return false;
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader other = (NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
				)obj;
			if (position != other.position)
			{
				return false;
			}
			return true;
		}

		public virtual void IncrementVersionAndUpdateDate()
		{
			objectVersion++;
			updateDate = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader Duplicate()
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = new NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
				();
			oih.SetAttributesIdentification(attributesIdentification);
			oih.SetAttributesIds(attributeIds);
			oih.SetClassInfoId(classInfoId);
			oih.SetCreationDate(creationDate);
			oih.SetNextObjectOID(nextObjectOID);
			oih.SetObjectVersion(objectVersion);
			oih.SetOid(oid);
			oih.SetPosition(position);
			oih.SetPreviousObjectOID(previousObjectOID);
			oih.SetUpdateDate(updateDate);
			return oih;
		}
	}
}
