namespace NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare
{
	/// <summary>Used to store informations about object changes</summary>
	public class ChangedObjectInfo
	{
		private NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo oldCi;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo newCi;

		private int fieldIndex;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo oldValue;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo newValue;

		private string message;

		private int objectRecursionLevel;

		public ChangedObjectInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo oldCi, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 newCi, int fieldIndex, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo 
			oldValue, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo newValue, int 
			objectRecursionLevel) : this(oldCi, newCi, fieldIndex, oldValue, newValue, null, 
			objectRecursionLevel)
		{
		}

		public ChangedObjectInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo oldCi, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 newCi, int fieldIndex, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo 
			oldValue, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo newValue, string
			 message, int objectRecursionLevel) : base()
		{
			this.oldCi = oldCi;
			this.newCi = newCi;
			this.fieldIndex = fieldIndex;
			this.oldValue = oldValue;
			this.newValue = newValue;
			this.message = message;
			this.objectRecursionLevel = objectRecursionLevel;
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			if (message != null)
			{
				buffer.Append(message).Append(" | ");
			}
			if (oldCi.GetId() != newCi.GetId())
			{
				buffer.Append("old class=").Append(oldCi.GetFullClassName()).Append(" | new class="
					).Append(newCi.GetFullClassName());
			}
			else
			{
				buffer.Append("class=").Append(oldCi.GetFullClassName());
			}
			buffer.Append(" | field=").Append(oldCi.GetAttributeInfo(fieldIndex).GetName());
			buffer.Append(" | old=").Append(oldValue.ToString()).Append(" | new=").Append(newValue
				.ToString());
			buffer.Append(" | obj. hier. level=").Append(objectRecursionLevel);
			return buffer.ToString();
		}
	}
}
