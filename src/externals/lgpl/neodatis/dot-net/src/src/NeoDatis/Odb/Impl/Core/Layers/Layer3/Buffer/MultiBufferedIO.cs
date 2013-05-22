namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Buffer
{
	/// <summary>
	/// Abstract class allowing buffering for IO
	/// This class is used to give a transparent access to buffered io : File, socket
	/// The DefaultFileIO and DefaultSocketIO inherits from AbstractIO
	/// </summary>
	/// <author>olivier s</author>
	public abstract class MultiBufferedIO : NeoDatis.Odb.Core.Layers.Layer3.IBufferedIO
	{
		public static long nbWrites;

		public static long totalWriteSize;

		/// <summary>Internal counter of flush</summary>
		public static long numberOfFlush = 0;

		public static long totalFlushSize = 0;

		public static int nbFlushForOverlap = 0;

		public static int nbBufferOk;

		public static int nbBufferNotOk;

		public static int nbSamePositionForWrite;

		public static int nbSamePositionForRead;

		public static readonly string LogId = "MultiBufferedIO";

		private const int Read = 1;

		private const int Write = 2;

		private string name;

		/// <summary>The length of the io device</summary>
		private long ioDeviceLength;

		private NeoDatis.Tool.Wrappers.IO.MultiBufferVO multiBuffer;

		private int nbBuffers;

		private int[] overlappingBuffers;

		private int currentBufferIndex;

		/// <summary>The size of the buffer</summary>
		private int bufferSize;

		/// <summary>A boolean value to check if read write are using buffer</summary>
		private bool isUsingBuffer;

		protected long currentPositionWhenUsingBuffer;

		private long currentPositionForDirectWrite;

		private bool enableAutomaticDelete;

		private int nextBufferIndex;

		public MultiBufferedIO(int nbBuffers, string name, int bufferSize, bool canWrite)
		{
			this.nbBuffers = nbBuffers;
			multiBuffer = new NeoDatis.Tool.Wrappers.IO.MultiBufferVO(nbBuffers, bufferSize);
			this.bufferSize = bufferSize;
			currentPositionWhenUsingBuffer = -1;
			currentPositionForDirectWrite = -1;
			overlappingBuffers = new int[nbBuffers];
			numberOfFlush = 0;
			isUsingBuffer = true;
			this.name = name;
			enableAutomaticDelete = true;
			nextBufferIndex = 0;
		}

		public abstract void GoToPosition(long position);

		public abstract long GetLength();

		public abstract void InternalWrite(byte b);

		public abstract void InternalWrite(byte[] bs, int size);

		public abstract byte InternalRead();

		public abstract long InternalRead(byte[] array, int size);

		public abstract void CloseIO();

		public virtual int ManageBufferForNewPosition(long newPosition, int readOrWrite, 
			int size)
		{
			int bufferIndex = multiBuffer.GetBufferIndexForPosition(newPosition, size);
			if (bufferIndex != -1)
			{
				nbBufferOk++;
				return bufferIndex;
			}
			nbBufferNotOk++;
			// checks if there is any overlapping buffer
			overlappingBuffers = GetOverlappingBuffers(newPosition, bufferSize);
			// Choose the first overlaping buffer
			bufferIndex = overlappingBuffers[0];
			if (nbBuffers > 1 && overlappingBuffers[1] != -1 && bufferIndex == currentBufferIndex)
			{
				bufferIndex = overlappingBuffers[1];
			}
			if (bufferIndex == -1)
			{
				bufferIndex = nextBufferIndex;
				nextBufferIndex = (nextBufferIndex + 1) % nbBuffers;
				if (bufferIndex == currentBufferIndex)
				{
					bufferIndex = nextBufferIndex;
					nextBufferIndex = (nextBufferIndex + 1) % nbBuffers;
				}
				Flush(bufferIndex);
			}
			currentBufferIndex = bufferIndex;
			// TODO check length and getLength
			long length = GetLength();
			if (readOrWrite == Read && newPosition >= length)
			{
				string message = "End Of File reached - position = " + newPosition + " : Length = "
					 + length;
				NeoDatis.Tool.DLogger.Error(message);
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.EndOfFileReached
					.AddParameter(newPosition).AddParameter(length));
			}
			// The buffer must be initialized with real data, so the first thing we
			// must do is read data from file and puts it in the array
			long nread = bufferSize;
			// if new position is in the file
			if (newPosition < length)
			{
				// We are in the file, we are updating content. to create the
				// buffer, we first read the content of the file
				GoToPosition(newPosition);
				// Actually loads data from the file to the buffer
				nread = InternalRead(multiBuffer.buffers[bufferIndex], bufferSize);
				multiBuffer.SetCreationDate(bufferIndex, NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs
					());
			}
			else
			{
				GoToPosition(newPosition);
			}
			long endPosition = -1;
			// If we are in READ, sets the size equal to what has been read
			if (readOrWrite == Read)
			{
				endPosition = newPosition + nread;
			}
			else
			{
				endPosition = newPosition + bufferSize;
			}
			multiBuffer.SetPositions(bufferIndex, newPosition, endPosition, 0);
			currentPositionWhenUsingBuffer = newPosition;
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("Creating buffer " + name + "-" + bufferIndex + " : "
					 + "[" + multiBuffer.bufferStartPosition[bufferIndex] + "," + multiBuffer.bufferEndPosition
					[bufferIndex] + "]");
			}
			return bufferIndex;
		}

		/// <summary>
		/// Check if a new buffer starting at position with a size ='size' would
		/// overlap with an existing buffer
		/// </summary>
		/// <param name="position"></param>
		/// <param name="size"></param>
		/// <returns>@</returns>
		private int[] GetOverlappingBuffers(long position, int size)
		{
			long start1 = position;
			long end1 = position + size;
			long start2 = 0;
			long end2 = 0;
			int[] indexes = new int[nbBuffers];
			int index = 0;
			for (int i = 0; i < nbBuffers; i++)
			{
				start2 = multiBuffer.bufferStartPosition[i];
				end2 = multiBuffer.bufferEndPosition[i];
				if ((start1 >= start2 && start1 < end2) || (start2 >= start1 && start2 < end1) ||
					 start2 <= start1 && end2 >= end1)
				{
					// This buffer is overlapping the buffer
					indexes[index++] = i;
					// Flushes the buffer
					Flush(i);
					nbFlushForOverlap++;
				}
			}
			for (int i = index; i < nbBuffers; i++)
			{
				indexes[i] = -1;
			}
			return indexes;
		}

		public virtual bool IsUsingbuffer()
		{
			return isUsingBuffer;
		}

		public virtual void SetUseBuffer(bool useBuffer)
		{
			// If we are using buffer, and the new useBuffer indicator if false
			// Then we need to flush all buffers
			if (isUsingBuffer && !useBuffer)
			{
				FlushAll();
			}
			this.isUsingBuffer = useBuffer;
		}

		public virtual long GetCurrentPosition()
		{
			if (!isUsingBuffer)
			{
				return currentPositionForDirectWrite;
			}
			return currentPositionWhenUsingBuffer;
		}

		public virtual void SetCurrentWritePosition(long currentPosition)
		{
			if (isUsingBuffer)
			{
				if (this.currentPositionWhenUsingBuffer == currentPosition)
				{
					nbSamePositionForWrite++;
					return;
				}
				this.currentPositionWhenUsingBuffer = currentPosition;
			}
			else
			{
				//manageBufferForNewPosition(currentPosition, WRITE, 1);
				this.currentPositionForDirectWrite = currentPosition;
				GoToPosition(currentPosition);
			}
		}

		public virtual void SetCurrentReadPosition(long currentPosition)
		{
			if (isUsingBuffer)
			{
				if (this.currentPositionWhenUsingBuffer == currentPosition)
				{
					nbSamePositionForRead++;
					return;
				}
				this.currentPositionWhenUsingBuffer = currentPosition;
				ManageBufferForNewPosition(currentPosition, Read, 1);
			}
			else
			{
				this.currentPositionForDirectWrite = currentPosition;
				GoToPosition(currentPosition);
			}
		}

		public virtual void WriteByte(byte b)
		{
			if (!isUsingBuffer)
			{
				GoToPosition(currentPositionForDirectWrite);
				InternalWrite(b);
				currentPositionForDirectWrite++;
				return;
			}
			int bufferIndex = multiBuffer.GetBufferIndexForPosition(currentPositionWhenUsingBuffer
				, 1);
			if (bufferIndex == -1)
			{
				bufferIndex = ManageBufferForNewPosition(currentPositionWhenUsingBuffer, Write, 1
					);
			}
			int positionInBuffer = (int)(currentPositionWhenUsingBuffer - multiBuffer.bufferStartPosition
				[bufferIndex]);
			multiBuffer.SetByte(bufferIndex, positionInBuffer, b);
			currentPositionWhenUsingBuffer++;
			if (currentPositionWhenUsingBuffer > ioDeviceLength)
			{
				ioDeviceLength = currentPositionWhenUsingBuffer;
			}
		}

		public virtual byte[] ReadBytesOld(int size)
		{
			byte[] bytes = new byte[size];
			for (int i = 0; i < size; i++)
			{
				bytes[i] = ReadByte();
			}
			return bytes;
		}

		public virtual byte[] ReadBytes(int size)
		{
			byte[] bytes = new byte[size];
			if (!isUsingBuffer)
			{
				// If there is no buffer, simply read data
				GoToPosition(currentPositionForDirectWrite);
				long realSize = InternalRead(bytes, size);
				currentPositionForDirectWrite += realSize;
				return bytes;
			}
			// If the size to read in smaller than the buffer size
			if (size <= bufferSize)
			{
				return ReadBytes(bytes, 0, size);
			}
			// else the read have to use various buffers
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("Data is larger than buffer size " + bytes.Length + " > "
					 + bufferSize + " : cutting the data");
			}
			int nbBuffersNeeded = bytes.Length / bufferSize + 1;
			int currentStart = 0;
			int currentEnd = bufferSize;
			for (int i = 0; i < nbBuffersNeeded; i++)
			{
				ReadBytes(bytes, currentStart, currentEnd);
				currentStart += bufferSize;
				if (currentEnd + bufferSize < bytes.Length)
				{
					currentEnd += bufferSize;
				}
				else
				{
					currentEnd = bytes.Length;
				}
			}
			return bytes;
		}

		public virtual byte[] ReadBytes(byte[] bytes, int startIndex, int endIndex)
		{
			int size = endIndex - startIndex;
			int bufferIndex = ManageBufferForNewPosition(currentPositionWhenUsingBuffer, Read
				, size);
			int start = (int)(currentPositionWhenUsingBuffer - multiBuffer.bufferStartPosition
				[bufferIndex]);
			byte[] buffer = multiBuffer.buffers[bufferIndex];
			System.Array.Copy(buffer, start, bytes, startIndex, size);
			currentPositionWhenUsingBuffer += size;
			return bytes;
		}

		public virtual byte ReadByte()
		{
			if (!isUsingBuffer)
			{
				GoToPosition(currentPositionForDirectWrite);
				byte b = InternalRead();
				currentPositionForDirectWrite++;
				return b;
			}
			int bufferIndex = ManageBufferForNewPosition(currentPositionWhenUsingBuffer, Read
				, 1);
			byte byt = multiBuffer.GetByte(bufferIndex, (int)(currentPositionWhenUsingBuffer 
				- multiBuffer.bufferStartPosition[bufferIndex]));
			currentPositionWhenUsingBuffer++;
			return byt;
		}

		public virtual void WriteBytes(byte[] bytes)
		{
			if (bytes.Length > bufferSize)
			{
				// throw new ODBRuntimeException(session,"The buffer has a size of "
				// + bufferSize + " but there exist data with " + bytes.length +
				// " size! - please set manually the odb data buffer to a greater value than "
				// + bytes.length +
				// " using Configuration.setDefaultBufferSizeForData(int)");
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug("Data is larger than buffer size " + bytes.Length + " > "
						 + bufferSize + " : cutting the data");
				}
				int nbBuffersNeeded = bytes.Length / bufferSize + 1;
				int currentStart = 0;
				int currentEnd = bufferSize;
				for (int i = 0; i < nbBuffersNeeded; i++)
				{
					WriteBytes(bytes, currentStart, currentEnd);
					currentStart += bufferSize;
					if (currentEnd + bufferSize < bytes.Length)
					{
						currentEnd += bufferSize;
					}
					else
					{
						currentEnd = bytes.Length;
					}
				}
			}
			else
			{
				WriteBytes(bytes, 0, bytes.Length);
			}
		}

		public virtual void WriteBytes(byte[] bytes, int startIndex, int endIndex)
		{
			if (!isUsingBuffer)
			{
				GoToPosition(currentPositionForDirectWrite);
				InternalWrite(bytes, bytes.Length);
				currentPositionForDirectWrite += bytes.Length;
				return;
			}
			int lengthToCopy = endIndex - startIndex;
			nbWrites++;
			totalWriteSize += lengthToCopy;
			int bufferIndex = ManageBufferForNewPosition(currentPositionWhenUsingBuffer, Write
				, lengthToCopy);
			int positionInBuffer = (int)(currentPositionWhenUsingBuffer - multiBuffer.bufferStartPosition
				[bufferIndex]);
			// Here, the bytes.length seems to have an average value lesser that 70,
			// and in this
			// It is faster to copy using System.arraycopy
			// see org.neodatis.odb.test.performance.TestArrayCopy	
			multiBuffer.WriteBytes(bufferIndex, bytes, startIndex, positionInBuffer, lengthToCopy
				);
			positionInBuffer = positionInBuffer + lengthToCopy - 1;
			currentPositionWhenUsingBuffer += lengthToCopy;
			if (currentPositionWhenUsingBuffer > ioDeviceLength)
			{
				ioDeviceLength = currentPositionWhenUsingBuffer;
			}
		}

		public virtual void FlushAll()
		{
			for (int i = 0; i < nbBuffers; i++)
			{
				Flush(i);
			}
		}

		public virtual void Flush(int bufferIndex)
		{
			byte[] buffer = multiBuffer.buffers[bufferIndex];
			if (buffer != null && multiBuffer.HasBeenUsedForWrite(bufferIndex))
			{
				GoToPosition(multiBuffer.bufferStartPosition[bufferIndex]);
				// the +1 is because the maxPositionInBuffer is a position and the
				// parameter is a length
				int bufferSizeToFlush = multiBuffer.maxPositionInBuffer[bufferIndex] + 1;
				InternalWrite(buffer, bufferSizeToFlush);
				numberOfFlush++;
				totalFlushSize += bufferSizeToFlush;
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug("Flushing buffer " + name + "-" + bufferIndex + " : ["
						 + multiBuffer.bufferStartPosition[bufferIndex] + ":" + multiBuffer.bufferEndPosition
						[bufferIndex] + "] - flush size=" + bufferSizeToFlush + "  flush number = " + numberOfFlush
						);
				}
				multiBuffer.ClearBuffer(bufferIndex);
			}
			else
			{
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug("Flushing buffer " + name + "-" + bufferIndex + " : ["
						 + multiBuffer.bufferStartPosition[bufferIndex] + ":" + multiBuffer.bufferEndPosition
						[bufferIndex] + "] - Nothing to flush!");
				}
				multiBuffer.ClearBuffer(bufferIndex);
			}
		}

		/// <returns>Returns the numberOfFlush.</returns>
		public virtual long GetNumberOfFlush()
		{
			return numberOfFlush;
		}

		public virtual long GetIoDeviceLength()
		{
			return ioDeviceLength;
		}

		public virtual void SetIoDeviceLength(long ioDeviceLength)
		{
			this.ioDeviceLength = ioDeviceLength;
		}

		public virtual void Close()
		{
			Clear();
			CloseIO();
		}

		public virtual void Clear()
		{
			FlushAll();
			multiBuffer.Clear();
			multiBuffer = null;
			overlappingBuffers = null;
		}

		public virtual bool IsForTransaction()
		{
			return name != null && name.Equals("transaction");
		}

		public virtual void EnableAutomaticDelete(bool yesOrNo)
		{
			this.enableAutomaticDelete = yesOrNo;
		}

		public virtual bool AutomaticDeleteIsEnabled()
		{
			return enableAutomaticDelete;
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append("Buffers=").Append("currbuffer=").Append(currentBufferIndex).Append
				(" : \n");
			for (int i = 0; i < nbBuffers; i++)
			{
				buffer.Append(i).Append(":[").Append(multiBuffer.bufferStartPosition[i]).Append(","
					).Append(multiBuffer.bufferEndPosition[i]).Append("] : write=").Append(multiBuffer
					.HasBeenUsedForWrite(i)).Append(" - when=").Append(multiBuffer.GetCreationDate(i
					));
				if (i + 1 < nbBuffers)
				{
					buffer.Append("\n");
				}
			}
			return buffer.ToString();
		}

		public abstract bool Delete();
	}
}
