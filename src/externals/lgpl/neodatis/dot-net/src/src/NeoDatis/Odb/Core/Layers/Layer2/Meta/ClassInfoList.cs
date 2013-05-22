namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>A simple list to contain some class infos.</summary>
	/// <remarks>
	/// A simple list to contain some class infos.
	/// <pre>
	/// It used by ClassIntropector.introspect to return all the class info detected by introspecting a class.
	/// For example, if we have a class Class1 that has a field of type Class2. And Class2 has a field of type Class3.
	/// Introspecting Class1 return a ClassInfoList with the classes Class1, Class2, Class3. Class1 being the main class info
	/// </pre>
	/// </remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class ClassInfoList
	{
		/// <summary>key=ClassInfoName,value=ClassInfo</summary>
		private System.Collections.Generic.IDictionary<string, ClassInfo> classInfos;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo mainClassInfo;

		public ClassInfoList()
		{
		}

		public ClassInfoList(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo mainClassInfo
			)
		{
			this.classInfos = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
				>();
			this.classInfos[mainClassInfo.GetFullClassName()] = mainClassInfo;
			this.mainClassInfo = mainClassInfo;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo GetMainClassInfo()
		{
			return mainClassInfo;
		}

		public virtual void AddClassInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo
			)
		{
			classInfos[classInfo.GetFullClassName()] = classInfo;
		}

		public virtual System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			> GetClassInfos()
		{
			return classInfos.Values;
		}

		public virtual bool HasClassInfos()
		{
			return classInfos.Count != 0;
		}

		/// <param name="name"></param>
		/// <returns>null if it does not exist</returns>
		public virtual ClassInfo GetClassInfoWithName(string name)
		{
            ClassInfo ci = null;
            classInfos.TryGetValue(name, out ci);
            return ci;
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append(classInfos.Count).Append(" - ").Append(classInfos.Keys);
			return buffer.ToString();
		}

		public virtual void SetMainClassInfo(ClassInfo classInfo)
		{
			this.mainClassInfo = classInfo;
		}
	}
}
