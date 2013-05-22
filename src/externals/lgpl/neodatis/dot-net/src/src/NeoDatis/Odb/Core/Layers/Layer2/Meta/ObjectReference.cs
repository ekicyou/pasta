namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>Meta representation of an object reference.</summary>
	/// <remarks>Meta representation of an object reference.</remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class ObjectReference : NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
	{
		private NeoDatis.Odb.OID id;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi;

		public ObjectReference(NeoDatis.Odb.OID id) : base(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.NonNativeId)
		{
			this.id = id;
		}

		public ObjectReference(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi
			) : base(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NonNativeId)
		{
			this.id = null;
			this.nnoi = nnoi;
		}

		/// <returns>Returns the id.</returns>
		public virtual NeoDatis.Odb.OID GetOid()
		{
			if (nnoi != null)
			{
				return nnoi.GetOid();
			}
			return id;
		}

		public override bool IsObjectReference()
		{
			return true;
		}

		public override string ToString()
		{
			return "ObjectReference to oid " + GetOid();
		}

		public override bool IsNull()
		{
			return false;
		}

		public override object GetObject()
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.MethodShouldNotBeCalled
				.AddParameter("getObject").AddParameter(this.GetType().FullName));
		}

		public override void SetObject(object @object)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.MethodShouldNotBeCalled
				.AddParameter("setObject").AddParameter(this.GetType().FullName));
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo GetNnoi()
		{
			return nnoi;
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo CreateCopy
			(System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			> cache, bool onlyData)
		{
			return new NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectReference((NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)nnoi.CreateCopy(cache, onlyData));
		}
	}
}
