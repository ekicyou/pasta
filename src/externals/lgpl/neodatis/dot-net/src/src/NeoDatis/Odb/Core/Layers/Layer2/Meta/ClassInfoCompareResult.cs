namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>To keep track of differences between two ClassInfo.</summary>
	/// <remarks>To keep track of differences between two ClassInfo. Ussed by the MetaModel compatibility checker
	/// 	</remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class ClassInfoCompareResult
	{
		private string fullClassName;

		private NeoDatis.Tool.Wrappers.List.IOdbList<string> incompatibleChanges;

		private NeoDatis.Tool.Wrappers.List.IOdbList<string> compatibleChanges;

		public ClassInfoCompareResult(string fullClassName)
		{
			this.fullClassName = fullClassName;
			incompatibleChanges = new NeoDatis.Tool.Wrappers.List.OdbArrayList<string>(5);
			compatibleChanges = new NeoDatis.Tool.Wrappers.List.OdbArrayList<string>(5);
		}

		/// <returns>the compatibleChanges</returns>
		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<string> GetCompatibleChanges(
			)
		{
			return compatibleChanges;
		}

		/// <param name="compatibleChanges">the compatibleChanges to set</param>
		public virtual void SetCompatibleChanges(NeoDatis.Tool.Wrappers.List.IOdbList<string
			> compatibleChanges)
		{
			this.compatibleChanges = compatibleChanges;
		}

		/// <returns>the incompatibleChanges</returns>
		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<string> GetIncompatibleChanges
			()
		{
			return incompatibleChanges;
		}

		/// <param name="incompatibleChanges">the incompatibleChanges to set</param>
		public virtual void SetIncompatibleChanges(NeoDatis.Tool.Wrappers.List.IOdbList<string
			> incompatibleChanges)
		{
			this.incompatibleChanges = incompatibleChanges;
		}

		/// <returns>the isCompatible</returns>
		public virtual bool IsCompatible()
		{
			return incompatibleChanges.IsEmpty();
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append(fullClassName).Append(" is Compatible = ").Append(IsCompatible()).Append
				("\n");
			buffer.Append("Incompatible changes = ").Append(incompatibleChanges);
			buffer.Append("\nCompatible changes = ").Append(compatibleChanges);
			return buffer.ToString();
		}

		public virtual void AddCompatibleChange(string o)
		{
			compatibleChanges.Add(o);
		}

		public virtual void AddIncompatibleChange(string o)
		{
			incompatibleChanges.Add(o);
		}

		public virtual bool HasCompatibleChanges()
		{
			return !compatibleChanges.IsEmpty();
		}

		/// <returns>the fullClassName</returns>
		public virtual string GetFullClassName()
		{
			return fullClassName;
		}
	}
}
