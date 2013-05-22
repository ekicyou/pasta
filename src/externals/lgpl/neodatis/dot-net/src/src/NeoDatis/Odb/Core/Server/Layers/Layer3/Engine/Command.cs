namespace NeoDatis.Odb.Core.Server.Layers.Layer3.Engine
{
	[System.Serializable]
	public class Command
	{
		public const int Connect = 1;

		public const int Get = 2;

		public const int GetObjectFromId = 3;

		public const int Store = 4;

		public const int DeleteObject = 5;

		public const int Close = 6;

		public const int Commit = 7;

		public const int Rollback = 8;

		public const int DeleteBase = 9;

		public const int GetSessions = 10;

		public const int AddUniqueIndex = 11;

		public const int AddClassInfoList = 12;

		public const int Count = 13;

		public const int GetObjectValues = 14;

		public const int GetObjectHeaderFromId = 15;

		public const int RebuildIndex = 16;

		public const int DeleteIndex = 17;

		public const int CheckMetaModelCompatibility = 18;

		public static readonly int[] commands = new int[] { Connect, Get, GetObjectFromId
			, Store, DeleteObject, Close, Commit, Rollback, DeleteBase, GetSessions, AddUniqueIndex
			, AddClassInfoList, Count, GetObjectValues, GetObjectHeaderFromId, RebuildIndex, 
			DeleteIndex, CheckMetaModelCompatibility };
	}
}
