namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>An index of a class info</summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class ClassInfoIndex
	{
		public const byte Enabled = 1;

		public const byte Disabled = 2;

		private NeoDatis.Odb.OID classInfoId;

		private string name;

		private byte status;

		private bool isUnique;

		private long creationDate;

		private long lastRebuild;

		private int[] attributeIds;

		private NeoDatis.Btree.IBTree btree;

		public virtual NeoDatis.Odb.OID GetClassInfoId()
		{
			return classInfoId;
		}

		public virtual void SetClassInfoId(NeoDatis.Odb.OID classInfoId)
		{
			this.classInfoId = classInfoId;
		}

		public virtual int[] GetAttributeIds()
		{
			return attributeIds;
		}

		public virtual void SetAttributeIds(int[] attributeIds)
		{
			this.attributeIds = attributeIds;
		}

		public virtual long GetCreationDate()
		{
			return creationDate;
		}

		public virtual void SetCreationDate(long creationDate)
		{
			this.creationDate = creationDate;
		}

		public virtual bool IsUnique()
		{
			return isUnique;
		}

		public virtual void SetUnique(bool isUnique)
		{
			this.isUnique = isUnique;
		}

		public virtual long GetLastRebuild()
		{
			return lastRebuild;
		}

		public virtual void SetLastRebuild(long lastRebuild)
		{
			this.lastRebuild = lastRebuild;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual byte GetStatus()
		{
			return status;
		}

		public virtual void SetStatus(byte status)
		{
			this.status = status;
		}

		public virtual int GetAttributeId(int index)
		{
			return attributeIds[index];
		}

		public virtual void SetBTree(NeoDatis.Btree.IBTree btree)
		{
			this.btree = btree;
		}

		public virtual NeoDatis.Btree.IBTree GetBTree()
		{
			return this.btree;
		}

		public virtual NeoDatis.Tool.Wrappers.OdbComparable ComputeKey(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi)
		{
			return NeoDatis.Odb.Core.Query.Execution.IndexTool.BuildIndexKey(name, nnoi, attributeIds
				);
		}

		public virtual int GetNbAttributes()
		{
			return attributeIds.Length;
		}

		/// <summary>Check if a list of attribute can use the index</summary>
		/// <param name="attributeIdsToMatch"></param>
		/// <returns>true if the list of attribute can use this index</returns>
		public virtual bool MatchAttributeIds(int[] attributeIdsToMatch)
		{
			//TODO an index with lesser attribute than the one to match can be used
			if (attributeIds.Length != attributeIdsToMatch.Length)
			{
				return false;
			}
			bool found = false;
			for (int i = 0; i < attributeIdsToMatch.Length; i++)
			{
				found = false;
				for (int j = 0; j < attributeIds.Length; j++)
				{
					if (attributeIds[j] == attributeIdsToMatch[i])
					{
						found = true;
						break;
					}
				}
				if (!found)
				{
					return false;
				}
			}
			return true;
		}
	}
}
