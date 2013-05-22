namespace NeoDatis.Odb.Core
{
	/// <summary>Interface that will be dynamically added by byte code instrumentation.</summary>
	/// <remarks>
	/// Interface that will be dynamically added by byte code instrumentation. The name of the methods
	/// are out the standards to try to avoid method name collision.
	/// </remarks>
	/// <author>olivier</author>
	public interface IOdbObject
	{
		NeoDatis.Odb.OID __getOid__();

		void __setOid__(NeoDatis.Odb.OID oid);

		NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader __getOih__();

		void __setOih__(NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih);
	}
}
