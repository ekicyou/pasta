namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid
{
	/// <summary>Used to obtain internal infos about all database ids</summary>
	/// <author>osmadja</author>
	public class FullIDInfo
	{
		private long id;

		private long position;

		private long blockId;

		private byte idStatus;

		private string objectClassName;

		private string objectToString;

		private NeoDatis.Odb.OID prevOID;

		private NeoDatis.Odb.OID nextOID;

		public FullIDInfo(long id, long position, byte idStatus, long blockId, string objectClassName
			, string objectToString, NeoDatis.Odb.OID prevOID, NeoDatis.Odb.OID nextOID)
		{
			this.id = id;
			this.position = position;
			this.blockId = blockId;
			this.objectClassName = objectClassName;
			this.objectToString = objectToString;
			this.idStatus = idStatus;
			this.prevOID = prevOID;
			this.nextOID = nextOID;
		}

		public virtual long GetBlockId()
		{
			return blockId;
		}

		public virtual void SetBlockId(long blockId)
		{
			this.blockId = blockId;
		}

		public virtual long GetId()
		{
			return id;
		}

		public virtual void SetId(long id)
		{
			this.id = id;
		}

		public virtual string GetObjectClassName()
		{
			return objectClassName;
		}

		public virtual void SetObjectClassName(string objectClassName)
		{
			this.objectClassName = objectClassName;
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append("Id=").Append(id).Append(" - Posi=").Append(position).Append(" - Status="
				).Append(idStatus).Append(" - Block Id=").Append(blockId);
			buffer.Append(" - Type=").Append(objectClassName);
			buffer.Append(" - prev inst. pos=").Append(prevOID);
			buffer.Append(" - next inst. pos=").Append(nextOID);
			buffer.Append(" - Object=").Append(objectToString);
			return buffer.ToString();
		}

		public static void Main2(string[] args)
		{
			NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.FullIDInfo ii = new NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.FullIDInfo
				(1, 1, (byte)1, 1, string.Empty, string.Empty, null, null);
			ii.SetObjectClassName("ola");
			System.Console.Out.WriteLine("ll=" + ii.GetObjectClassName());
		}
	}
}
