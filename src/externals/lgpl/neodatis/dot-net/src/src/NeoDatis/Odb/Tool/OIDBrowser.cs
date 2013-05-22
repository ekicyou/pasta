namespace NeoDatis.Odb.Tool
{
	public class OIDBrowser
	{
		public OIDBrowser() : base()
		{
		}

		// TODO Auto-generated constructor stub
		/// <exception cref="System.Exception"></exception>
		public static void Main(string[] args)
		{
			//LogUtil.allOn(true);
			NeoDatis.Odb.Configuration.SetCheckModelCompatibility(false);
			string fileName = "array1.odb";
			string user = "root";
			string password = "root";
			NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification parameter = new NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter
				(fileName, false);
			//IStorageEngine engine = StorageEngineFactory.get(parameter,user,password);
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = NeoDatis.Odb.Configuration
				.GetCoreProvider().GetClientStorageEngine(parameter, null, null);
			System.Collections.IList l = engine.GetAllObjectIdInfos(null, true);
			//"br.com.ccr.sct.dav.vo.RelFunctionProfile",true);
			NeoDatis.Tool.DisplayUtility.Display("All ids", l);
			engine.Close();
		}
	}
}
