namespace NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization
{
	public class Serializer
	{
		public static readonly string CollectionElementSeparator = ",";

		public static readonly string FieldSeparator = ";";

		public static readonly string AttributeSeparator = "|";

		public static readonly string CollectionStart = "(";

		public static readonly string CollectionEnd = ")";

		private static System.Collections.Generic.IDictionary<string, NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.ISerializer
			> serializers = null;

		private static NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer
			 instance = null;

		public static NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer 
			GetInstance()
		{
			lock (typeof(Serializer))
			{
				if (instance == null)
				{
					instance = new NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer
						();
				}
				return instance;
			}
		}

		private Serializer()
		{
			serializers = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.ISerializer
				>();
			serializers.Add(GetClassId(typeof(NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
				)), new NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.AtomicNativeObjectSerializer
				());
			serializers.Add(GetClassId(typeof(NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo
				)), new NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.CollectionObjectInfoSerializer
				());
		}

		public virtual string ToString(System.Collections.IList objectList)
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			for (int i = 0; i < objectList.Count; i++)
			{
				buffer.Append(ToString(objectList[i])).Append("\n");
			}
			return buffer.ToString();
		}

		public virtual string ToString(object @object)
		{
			string classId = GetClassId(@object.GetType());
			NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.ISerializer serializer = 
				serializers[classId];
			if (serializer != null)
			{
				return serializer.ToString(@object);
			}
			throw new System.Exception("toString not implemented for " + @object.GetType().FullName
				);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.ObjectContainer
			 FromString(string data)
		{
			NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.ObjectContainer container
				 = new NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.ObjectContainer();
			string[] lines = NeoDatis.Tool.Wrappers.OdbString.Split(data, "\n");
			for (int i = 0; i < lines.Length; i++)
			{
				if (lines[i] != null && lines[i].Trim().Length > 0)
				{
					container.Add(FromOneString(lines[i]));
				}
			}
			return container;
		}

		/// <exception cref="System.Exception"></exception>
		public virtual object FromOneString(string data)
		{
			int index = data.IndexOf(";");
			if (index == -1)
			{
				return null;
			}
			string type = NeoDatis.Tool.Wrappers.OdbString.Substring(data, 0, index);
			NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.ISerializer serializer = 
				serializers[type];
			if (serializer != null)
			{
				return serializer.FromString(data);
			}
			throw new System.Exception("fromString unimplemented for " + type);
		}

		public static string GetClassId(System.Type clazz)
		{
			if (clazz == typeof(NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo))
			{
				return "1";
			}
			if (clazz == typeof(NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo))
			{
				return "2";
			}
			return "0";
		}
	}
}
