namespace NeoDatis.Odb.Impl.Core.Transaction
{
	/// <summary>A specific cache for the server in Client/Server mode</summary>
	/// <author>osmadja</author>
	public class ServerCache : NeoDatis.Odb.Impl.Core.Transaction.Cache
	{
		/// <summary>Object id of NonNativeObjectInfo</summary>
		/// <TODO>check why we need this, there is no getter for it =&gt; we don't use it</TODO>
		protected System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			> oidsOfNNoi;

		public ServerCache(NeoDatis.Odb.Core.Transaction.ISession session) : base(session
			, "server")
		{
			oidsOfNNoi = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<NeoDatis.Odb.OID, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				>();
		}

		public virtual void AddOid(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi)
		{
			oidsOfNNoi.Add(oid, nnoi);
		}

		public override void StartInsertingObjectWithOid(object @object, NeoDatis.Odb.OID
			 oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi)
		{
			base.StartInsertingObjectWithOid(@object, oid, nnoi);
			AddOid(oid, nnoi);
		}

		public override void Clear(bool setToNull)
		{
			base.Clear(setToNull);
			oidsOfNNoi.Clear();
			if (setToNull)
			{
				oidsOfNNoi = null;
			}
		}

		protected override bool CheckHeaderPosition()
		{
			return true;
		}
	}
}
