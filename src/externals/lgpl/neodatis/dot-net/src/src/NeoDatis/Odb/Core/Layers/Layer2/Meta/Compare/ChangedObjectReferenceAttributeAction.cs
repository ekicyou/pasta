namespace NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare
{
	/// <summary>Used to store informations about object changes when the change is only a reference change
	/// 	</summary>
	/// <author>osmadja</author>
	public class ChangedObjectReferenceAttributeAction : NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedAttribute
	{
		private long updatePosition;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectReference objectReference;

		private int recursionLevel;

		public ChangedObjectReferenceAttributeAction(long position, NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectReference
			 oref, int recursionLevel)
		{
			this.updatePosition = position;
			this.objectReference = oref;
			this.recursionLevel = recursionLevel;
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append("update position=").Append(updatePosition).Append(" - new obj ref="
				).Append(objectReference.GetOid()).Append(" - level=").Append(recursionLevel);
			return buffer.ToString();
		}

		public virtual NeoDatis.Odb.OID GetNewId()
		{
			return objectReference.GetOid();
		}

		public virtual int GetRecursionLevel()
		{
			return recursionLevel;
		}

		public virtual long GetUpdatePosition()
		{
			return updatePosition;
		}

		public virtual bool CanUpdateInPlace()
		{
			return true;
		}

		public virtual bool IsString()
		{
			return false;
		}
	}
}
