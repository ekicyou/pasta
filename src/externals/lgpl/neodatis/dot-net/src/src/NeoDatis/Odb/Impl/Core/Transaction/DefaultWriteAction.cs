namespace NeoDatis.Odb.Impl.Core.Transaction
{
	/// <summary>The WriteAction class is the description of a Write operation that will be applied to the main database file when committing.
	/// 	</summary>
	/// <remarks>
	/// The WriteAction class is the description of a Write operation that will be applied to the main database file when committing.
	/// All operations(writes) that can not be written to the database file before committing , pointers (for example) are stored in WriteAction
	/// objects. The transaction keeps track of all these WriteActions. When committing, the transaction apply each WriteAction to the engine database file.
	/// </remarks>
	/// <author>osmadja</author>
	public class DefaultWriteAction : NeoDatis.Odb.Core.Transaction.IWriteAction
	{
		public static int count = 0;

		public const int UnknownWriteAction = 0;

		public const int DataWriteAction = 1;

		public const int PointerWriteAction = 2;

		public const int DirectWriteAction = 3;

		public static readonly string LogId = "WriteAction";

		private static string UnknownLabel = "?";

		private long position;

		private NeoDatis.Odb.Core.Layers.Layer3.Engine.IByteArrayConverter byteArrayConverter;

		private NeoDatis.Tool.Wrappers.List.IOdbList<byte[]> listOfBytes;

		private int size;

		public DefaultWriteAction(long position) : this(position, null)
		{
		}

		public DefaultWriteAction(long position, byte[] bytes) : this(position, bytes, null
			)
		{
		}

		public DefaultWriteAction(long position, byte[] bytes, string label)
		{
			this.byteArrayConverter = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetByteArrayConverter
				();
			this.position = position;
			//TODO:perf should init with no default size?
			listOfBytes = new NeoDatis.Tool.Wrappers.List.OdbArrayList<byte[]>(20);
			if (bytes != null)
			{
				listOfBytes.Add(bytes);
				this.size = bytes.Length;
			}
		}

		public virtual long GetPosition()
		{
			return position;
		}

		public virtual void SetPosition(long position)
		{
			this.position = position;
		}

		public virtual byte[] GetBytes(int index)
		{
			return listOfBytes[index];
		}

		public virtual void AddBytes(byte[] bytes)
		{
			listOfBytes.Add(bytes);
			size += bytes.Length;
		}

		public virtual void Persist(NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface
			 fsi, int index)
		{
			long currentPosition = fsi.GetPosition();
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
			}
			// DLogger.debug("# Writing WriteAction #" + index + " at " +
			// currentPosition+" : " + toString());
			int sizeOfLong = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize();
			int sizeOfInt = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Integer.GetSize();
			// build the full byte array to write once
			byte[] bytes = new byte[sizeOfLong + sizeOfInt + size];
			byte[] bytesOfPosition = byteArrayConverter.LongToByteArray(position);
			byte[] bytesOfSize = byteArrayConverter.IntToByteArray(size);
			for (int i = 0; i < sizeOfLong; i++)
			{
				bytes[i] = bytesOfPosition[i];
			}
			int offset = sizeOfLong;
			for (int i = 0; i < sizeOfInt; i++)
			{
				bytes[offset] = bytesOfSize[i];
				offset++;
			}
			for (int i = 0; i < listOfBytes.Count; i++)
			{
				byte[] tmp = listOfBytes[i];
				System.Array.Copy(tmp, 0, bytes, offset, tmp.Length);
				offset += tmp.Length;
			}
			fsi.WriteBytes(bytes, false, "Transaction");
			int fixedSize = sizeOfLong + sizeOfInt;
			long positionAfterWrite = fsi.GetPosition();
			long writeSize = positionAfterWrite - currentPosition;
			if (writeSize != size + fixedSize)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.DifferentSizeInWriteAction
					.AddParameter(size).AddParameter(writeSize));
			}
		}

		public static NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction Read(NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface
			 fsi, int index)
		{
			try
			{
				long position = fsi.ReadLong();
				int size = fsi.ReadInt();
				byte[] bytes = fsi.ReadBytes(size);
				NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction writeAction = new NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction
					(position, bytes);
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug("Loading Write Action # " + index + " at " + fsi.GetPosition
						() + " => " + writeAction.ToString());
				}
				return writeAction;
			}
			catch (NeoDatis.Odb.ODBRuntimeException e)
			{
				NeoDatis.Tool.DLogger.Error("error reading write action " + index + " at position "
					 + fsi.GetPosition());
				throw;
			}
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append("position=").Append(position);
			System.Text.StringBuilder bytes = new System.Text.StringBuilder();
			if (listOfBytes != null)
			{
				for (int i = 0; i < listOfBytes.Count; i++)
				{
					bytes.Append(NeoDatis.Tool.DisplayUtility.ByteArrayToString(GetBytes(i)));
				}
				buffer.Append(" | bytes=[").Append(bytes).Append("] & size=" + size);
			}
			else
			{
				buffer.Append(" | bytes=null & size=").Append(size);
			}
			return buffer.ToString();
		}

		public virtual void ApplyTo(NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface
			 fsi, int index)
		{
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("Applying WriteAction #" + index + " : " + ToString()
					);
			}
			fsi.SetWritePosition(position, false);
			for (int i = 0; i < listOfBytes.Count; i++)
			{
				fsi.WriteBytes(GetBytes(i), false, "WriteAction");
			}
		}

		public virtual bool IsEmpty()
		{
			return listOfBytes == null || listOfBytes.IsEmpty();
		}

		public virtual void Clear()
		{
			listOfBytes.Clear();
			listOfBytes = null;
			NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction.count--;
		}
	}
}
