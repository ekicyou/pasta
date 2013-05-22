namespace NeoDatis.Odb.Impl.Main
{
	/// <summary>The Local ODB implementation.</summary>
	/// <remarks>The Local ODB implementation.</remarks>
	/// <author>osmadja</author>
	public class LocalODB : NeoDatis.Odb.Impl.Main.ODBAdapter
	{
		public static NeoDatis.Odb.Impl.Main.LocalODB GetInstance(string fileName)
		{
			return new NeoDatis.Odb.Impl.Main.LocalODB(fileName, null, null);
		}

		public static NeoDatis.Odb.Impl.Main.LocalODB GetInstance(string fileName, string
			 user, string password)
		{
			return new NeoDatis.Odb.Impl.Main.LocalODB(fileName, user, password);
		}

		/// <summary>protected Constructor with user and password</summary>
		/// <param name="fileName"></param>
		/// <param name="user"></param>
		/// <param name="password"></param>
		/// <exception cref="System.Exception">System.Exception</exception>
		protected LocalODB(string fileName, string user, string password) : base(NeoDatis.Odb.OdbConfiguration
			.GetCoreProvider().GetClientStorageEngine(new NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter
			(fileName, true, user, password)))
		{
		}
	}
}
