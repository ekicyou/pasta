namespace NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization
{
	public class CollectionObjectInfoSerializer : NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.ISerializer
	{
		public static readonly string classId = NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer
			.GetClassId(typeof(NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo));

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
			string realCollectionName = tokens[1];
			int collectionSize = int.Parse(tokens[2]);
			string collectionData = tokens[3];
			string[] objects = NeoDatis.Tool.Wrappers.OdbString.Split(collectionData, NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer
				.CollectionElementSeparator);
			if (objects.Length != collectionSize)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.SerializationCollection
					.AddParameter(collectionSize).AddParameter(objects.Length));
			}
			System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
				> l = new System.Collections.Generic.List<NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
				>(collectionSize);
			for (int i = 0; i < collectionSize; i++)
			{
				l.Add((NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo)NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer
					.GetInstance().FromOneString(objects[i]));
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo coi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo
				(l);
			coi.SetRealCollectionClassName(realCollectionName);
			return coi;
		}

		public virtual string ToString(object @object)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo coi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo
				)@object;
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			// TODO escape ;
			buffer.Append(classId).Append(NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer
				.FieldSeparator);
			buffer.Append(coi.GetRealCollectionClassName()).Append(NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer
				.FieldSeparator);
			buffer.Append(coi.GetCollection().Count).Append(NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer
				.FieldSeparator);
			buffer.Append(NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer.
				CollectionStart);
			System.Collections.IEnumerator iterator = coi.GetCollection().GetEnumerator();
			while (iterator.MoveNext())
			{
				buffer.Append(NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer.
					GetInstance().ToString(iterator.Current));
				if (iterator.MoveNext())
				{
					buffer.Append(NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer.
						CollectionElementSeparator);
				}
			}
			buffer.Append(NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer.
				CollectionEnd);
			return buffer.ToString();
		}
	}
}
