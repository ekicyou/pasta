namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Buffer
{
	/// <summary>A buffer manager that can manage more than one buffer.</summary>
	/// <remarks>
	/// A buffer manager that can manage more than one buffer. Number of buffers can
	/// be configured using Configuration.setNbBuffers().
	/// </remarks>
	/// <author>osmadja</author>
	public class MultiBufferedFileIO : NeoDatis.Odb.Impl.Core.Layers.Layer3.Buffer.MultiBufferedIO
	{
		private static readonly string LogId = "MultiBufferedFileIO";

		private NeoDatis.Tool.Wrappers.IO.OdbFileIO fileWriter;

		public static int nbcalls = 0;

		public static int nbdiffcalls = 0;

		private string wholeFileName;

		public MultiBufferedFileIO(int nbBuffers, string name, string fileName, bool canWrite
			, int bufferSize) : base(nbBuffers, name, bufferSize, canWrite)
		{
			Init(fileName, canWrite);
		}

		private void Init(string fileName, bool canWrite)
		{
			string dataDirectory = NeoDatis.Tool.Wrappers.OdbSystem.GetProperty("data.directory"
				);
			if (dataDirectory != null)
			{
				wholeFileName = dataDirectory + "/" + fileName;
			}
			else
			{
				wholeFileName = fileName;
			}
			try
			{
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Info("Opening datatbase file : " + new NeoDatis.Tool.Wrappers.IO.OdbFile
						(wholeFileName).GetFullPath());
				}
				fileWriter = BuildFileWriter(canWrite);
				SetIoDeviceLength(fileWriter.Length());
			}
			catch (System.Exception e)
			{
				//fixme
				new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
					, e);
			}
			if (canWrite)
			{
				try
				{
					fileWriter.LockFile();
				}
				catch (System.Exception)
				{
					// The file region is already locked
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbFileIsLockedByCurrentVirtualMachine
						.AddParameter(wholeFileName).AddParameter(NeoDatis.Tool.Wrappers.OdbThread.GetCurrentThreadName
						()).AddParameter(NeoDatis.Odb.OdbConfiguration.IsMultiThread().ToString()));
				}
				if (!fileWriter.IsLocked())
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbFileIsLockedByExternalProgram
						.AddParameter(wholeFileName).AddParameter(NeoDatis.Tool.Wrappers.OdbThread.GetCurrentThreadName
						()).AddParameter(NeoDatis.Odb.OdbConfiguration.IsMultiThread().ToString()));
				}
			}
		}

		/// <exception cref="System.IO.IOException"></exception>
		protected virtual NeoDatis.Tool.Wrappers.IO.OdbFileIO BuildFileWriter(bool canWrite
			)
		{
			return new NeoDatis.Tool.Wrappers.IO.OdbFileIO(wholeFileName, canWrite, null);
		}

		public override void GoToPosition(long position)
		{
			try
			{
				if (position < 0)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NegativePosition
						.AddParameter(position));
				}
				fileWriter.Seek(position);
			}
			catch (System.IO.IOException e)
			{
				long l = -1;
				try
				{
					l = fileWriter.Length();
				}
				catch (System.IO.IOException)
				{
				}
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.GoToPosition
					.AddParameter(position).AddParameter(l), e);
			}
		}

		public override long GetLength()
		{
			nbcalls++;
			return GetIoDeviceLength();
		}

		public override void InternalWrite(byte b)
		{
			try
			{
				fileWriter.Write(b);
			}
			catch (System.IO.IOException e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(e, "Error while writing a byte");
			}
		}

		public override void InternalWrite(byte[] bs, int size)
		{
			try
			{
				fileWriter.Write(bs, 0, size);
			}
			catch (System.IO.IOException e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(e, "Error while writing an array of byte"
					);
			}
		}

		public override byte InternalRead()
		{
			int b;
			try
			{
				b = fileWriter.Read();
				if (b == -1)
				{
					throw new System.IO.IOException("Enf of file");
				}
				return (byte)b;
			}
			catch (System.IO.IOException e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(e, "Error while reading a byte");
			}
		}

		public override long InternalRead(byte[] array, int size)
		{
			// FIXME raf.read only returns int not long
			try
			{
				return fileWriter.Read(array, 0, size);
			}
			catch (System.IO.IOException e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(e, "Error while reading an array of byte"
					);
			}
		}

		public override void CloseIO()
		{
			try
			{
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug("Closing file with size " + fileWriter.Length());
				}
				// Problem found by mayworm : necessary for MacOSX
				if (fileWriter.IsLocked())
				{
					fileWriter.UnlockFile();
				}
				fileWriter.Close();
			}
			catch (System.IO.IOException e)
			{
				NeoDatis.Tool.DLogger.Error(NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, 
					true));
			}
			fileWriter = null;
			if (IsForTransaction() && AutomaticDeleteIsEnabled())
			{
				bool b = NeoDatis.Tool.IOUtil.DeleteFile(wholeFileName);
				if (!b)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.CanNotDeleteFile
						.AddParameter(wholeFileName));
				}
			}
		}

		// The file lock is automatically released closing the raf object
		public override void Clear()
		{
			base.Clear();
		}

		public override bool Delete()
		{
			return NeoDatis.Tool.IOUtil.DeleteFile(wholeFileName);
		}
	}
}
