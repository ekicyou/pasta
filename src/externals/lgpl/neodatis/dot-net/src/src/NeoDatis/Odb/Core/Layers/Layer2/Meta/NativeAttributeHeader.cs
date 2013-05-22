namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>A class that contain basic information about a native object</summary>
	/// <author>osmadja</author>
	public class NativeAttributeHeader
	{
		private int blockSize;

		private byte blockType;

		private bool isNative;

		private int odbTypeId;

		private bool isNull;

		public NativeAttributeHeader() : base()
		{
		}

		public NativeAttributeHeader(int blockSize, byte blockType, bool isNative, int odbTypeId
			, bool isNull) : base()
		{
			this.blockSize = blockSize;
			this.blockType = blockType;
			this.isNative = isNative;
			this.odbTypeId = odbTypeId;
			this.isNull = isNull;
		}

		public virtual int GetBlockSize()
		{
			return blockSize;
		}

		public virtual void SetBlockSize(int blockSize)
		{
			this.blockSize = blockSize;
		}

		public virtual byte GetBlockType()
		{
			return blockType;
		}

		public virtual void SetBlockType(byte blockType)
		{
			this.blockType = blockType;
		}

		public virtual bool IsNative()
		{
			return isNative;
		}

		public virtual void SetNative(bool isNative)
		{
			this.isNative = isNative;
		}

		public virtual bool IsNull()
		{
			return isNull;
		}

		public virtual void SetNull(bool isNull)
		{
			this.isNull = isNull;
		}

		public virtual int GetOdbTypeId()
		{
			return odbTypeId;
		}

		public virtual void SetOdbTypeId(int odbTypeId)
		{
			this.odbTypeId = odbTypeId;
		}
	}
}
