namespace NeoDatis.Odb
{
	/// <author>olivier</author>
	public class OIDTypes
	{
		public const int TypeClassOid = 1;

		public const int TypeObjectOid = 2;

		public const int TypeNativeOid = 3;

		public const int TypeExternalClassOid = 4;

		public const int TypeExternalObjectOid = 5;

		public static readonly string TypeNameUnknow = "unknown";

		public static readonly string TypeNameClassOid = "class-oid";

		public static readonly string TypeNameObjectOid = "object-oid";

		public static readonly string TypeNameNativeOid = "native-oid";

		public static readonly string TypeNameExternalClassOid = "ext-class-oid";

		public static readonly string TypeNameExternalObjectOid = "ext-object-oid";

		public static readonly string[] names = new string[] { TypeNameClassOid, TypeNameObjectOid
			, TypeNameNativeOid, TypeNameExternalClassOid, TypeNameExternalObjectOid };

		public static string GetTypeName(int type)
		{
			return names[type];
		}
	}
}
