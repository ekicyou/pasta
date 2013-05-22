using System;
using System.IO;
using NeoDatis.Odb;
using NeoDatis.Odb.Core;

namespace NeoDatis.Tool.Wrappers.IO{

	//@TODO
	public class OdbFileIO : NeoDatis.Odb.Core.Layers.Layer3.IO{
		
		private FileStream fileAccess;
	
		private string fileName;
	
	
	public OdbFileIO(string wholeFileName, bool canWrite, string password) {
		Init(wholeFileName,canWrite,password);
	}
	public long Length() {
		return fileAccess.Length;
	}
	public void Seek(long position)  {
		try {
			if(position<0){
				throw new ODBRuntimeException(NeoDatisError.NegativePosition.AddParameter(position));
			}
			fileAccess.Seek(position, System.IO.SeekOrigin.Begin);
		} catch (IOException e) {
			throw new ODBRuntimeException(NeoDatisError.GoToPosition.AddParameter(position).AddParameter(fileAccess.Length), e);
		}
	}
	public void Write(byte b) {
		fileAccess.WriteByte(b);
	}
	public void Write(byte[] bs, int offset, int size) {
		fileAccess.Write(bs,offset,size);
		
	}
	public int Read() {
		return fileAccess.ReadByte();
	}
	public long Read(byte[] array, int offset, int size) {
		return fileAccess.Read(array, offset, size);
	}
	public void Close() {
		fileAccess.Close();		
	}

	public bool LockFile() {
		fileAccess.Lock(0,1);
		return true;
	}
	public bool UnlockFile() {
		fileAccess.Unlock(0,1);
		return true;
	}
	public bool IsLocked(){
		return fileAccess!=null;
	}
	public void Init(string fileName, bool canWrite, string password){
		this.fileName = fileName;
		try{
			this.fileAccess = new FileStream(fileName, FileMode.OpenOrCreate, canWrite? FileAccess.ReadWrite:FileAccess.Read);
		}catch (IOException e) {
			throw new ODBRuntimeException(NeoDatisError.FileNotFound.AddParameter(fileName));
		}

	}

	}
}