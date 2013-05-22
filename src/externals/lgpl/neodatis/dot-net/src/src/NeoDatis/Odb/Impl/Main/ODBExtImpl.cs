namespace NeoDatis.Odb.Impl.Main
{
	public class ODBExtImpl : NeoDatis.Odb.ODBExt
	{
		protected NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine;

		public ODBExtImpl(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine)
		{
			this.engine = storageEngine;
		}

		public virtual NeoDatis.Odb.ExternalOID ConvertToExternalOID(NeoDatis.Odb.OID oid
			)
		{
			return new NeoDatis.Odb.Impl.Core.Oid.ExternalObjectOID(oid, engine.GetDatabaseId
				());
		}

		public virtual NeoDatis.Odb.TransactionId GetCurrentTransactionId()
		{
			return engine.GetCurrentTransactionId();
		}

		public virtual NeoDatis.Odb.DatabaseId GetDatabaseId()
		{
			return engine.GetDatabaseId();
		}

		public virtual NeoDatis.Odb.ExternalOID GetObjectExternalOID(object @object)
		{
			return ConvertToExternalOID(engine.GetObjectId(@object, true));
		}

		public virtual int GetObjectVersion(NeoDatis.Odb.OID oid)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = engine.GetObjectInfoHeaderFromOid
				(oid);
			if (oih == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ObjectWithOidDoesNotExistInCache
					.AddParameter(oid));
			}
			return oih.GetObjectVersion();
		}

		public virtual long GetObjectCreationDate(NeoDatis.Odb.OID oid)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = engine.GetObjectInfoHeaderFromOid
				(oid);
			if (oih == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ObjectWithOidDoesNotExistInCache
					.AddParameter(oid));
			}
			return oih.GetCreationDate();
		}

		public virtual long GetObjectUpdateDate(NeoDatis.Odb.OID oid)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = engine.GetObjectInfoHeaderFromOid
				(oid);
			if (oih == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ObjectWithOidDoesNotExistInCache
					.AddParameter(oid));
			}
			return oih.GetUpdateDate();
		}
	}
}
