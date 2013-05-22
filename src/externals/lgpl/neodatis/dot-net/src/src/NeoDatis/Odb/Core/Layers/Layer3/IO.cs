namespace NeoDatis.Odb.Core.Layers.Layer3
{
	/// <summary>The basic IO interface for basic IO operation like reading and writing bytes
	/// 	</summary>
	/// <author>olivier</author>
	public interface IO
	{
		/// <exception cref="Java.IO.FileNotFoundException"></exception>
		/// <exception cref="System.Exception"></exception>
		void Init(string fileName, bool canWrite, string password);

		/// <exception cref="System.IO.IOException"></exception>
		void Seek(long pos);

		/// <exception cref="System.IO.IOException"></exception>
		void Close();

		/// <exception cref="System.IO.IOException"></exception>
		void Write(byte b);

		/// <exception cref="System.IO.IOException"></exception>
		void Write(byte[] bytes, int offset, int size);

		/// <exception cref="System.IO.IOException"></exception>
		long Read(byte[] bytes, int offset, int size);

		/// <exception cref="System.IO.IOException"></exception>
		int Read();

		/// <exception cref="System.IO.IOException"></exception>
		long Length();

		/// <exception cref="System.IO.IOException"></exception>
		bool LockFile();

		/// <exception cref="System.IO.IOException"></exception>
		bool UnlockFile();

		/// <exception cref="System.IO.IOException"></exception>
		bool IsLocked();
	}
}
