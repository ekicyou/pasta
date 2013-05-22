namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine
{
	/// <summary>A mutex to logically lock ODB database file</summary>
	/// <author>osmadja</author>
	public class FileMutex
	{
		private static NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.FileMutex instance = new 
			NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.FileMutex();

		private System.Collections.Generic.IDictionary<string, string> openFiles;

		private FileMutex()
		{
			openFiles = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, string>();
		}

		public static NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.FileMutex GetInstance()
		{
			lock (typeof(FileMutex))
			{
				return instance;
			}
		}

		public virtual void ReleaseFile(string fileName)
		{
			lock (openFiles)
			{
				openFiles.Remove(fileName);
			}
		}

		public virtual void LockFile(string fileName)
		{
			lock (openFiles)
			{
				openFiles.Add(fileName, fileName);
			}
		}

		private bool CanOpenFile(string fileName)
		{
			lock (openFiles)
			{
                string f = null;
                openFiles.TryGetValue(fileName,out f);
                bool canOpen = f == null;
				if (canOpen)
				{
					LockFile(fileName);
				}
				return canOpen;
			}
		}

		public virtual bool OpenFile(string fileName)
		{
			bool canOpenfile = CanOpenFile(fileName);
			if (!canOpenfile)
			{
				if (NeoDatis.Odb.OdbConfiguration.RetryIfFileIsLocked())
				{
					int nbRetry = 0;
					while (!CanOpenFile(fileName) && nbRetry < NeoDatis.Odb.OdbConfiguration.GetNumberOfRetryToOpenFile
						())
					{
						try
						{
							NeoDatis.Tool.Wrappers.OdbThread.Sleep(NeoDatis.Odb.OdbConfiguration.GetRetryTimeout
								());
						}
						catch (System.Exception)
						{
						}
						// nothing to do
						nbRetry++;
					}
					if (nbRetry < NeoDatis.Odb.OdbConfiguration.GetNumberOfRetryToOpenFile())
					{
						return true;
					}
				}
				return false;
			}
			return true;
		}
	}
}
