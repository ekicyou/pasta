namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Crypto
{
	/// <summary>A simple cypher based on AES/MD5.</summary>
	/// <remarks>A simple cypher based on AES/MD5. Code from Grant Slender</remarks>
	/// <author>osmadja</author>
	public class AesMd5Cypher : NeoDatis.Odb.Core.Layers.Layer3.IO
	{

		public AesMd5Cypher()
		{
		}

		// AES = 16 bytes or 128 bits.
		// ensure total block size is much
		// larger.
		/// <exception cref="System.Exception"></exception>
		public virtual void Init(string fileName, bool canWrite, string password)
		{
		}

		/// <exception cref="System.IO.IOException"></exception>
		public virtual void Close()
		{
		}

		// The file lock is automatically released closing the raf object
		/// <exception cref="System.IO.IOException"></exception>
		public virtual int Read()
		{
			byte[] bytes = new byte[1];
			long l = Read(bytes,0, 1);
			return bytes[0];
		}

		/// <exception cref="System.IO.IOException"></exception>
		public virtual long Read(byte[] bytes, int offset, int size)
		{
			return 1;
		}

		/// <exception cref="System.IO.IOException"></exception>
		public virtual void Seek(long position)
		{
			try
			{
				if (position < 0)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NegativePosition
						.AddParameter(position));
				}
			}
			catch (System.IO.IOException e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.GoToPosition.AddParameter
					(position).AddParameter(0), e);
			}
		}

		/// <exception cref="System.IO.IOException"></exception>
		public virtual void Write(byte b)
		{
		}

		/// <exception cref="System.IO.IOException"></exception>
		public virtual void Write(byte[] bytes, int offset, int size)
		{
		}

		/// <exception cref="System.IO.IOException"></exception>
		public virtual long Length()
		{
			return 0;
		}

		/// <exception cref="System.IO.IOException"></exception>
		public virtual bool LockFile()
		{
			return false;
		}
		public virtual bool UnlockFile()
		{
			return false;
		}
		public virtual bool IsLocked()
		{
			return false;
		}
	}
}
