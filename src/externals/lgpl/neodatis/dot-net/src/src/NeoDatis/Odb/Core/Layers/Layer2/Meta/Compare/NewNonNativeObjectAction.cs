namespace NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare
{
	/// <summary>Used to store that a new Object was created when comparing to Objects.</summary>
	/// <remarks>Used to store that a new Object was created when comparing to Objects.</remarks>
	/// <author>osmadja</author>
	public class NewNonNativeObjectAction
	{
		private long updatePosition;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi;

		private int recursionLevel;

		private string attributeName;

		public NewNonNativeObjectAction(long position, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi, int recursionLevel, string attributeName)
		{
			this.updatePosition = position;
			this.nnoi = nnoi;
			this.recursionLevel = recursionLevel;
			this.attributeName = attributeName;
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append("field ").Append(attributeName).Append(" - update reference position="
				).Append(updatePosition).Append(" - new nnoi=").Append(nnoi).Append(" - level=")
				.Append(recursionLevel);
			return buffer.ToString();
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo GetNnoi()
		{
			return nnoi;
		}

		public virtual int GetRecursionLevel()
		{
			return recursionLevel;
		}

		public virtual long GetUpdatePosition()
		{
			return updatePosition;
		}
	}
}
