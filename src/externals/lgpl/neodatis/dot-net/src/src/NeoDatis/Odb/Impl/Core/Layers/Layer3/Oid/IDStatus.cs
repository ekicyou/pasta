namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid
{
	/// <summary>Status of  ID.</summary>
	/// <remarks>Status of  ID.</remarks>
	/// <author>osmadja</author>
	public class IDStatus
	{
		public const byte Unknown = 0;

		public const byte Active = 1;

		public const byte Deleted = 2;

		public static bool IsActive(byte status)
		{
			return status == Active;
		}
	}
}
