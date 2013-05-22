namespace NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization
{
	public class NativeObjectSerializer : NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.ISerializer
	{
		public static readonly string classId = NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer
			.GetClassId(typeof(NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo));

		/// <exception cref="System.Exception"></exception>
		public virtual object FromString(string data)
		{
			string[] tokens = NeoDatis.Tool.Wrappers.OdbString.Split(data, NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer
				.FieldSeparator);
			if (!tokens[0].Equals(classId))
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.SerializationFromString
					.AddParameter(classId).AddParameter(tokens[0]));
			}
			int odbTypeId = int.Parse(tokens[1]);
			object o = NeoDatis.Odb.Impl.Tool.ObjectTool.StringToObjectInfo(odbTypeId, tokens
				[2], NeoDatis.Odb.Impl.Tool.ObjectTool.IdCallerIsSerializer, null);
			return o;
		}

		public virtual string ToString(object @object)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo anoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
				)@object;
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			// TODO escape ;
			buffer.Append(classId).Append(NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer
				.FieldSeparator);
			buffer.Append(anoi.GetOdbTypeId()).Append(NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer
				.FieldSeparator);
			buffer.Append(NeoDatis.Odb.Impl.Tool.ObjectTool.AtomicNativeObjectToString(anoi, 
				NeoDatis.Odb.Impl.Tool.ObjectTool.IdCallerIsSerializer));
			return buffer.ToString();
		}
	}
}
