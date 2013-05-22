namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>
	/// to keep informations about an attribute of a class :
	/// <pre>
	/// - Its type
	/// - its name
	/// - If it is an index
	/// </pre>
	/// </summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class ClassAttributeInfo
	{
		private int id;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo;

		private string className;

		private string packageName;

		private string name;

		private bool isIndex;

		private string fullClassName;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType attributeType;

		/// <summary>can be null</summary>
		private System.Type nativeClass;

		public ClassAttributeInfo()
		{
		}

		public ClassAttributeInfo(int attributeId, string name, string fullClassName, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 info) : this(attributeId, name, null, fullClassName, info)
		{
		}

		public ClassAttributeInfo(int attributeId, string name, System.Type nativeClass, 
			string fullClassName, ClassInfo info) : base
			()
		{
			//private transient static int nb=0;
			this.id = attributeId;
			this.name = name;
			this.nativeClass = nativeClass;
			SetFullClassName(fullClassName);
			if (nativeClass != null)
			{
				attributeType = ODBType.GetFromClass(nativeClass
					);
			}
			else
			{
				if (fullClassName != null)
				{
					attributeType = ODBType.GetFromName(fullClassName);
				}
			}
			classInfo = info;
			isIndex = false;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo GetClassInfo()
		{
			return classInfo;
		}

		public virtual void SetClassInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo
			)
		{
			this.classInfo = classInfo;
		}

		public virtual bool IsIndex()
		{
			return isIndex;
		}

		public virtual void SetIndex(bool isIndex)
		{
			this.isIndex = isIndex;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual bool IsNative()
		{
			return attributeType.IsNative();
		}

		public virtual bool IsNonNative()
		{
			return !attributeType.IsNative();
		}

		public virtual void SetFullClassName(string fullClassName)
		{
			this.fullClassName = fullClassName;
			SetClassName(NeoDatis.Tool.Wrappers.OdbClassUtil.GetClassName(fullClassName));
			SetPackageName(NeoDatis.Tool.Wrappers.OdbClassUtil.GetPackageName(fullClassName));
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append("id=").Append(id).Append(" name=").Append(name).Append(" | is Native="
				).Append(IsNative()).Append(" | type=").Append(GetFullClassname()).Append(" | isIndex="
				).Append(isIndex);
			return buffer.ToString();
		}

		public virtual string GetClassName()
		{
			return className;
		}

		public virtual void SetClassName(string className)
		{
			this.className = className;
		}

		public virtual string GetPackageName()
		{
			return packageName;
		}

		public virtual void SetPackageName(string packageName)
		{
			this.packageName = packageName;
		}

		public virtual string GetFullClassname()
		{
			if (fullClassName != null)
			{
				return fullClassName;
			}
			if (packageName == null || packageName.Length == 0)
			{
				fullClassName = className;
				return className;
			}
			fullClassName = packageName + "." + className;
			return fullClassName;
		}

		public virtual void SetAttributeType(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			 attributeType)
		{
			this.attributeType = attributeType;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType GetAttributeType()
		{
			return attributeType;
		}

		public virtual System.Type GetNativeClass()
		{
			return nativeClass;
		}

		public virtual int GetId()
		{
			return id;
		}

		public virtual void SetId(int id)
		{
			this.id = id;
		}
	}
}
