namespace NeoDatis.Tool
{
	/// <summary>Delete file function</summary>
	/// <author>osmadja</author>
	public class IOUtil
	{
		public static bool DeleteFile(string fileName)
		{
			NeoDatis.Tool.Wrappers.IO.OdbFile file = null;
			string dataDirectory = NeoDatis.Tool.Wrappers.OdbSystem.GetProperty("data.directory"
				);
			if (dataDirectory != null)
			{
				System.Text.StringBuilder buffer = new System.Text.StringBuilder(fileName.Length 
					+ 1 + dataDirectory.Length);
				buffer.Append(dataDirectory).Append("/").Append(fileName);
				file = new NeoDatis.Tool.Wrappers.IO.OdbFile(buffer.ToString());
			}
			else
			{
				file = new NeoDatis.Tool.Wrappers.IO.OdbFile(fileName);
			}
			bool deleted = file.Delete();
			return deleted;
		}

		public static bool ExistFile(string fileName)
		{
			NeoDatis.Tool.Wrappers.IO.OdbFile file = null;
			string dataDirectory = NeoDatis.Tool.Wrappers.OdbSystem.GetProperty("data.directory"
				);
			if (dataDirectory != null)
			{
				System.Text.StringBuilder buffer = new System.Text.StringBuilder(fileName.Length 
					+ 1 + dataDirectory.Length);
				buffer.Append(dataDirectory).Append("/").Append(fileName);
				file = new NeoDatis.Tool.Wrappers.IO.OdbFile(buffer.ToString());
			}
			else
			{
				file = new NeoDatis.Tool.Wrappers.IO.OdbFile(fileName);
			}
			return file.Exists();
		}
	}
}
