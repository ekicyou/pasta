namespace NeoDatis.Odb.Core.Layers.Layer3
{
	/// <summary>The interface for buffered IO</summary>
	/// <author>osmadja</author>
	public interface IBufferedIO
	{
		void GoToPosition(long position);

		long GetLength();

		/// <summary>
		/// Checks if the new position is in the buffer, if not, flushes the buffer
		/// and rebuilds it to the correct position
		/// </summary>
		/// <param name="newPosition"></param>
		/// <param name="readOrWrite"></param>
		/// <param name="size">Size if the data that must be stored</param>
		/// <returns>The index of the buffer where that contains the position</returns>
		int ManageBufferForNewPosition(long newPosition, int readOrWrite, int size);

		bool IsUsingbuffer();

		void SetUseBuffer(bool useBuffer);

		long GetCurrentPosition();

		void SetCurrentWritePosition(long currentPosition);

		void SetCurrentReadPosition(long currentPosition);

		void WriteByte(byte b);

		byte[] ReadBytesOld(int size);

		byte[] ReadBytes(int size);

		byte ReadByte();

		void WriteBytes(byte[] bytes);

		void Flush(int bufferIndex);

		void FlushAll();

		long GetIoDeviceLength();

		void SetIoDeviceLength(long ioDeviceLength);

		void Close();

		void Clear();

		bool Delete();

		bool IsForTransaction();

		void EnableAutomaticDelete(bool yesOrNo);

		bool AutomaticDeleteIsEnabled();
	}
}
