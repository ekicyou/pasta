namespace NeoDatis.Odb.Core.Layers.Layer3.Engine
{
	/// <summary>
	/// Class that knows how to read/write all language native types : byte, char,
	/// String, int, long,....
	/// </summary>
	/// <remarks>
	/// Class that knows how to read/write all language native types : byte, char,
	/// String, int, long,....
	/// </remarks>
	/// <author>osmadja</author>
	public abstract class FileSystemInterface : NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface
	{
		public virtual void SetDatabaseCharacterEncoding(string databaseCharacterEncoding
			)
		{
			byteArrayConverter.SetDatabaseCharacterEncoding(databaseCharacterEncoding);
		}

		private static readonly int IntSize = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Integer.GetSize();

		private static readonly int INT_SIZE_x_2 = IntSize * 2;

		public static int nbCall1;

		public static int nbCall2;

		public static readonly string LogId = "FileSystemInterface";

		private string name;

		private bool canLog;

		private NeoDatis.Odb.Core.Layers.Layer3.IBufferedIO io;

		protected NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification parameters;

		protected NeoDatis.Odb.Core.Layers.Layer3.Engine.IByteArrayConverter byteArrayConverter;

		private const byte ReservedSpace = (byte)128;

		public FileSystemInterface(string name, string fileName, bool canWrite, bool canLog
			, int bufferSize) : this(name, new NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter
			(fileName, canWrite, null, null), canLog, bufferSize)
		{
		}

		public FileSystemInterface(string name, NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification
			 parameters, bool canLog, int bufferSize)
		{
			this.name = name;
			this.parameters = parameters;
			this.canLog = canLog;
			NeoDatis.Odb.Core.ICoreProvider provider = NeoDatis.Odb.OdbConfiguration.GetCoreProvider
				();
			this.io = provider.GetIO(name, parameters, bufferSize);
			this.byteArrayConverter = provider.GetByteArrayConverter();
		}

		public abstract NeoDatis.Odb.Core.Transaction.ISession GetSession();

		public virtual void UseBuffer(bool useBuffer)
		{
			io.SetUseBuffer(useBuffer);
		}

		public virtual void Flush()
		{
			io.FlushAll();
		}

		public virtual long GetPosition()
		{
			return io.GetCurrentPosition();
		}

		public virtual long GetLength()
		{
			return io.GetLength();
		}

		/// <summary>
		/// Writing at position &lt; DATABASE_HEADER_PROTECTED_ZONE_SIZE is writing in
		/// ODB Header place.
		/// </summary>
		/// <remarks>
		/// Writing at position &lt; DATABASE_HEADER_PROTECTED_ZONE_SIZE is writing in
		/// ODB Header place. Here we check the positions where the writing is done.
		/// Search for 'page format' in ODB wiki to understand the positions
		/// </remarks>
		/// <param name="position"></param>
		/// <returns></returns>
		internal virtual bool IsWritingInWrongPlace(long position)
		{
			if (position < NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.
				DatabaseHeaderProtectedZoneSize)
			{
				int size = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.DatabaseHeaderPositions
					.Length;
				for (int i = 0; i < size; i++)
				{
					if (position == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
						.DatabaseHeaderPositions[i])
					{
						return false;
					}
				}
				return true;
			}
			return false;
		}

		public virtual void SetWritePositionNoVerification(long position, bool writeInTransacation
			)
		{
			io.SetCurrentWritePosition(position);
			if (writeInTransacation)
			{
				GetSession().GetTransaction().SetWritePosition(position);
			}
		}

		public virtual void SetWritePosition(long position, bool writeInTransacation)
		{
			if (position < NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.
				DatabaseHeaderProtectedZoneSize)
			{
				if (IsWritingInWrongPlace(position))
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
						.AddParameter("Trying to write in Protected area at position " + position));
				}
			}
			io.SetCurrentWritePosition(position);
			if (writeInTransacation)
			{
				GetSession().GetTransaction().SetWritePosition(position);
			}
		}

		public virtual void SetReadPosition(long position)
		{
			io.SetCurrentReadPosition(position);
		}

		public virtual long GetAvailablePosition()
		{
			return io.GetLength();
		}

		private bool PointerAtTheEndOfTheFile()
		{
			return io.GetCurrentPosition() == io.GetLength();
		}

		/// <summary>
		/// Reserve space in the file when it is at the end of the file Used in
		/// transaction mode where real write will happen later
		/// </summary>
		/// <param name="quantity">The number of object to reserve space for</param>
		/// <param name="type">The type of the object to reserve space for</param>
		private void EnsureSpaceFor(long quantity, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			 type)
		{
			long space = type.GetSize() * quantity;
			// We are in transaction mode - do not write just reserve space if
			// necessary
			// ensure space will be available when applying transaction
			if (PointerAtTheEndOfTheFile())
			{
				if (space != 1)
				{
					io.SetCurrentWritePosition(io.GetCurrentPosition() + space - 1);
				}
				io.WriteByte(ReservedSpace);
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog)
				{
				}
			}
			else
			{
				// DLogger.debug("Reserving " + space + " bytes (" + quantity +
				// " " + type.getName() + ")");
				// We must simulate the move
				io.SetCurrentWritePosition(io.GetCurrentPosition() + space);
			}
		}

		public virtual void EnsureSpaceFor(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType type
			)
		{
			EnsureSpaceFor(1, type);
		}

		public virtual void WriteByte(byte i, bool writeInTransaction)
		{
			WriteByte(i, writeInTransaction, null);
		}

		public virtual void WriteByte(byte i, bool writeInTransaction, string label)
		{
			byte[] bytes = new byte[] { i };
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog)
			{
				NeoDatis.Tool.DLogger.Debug("writing byte " + i + " at " + GetPosition() + (label
					 != null ? " : " + label : string.Empty));
			}
			if (!writeInTransaction)
			{
				io.WriteByte(i);
			}
			else
			{
				GetSession().GetTransaction().ManageWriteAction(io.GetCurrentPosition(), bytes);
				EnsureSpaceFor(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeByte);
			}
		}

		public virtual byte ReadByte()
		{
			return ReadByte(null);
		}

		public virtual byte ReadByte(string label)
		{
			long position = io.GetCurrentPosition();
			byte i = io.ReadByte();
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog)
			{
				NeoDatis.Tool.DLogger.Debug("reading byte " + i + " at " + position + (label != null
					 ? " : " + label : string.Empty));
			}
			return i;
		}

		public virtual void WriteBytes(byte[] bytes, bool writeInTransaction, string label
			)
		{
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog)
			{
				NeoDatis.Tool.DLogger.Debug("writing " + bytes.Length + " bytes at " + GetPosition
					() + (label != null ? " : " + label : string.Empty) + " = " + NeoDatis.Tool.DisplayUtility
					.ByteArrayToString(bytes));
			}
			if (!writeInTransaction)
			{
				io.WriteBytes(bytes);
			}
			else
			{
				GetSession().GetTransaction().ManageWriteAction(io.GetCurrentPosition(), bytes);
				EnsureSpaceFor(bytes.Length, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeByte
					);
			}
			bytes = null;
		}

		public virtual byte[] ReadBytes(int length)
		{
			long position = io.GetCurrentPosition();
			byte[] bytes = io.ReadBytes(length);
			int byteCount = bytes.Length;
			if (byteCount != length)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.FileInterfaceReadError
					.AddParameter(length).AddParameter(position).AddParameter(byteCount));
			}
			return bytes;
		}

		public virtual void WriteChar(char c, bool writeInTransaction)
		{
			byte[] bytes = byteArrayConverter.CharToByteArray(c);
			if (!writeInTransaction)
			{
				io.WriteBytes(bytes);
			}
			else
			{
				GetSession().GetTransaction().ManageWriteAction(io.GetCurrentPosition(), bytes);
				EnsureSpaceFor(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeChar);
			}
			bytes = null;
		}

		public virtual byte[] ReadCharBytes()
		{
			return io.ReadBytes(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Character.GetSize
				());
		}

		public virtual char ReadChar()
		{
			return ReadChar(null);
		}

		public virtual char ReadChar(string label)
		{
			long position = io.GetCurrentPosition();
			char c = byteArrayConverter.ByteArrayToChar(ReadCharBytes());
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog && label != null)
			{
				NeoDatis.Tool.DLogger.Debug("reading char " + c + " at " + position + " : " + label
					);
			}
			return c;
		}

		public virtual void WriteShort(short s, bool writeInTransaction)
		{
			byte[] bytes = byteArrayConverter.ShortToByteArray(s);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog)
			{
				NeoDatis.Tool.DLogger.Debug("writing short " + s + " at " + GetPosition());
			}
			if (!writeInTransaction)
			{
				io.WriteBytes(bytes);
			}
			else
			{
				GetSession().GetTransaction().ManageWriteAction(io.GetCurrentPosition(), bytes);
				EnsureSpaceFor(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeShort);
			}
			bytes = null;
		}

		public virtual byte[] ReadShortBytes()
		{
			return io.ReadBytes(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeShort.GetSize
				());
		}

		public virtual short ReadShort()
		{
			return ReadShort(null);
		}

		public virtual short ReadShort(string label)
		{
			long position = io.GetCurrentPosition();
			short s = byteArrayConverter.ByteArrayToShort(ReadShortBytes());
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog && label != null)
			{
				NeoDatis.Tool.DLogger.Debug("reading short " + s + " at " + position + " : " + label
					);
			}
			return s;
		}

		public virtual void WriteInt(int i, bool writeInTransaction, string label)
		{
			byte[] bytes = byteArrayConverter.IntToByteArray(i);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog)
			{
				NeoDatis.Tool.DLogger.Debug("writing int " + i + " at " + GetPosition() + " : " +
					 label);
			}
			if (!writeInTransaction)
			{
				io.WriteBytes(bytes);
			}
			else
			{
				GetSession().GetTransaction().ManageWriteAction(io.GetCurrentPosition(), bytes);
				EnsureSpaceFor(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeInt);
			}
			bytes = null;
		}

		public virtual byte[] ReadIntBytes()
		{
			return io.ReadBytes(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Integer.GetSize(
				));
		}

		public virtual int ReadInt()
		{
			return ReadInt(null);
		}

		public virtual int ReadInt(string label)
		{
			long position = io.GetCurrentPosition();
			int i = byteArrayConverter.ByteArrayToInt(ReadIntBytes(), 0);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog)
			{
				NeoDatis.Tool.DLogger.Debug("reading int " + i + " at " + position + (label != null
					 ? " : " + label : string.Empty));
			}
			return i;
		}

		public virtual void WriteLong(long i, bool writeInTransaction, string label, int 
			writeActionType)
		{
			byte[] bytes = byteArrayConverter.LongToByteArray(i);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog && label != null)
			{
				NeoDatis.Tool.DLogger.Debug("writing long " + i + " at " + GetPosition() + " : " 
					+ label);
			}
			if (!writeInTransaction)
			{
				io.WriteBytes(bytes);
			}
			else
			{
				GetSession().GetTransaction().ManageWriteAction(io.GetCurrentPosition(), bytes);
				EnsureSpaceFor(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeLong);
			}
			bytes = null;
		}

		public virtual byte[] ReadLongBytes()
		{
			return io.ReadBytes(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long.GetSize());
		}

		public virtual long ReadLong()
		{
			return ReadLong(null);
		}

		public virtual long ReadLong(string label)
		{
			long position = io.GetCurrentPosition();
			long l = byteArrayConverter.ByteArrayToLong(ReadLongBytes(), 0);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog)
			{
				NeoDatis.Tool.DLogger.Debug("reading long " + l + " at " + position + (label != null
					 ? " : " + label : string.Empty));
			}
			return l;
		}

		public virtual void WriteFloat(float f, bool writeInTransaction)
		{
			byte[] bytes = byteArrayConverter.FloatToByteArray(f);
			if (!writeInTransaction)
			{
				io.WriteBytes(bytes);
			}
			else
			{
				GetSession().GetTransaction().ManageWriteAction(io.GetCurrentPosition(), bytes);
				EnsureSpaceFor(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeFloat);
			}
			bytes = null;
		}

		public virtual byte[] ReadFloatBytes()
		{
			return io.ReadBytes(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Float.GetSize());
		}

		public virtual float ReadFloat()
		{
			return ReadFloat(null);
		}

		public virtual float ReadFloat(string label)
		{
			long position = io.GetCurrentPosition();
			float f = byteArrayConverter.ByteArrayToFloat(ReadFloatBytes());
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog)
			{
				NeoDatis.Tool.DLogger.Debug("Reading float '" + f + "' at " + position + (label !=
					 null ? " : " + label : string.Empty));
			}
			return f;
		}

		public virtual void WriteDouble(double d, bool writeInTransaction)
		{
			byte[] bytes = byteArrayConverter.DoubleToByteArray(d);
			if (!writeInTransaction)
			{
				io.WriteBytes(bytes);
			}
			else
			{
				GetSession().GetTransaction().ManageWriteAction(io.GetCurrentPosition(), bytes);
				EnsureSpaceFor(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeDouble);
			}
			bytes = null;
		}

		public virtual byte[] ReadDoubleBytes()
		{
			return io.ReadBytes(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Double.GetSize()
				);
		}

		public virtual double ReadDouble()
		{
			return ReadDouble(null);
		}

		public virtual double ReadDouble(string label)
		{
			long position = io.GetCurrentPosition();
			double d = byteArrayConverter.ByteArrayToDouble(ReadDoubleBytes());
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog)
			{
				NeoDatis.Tool.DLogger.Debug("Reading double '" + d + "' at " + position + (label 
					!= null ? " : " + label : string.Empty));
			}
			return d;
		}

		public virtual void WriteBigDecimal(System.Decimal d, bool writeInTransaction)
		{
			byte[] bytes = byteArrayConverter.BigDecimalToByteArray(d, true);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog)
			{
				NeoDatis.Tool.DLogger.Debug("writing BigDecimal " + d + " at " + GetPosition());
			}
			if (!writeInTransaction)
			{
				io.WriteBytes(bytes);
			}
			else
			{
				GetSession().GetTransaction().ManageWriteAction(io.GetCurrentPosition(), bytes);
				EnsureSpaceFor(bytes.Length, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.BigDecimal
					);
			}
			bytes = null;
		}

		public virtual byte[] ReadBigDecimalBytes()
		{
			return ReadStringBytes(false);
		}

		// return BigDecimal(io.readBytes(ODBType.BIG_DECIMAL.getSize()));
		public virtual System.Decimal ReadBigDecimal()
		{
			return ReadBigDecimal(null);
		}

		public virtual System.Decimal ReadBigDecimal(string label)
		{
			long position = io.GetCurrentPosition();
			System.Decimal d = byteArrayConverter.ByteArrayToBigDecimal(ReadBigDecimalBytes()
				, false);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog)
			{
				NeoDatis.Tool.DLogger.Debug("Reading bigDecimal '" + d + "' at " + position + (label
					 != null ? " : " + label : string.Empty));
			}
			return d;
		}

		public virtual void WriteBigInteger(System.Decimal d, bool writeInTransaction)
		{
			byte[] bytes = byteArrayConverter.BigIntegerToByteArray(d, true);
			if (!writeInTransaction)
			{
				io.WriteBytes(bytes);
			}
			else
			{
				GetSession().GetTransaction().ManageWriteAction(io.GetCurrentPosition(), bytes);
				EnsureSpaceFor(bytes.Length, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.BigInteger
					);
			}
			bytes = null;
		}

		public virtual byte[] ReadBigIntegerBytes(bool hasSize)
		{
			return ReadStringBytes(hasSize);
		}

		public virtual System.Decimal ReadBigInteger()
		{
			return ReadBigInteger(null);
		}

		public virtual System.Decimal ReadBigInteger(string label)
		{
			long position = io.GetCurrentPosition();
			System.Decimal d = byteArrayConverter.ByteArrayToBigInteger(ReadBigIntegerBytes(true
				), true);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog)
			{
				NeoDatis.Tool.DLogger.Debug("Reading bigInteger '" + d + "' at " + position + (label
					 != null ? " : " + label : string.Empty));
			}
			return d;
		}

		public virtual void WriteDate(System.DateTime d, bool writeInTransaction)
		{
			byte[] bytes = byteArrayConverter.DateToByteArray(d);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog)
			{
				NeoDatis.Tool.DLogger.Debug("writing Date " + NeoDatis.Tool.Wrappers.OdbTime.GetMilliseconds
					(d) + " at " + GetPosition());
			}
			if (!writeInTransaction)
			{
				io.WriteBytes(bytes);
			}
			else
			{
				GetSession().GetTransaction().ManageWriteAction(io.GetCurrentPosition(), bytes);
				EnsureSpaceFor(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Date);
			}
			bytes = null;
		}

		public virtual byte[] ReadDateBytes()
		{
			return io.ReadBytes(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Date.GetSize());
		}

		public virtual System.DateTime ReadDate()
		{
			return ReadDate(null);
		}

		public virtual System.DateTime ReadDate(string label)
		{
			long position = io.GetCurrentPosition();
			System.DateTime date = byteArrayConverter.ByteArrayToDate(ReadDateBytes());
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog)
			{
				NeoDatis.Tool.DLogger.Debug("Reading date '" + date + "' at " + position + (label
					 != null ? " : " + label : string.Empty));
			}
			return date;
		}

		public virtual void WriteString(string s, bool writeInTransaction, bool useEncoding
			)
		{
			WriteString(s, writeInTransaction, useEncoding, -1);
		}

		public virtual void WriteString(string s, bool writeInTransaction, bool useEncoding
			, int totalSpace)
		{
			byte[] bytes = byteArrayConverter.StringToByteArray(s, true, totalSpace, useEncoding
				);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				long position = GetPosition();
				NeoDatis.Tool.DLogger.Debug("Writing string '" + s + "' at " + position + " size="
					 + bytes.Length + " bytes");
			}
			if (!writeInTransaction)
			{
				long startPosition = io.GetCurrentPosition();
				io.WriteBytes(bytes);
				long endPosition = io.GetCurrentPosition();
				if (NeoDatis.Odb.OdbConfiguration.IsEnableAfterWriteChecking())
				{
					// To check the write
					io.SetCurrentWritePosition(startPosition);
					string s2 = ReadString(useEncoding);
					// DLogger.debug("s1 : " + s.length() + " = " + s + "\ts2 : " +
					// s2.length() + " = " + s2);
					// FIXME replace RuntimeException by a ODBRuntimeException with
					// an Error constant
					throw new System.Exception("error while writing string at " + startPosition + " :  "
						 + s + " / check after writing =" + s2);
				}
			}
			else
			{
				GetSession().GetTransaction().ManageWriteAction(io.GetCurrentPosition(), bytes);
				EnsureSpaceFor(bytes.Length, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.String);
			}
			bytes = null;
		}

		public virtual byte[] ReadStringBytes(bool withSize)
		{
			if (withSize)
			{
				byte[] sizeBytes = io.ReadBytes(INT_SIZE_x_2);
				int totalSize = byteArrayConverter.ByteArrayToInt(sizeBytes, 0);
				// Use offset of int size to read real size
				int stringSize = byteArrayConverter.ByteArrayToInt(sizeBytes, IntSize);
				byte[] bytes = ReadBytes(stringSize);
				nbCall2++;
				// Reads extra bytes
				byte[] extraBytes = ReadBytes(totalSize - stringSize);
				byte[] bytes2 = new byte[stringSize + INT_SIZE_x_2];
				for (int i = 0; i < INT_SIZE_x_2; i++)
				{
					bytes2[i] = sizeBytes[i];
				}
				for (int i = 0; i < bytes.Length; i++)
				{
					bytes2[i + 8] = bytes[i];
				}
				extraBytes = null;
				sizeBytes = null;
				return bytes2;
			}
			byte[] sizeBytesNoSize = io.ReadBytes(INT_SIZE_x_2);
			int stringSizeNoSize = byteArrayConverter.ByteArrayToInt(sizeBytesNoSize, IntSize
				);
			byte[] bytesNoSize = ReadBytes(stringSizeNoSize);
			nbCall1++;
			sizeBytesNoSize = null;
			return bytesNoSize;
		}

		public virtual string ReadString(bool useEncoding)
		{
			return ReadString(useEncoding, NeoDatis.Odb.OdbConfiguration.GetDatabaseCharacterEncoding
				());
		}

		public virtual string ReadString(bool useEncoding, string label)
		{
			string s = byteArrayConverter.ByteArrayToString(ReadStringBytes(true), true, useEncoding
				);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog)
			{
				long startPosition = io.GetCurrentPosition();
				NeoDatis.Tool.DLogger.Debug("Reading string '" + s + "' at " + startPosition + (label
					 != null ? " : " + label : string.Empty));
			}
			return s;
		}

		public virtual void WriteBoolean(bool b, bool writeInTransaction)
		{
			WriteBoolean(b, writeInTransaction, null);
		}

		public virtual void WriteBoolean(bool b, bool writeInTransaction, string label)
		{
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog && label != null)
			{
				NeoDatis.Tool.DLogger.Debug("writing boolean " + b + " at " + GetPosition() + " : "
					 + label);
			}
			byte[] bytes = byteArrayConverter.BooleanToByteArray(b);
			if (!writeInTransaction)
			{
				io.WriteBytes(bytes);
			}
			else
			{
				GetSession().GetTransaction().ManageWriteAction(io.GetCurrentPosition(), bytes);
				EnsureSpaceFor(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeBoolean);
			}
			bytes = null;
		}

		public virtual byte[] ReadBooleanBytes()
		{
			return io.ReadBytes(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Boolean.GetSize(
				));
		}

		public virtual bool ReadBoolean()
		{
			return ReadBoolean(null);
		}

		public virtual bool ReadBoolean(string label)
		{
			long position = io.GetCurrentPosition();
			bool b = byteArrayConverter.ByteArrayToBoolean(ReadBooleanBytes(), 0);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId) && canLog && label != null)
			{
				NeoDatis.Tool.DLogger.Debug("reading boolean " + b + " at " + position + " : " + 
					label);
			}
			return b;
		}

		public virtual byte[] ReadNativeAttributeBytes(int attributeType)
		{
			switch (attributeType)
			{
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeByteId:
				{
					byte[] bytes = new byte[1];
					bytes[0] = ReadByte();
					return bytes;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeBooleanId:
				{
					return ReadBooleanBytes();
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeCharId:
				{
					return ReadCharBytes();
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeFloatId:
				{
					return ReadFloatBytes();
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeDoubleId:
				{
					return ReadDoubleBytes();
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeIntId:
				{
					return ReadIntBytes();
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeLongId:
				{
					return ReadLongBytes();
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeShortId:
				{
					return ReadShortBytes();
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.BigDecimalId:
				{
					return ReadBigDecimalBytes();
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.BigIntegerId:
				{
					return ReadBigIntegerBytes(true);
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.BooleanId:
				{
					return ReadBooleanBytes();
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.CharacterId:
				{
					return ReadCharBytes();
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DateId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DateSqlId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DateTimestampId:
				{
					return ReadDateBytes();
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.FloatId:
				{
					return ReadFloatBytes();
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DoubleId:
				{
					return ReadDoubleBytes();
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.IntegerId:
				{
					return ReadIntBytes();
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.StringId:
				{
					return ReadStringBytes(true);
				}

				default:
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NativeTypeNotSupported
						.AddParameter(attributeType).AddParameter(string.Empty));
					break;
				}
			}
		}

		public virtual void Close()
		{
			Clear();
			io.Close();
			io = null;
		}

		public virtual void Clear()
		{
		}

		// Nothing to do
		public virtual NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification GetParameters(
			)
		{
			return parameters;
		}

		public virtual bool Delete()
		{
			return io.Delete();
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.IBufferedIO GetIo()
		{
			return io;
		}
	}
}
