namespace NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare
{
	/// <summary>Used to store informations about object changes at attribute level</summary>
	/// <author>osmadja</author>
	public class ChangedNativeAttributeAction : NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedAttribute
	{
		/// <summary>The old object meta representation: is case of no in place update</summary>
		private NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo oldNnoi;

		/// <summary>The new object meta representation: is case of no in place update</summary>
		private NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo newNoi;

		private long updatePosition;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo noiWithNewValue;

		private int recursionLevel;

		private string attributeName;

		/// <summary>This boolean value is set to true when original object is null, is this case there is no way to do in place update
		/// 	</summary>
		private bool reallyCantDoInPlaceUpdate;

		public ChangedNativeAttributeAction(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 oldNnoi, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo newNnoi, long
			 position, NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo newNoi, int recursionLevel
			, bool canDoInPlaceUpdate, string attributeName)
		{
			this.oldNnoi = oldNnoi;
			this.newNoi = newNnoi;
			this.updatePosition = position;
			this.noiWithNewValue = newNoi;
			this.recursionLevel = recursionLevel;
			this.reallyCantDoInPlaceUpdate = canDoInPlaceUpdate;
			this.attributeName = attributeName;
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append("field : '").Append(attributeName).Append("' - update position=").Append
				(updatePosition).Append(" - new value=").Append(noiWithNewValue).Append(" - level="
				).Append(recursionLevel);
			return buffer.ToString();
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo GetNoiWithNewValue
			()
		{
			return noiWithNewValue;
		}

		public virtual int GetRecursionLevel()
		{
			return recursionLevel;
		}

		public virtual long GetUpdatePosition()
		{
			return updatePosition;
		}

		public virtual bool ReallyCantDoInPlaceUpdate()
		{
			return reallyCantDoInPlaceUpdate;
		}

		public virtual bool InPlaceUpdateIsGuaranteed()
		{
			return !reallyCantDoInPlaceUpdate && noiWithNewValue.IsAtomicNativeObject() && noiWithNewValue
				.GetOdbTypeId() != NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.StringId;
		}

		public virtual bool IsString()
		{
			return noiWithNewValue.GetOdbTypeId() == NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.StringId;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo GetOldNnoi
			()
		{
			return oldNnoi;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo GetNewNoi
			()
		{
			return newNoi;
		}
	}
}
