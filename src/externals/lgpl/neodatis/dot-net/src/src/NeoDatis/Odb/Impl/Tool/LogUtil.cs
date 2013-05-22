namespace NeoDatis.Odb.Impl.Tool
{
	/// <summary>To manage logging level</summary>
	/// <author>osmadja</author>
	public class LogUtil
	{
		public static readonly string ObjectWriter = "ObjectWriter";

		public static readonly string ObjectReader = "ObjectReader";

		public static readonly string FileSystemInterface = "FileSystemInterface";

		public static readonly string IdManager = "IdManager";

		public static readonly string Transaction = "Transaction";

		public static readonly string BufferedIo = "BufferedIO";

		public static readonly string MultiBufferedIo = "MultiBufferedIO";

		public static readonly string WriteAction = "WriteAction";

		public static void ObjectWriterOn(bool yes)
		{
			if (yes)
			{
				NeoDatis.Odb.OdbConfiguration.AddLogId(ObjectWriter);
			}
			else
			{
				NeoDatis.Odb.OdbConfiguration.RemoveLogId(ObjectWriter);
			}
		}

		public static void ObjectReaderOn(bool yes)
		{
			if (yes)
			{
				NeoDatis.Odb.OdbConfiguration.AddLogId(ObjectReader);
			}
			else
			{
				NeoDatis.Odb.OdbConfiguration.RemoveLogId(ObjectReader);
			}
		}

		public static void FileSystemOn(bool yes)
		{
			if (yes)
			{
				NeoDatis.Odb.OdbConfiguration.AddLogId(FileSystemInterface);
			}
			else
			{
				NeoDatis.Odb.OdbConfiguration.RemoveLogId(FileSystemInterface);
			}
		}

		public static void IdManagerOn(bool yes)
		{
			if (yes)
			{
				NeoDatis.Odb.OdbConfiguration.AddLogId(IdManager);
			}
			else
			{
				NeoDatis.Odb.OdbConfiguration.RemoveLogId(IdManager);
			}
		}

		public static void TransactionOn(bool yes)
		{
			if (yes)
			{
				NeoDatis.Odb.OdbConfiguration.AddLogId(Transaction);
			}
			else
			{
				NeoDatis.Odb.OdbConfiguration.RemoveLogId(Transaction);
			}
		}

		public static void LogOn(string logId, bool yes)
		{
			if (yes)
			{
				NeoDatis.Odb.OdbConfiguration.AddLogId(logId);
			}
			else
			{
				NeoDatis.Odb.OdbConfiguration.RemoveLogId(logId);
			}
		}

		public static void AllOn(bool yes)
		{
			NeoDatis.Odb.OdbConfiguration.SetLogAll(yes);
		}

		public static void Enable(string logId)
		{
			NeoDatis.Odb.OdbConfiguration.AddLogId(logId);
		}
	}
}
