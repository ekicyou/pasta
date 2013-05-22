namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine
{
	/// <summary>Undocumented class</summary>
	/// <author>osmadja</author>
	public class Dummy
	{
		public static NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine GetEngine(NeoDatis.Odb.ODB
			 odb)
		{
			if (odb is NeoDatis.Odb.Impl.Main.ODBAdapter)
			{
				NeoDatis.Odb.Impl.Main.ODBAdapter oa = (NeoDatis.Odb.Impl.Main.ODBAdapter)odb;
				return oa.GetSession().GetStorageEngine();
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
				.AddParameter("getEngine not implemented for " + odb.GetType().FullName));
		}

		public static NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo GetNnoi(NeoDatis.Odb.ObjectRepresentation
			 objectRepresentation)
		{
			if (objectRepresentation is NeoDatis.Odb.Impl.Core.Server.Trigger.DefaultObjectRepresentation)
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = ((NeoDatis.Odb.Impl.Core.Server.Trigger.DefaultObjectRepresentation
					)objectRepresentation).GetNnoi();
				return nnoi;
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
				.AddParameter("getNnoi not implemented for " + objectRepresentation.GetType().FullName
				));
		}
	}
}
