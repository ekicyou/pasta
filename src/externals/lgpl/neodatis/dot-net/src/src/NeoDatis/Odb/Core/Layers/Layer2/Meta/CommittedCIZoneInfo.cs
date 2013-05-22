namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>Used for committed zone info.</summary>
	/// <remarks>
	/// Used for committed zone info. It has one more attribute than the super class. It is used
	/// to keep track of committed deleted objects
	/// </remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class CommittedCIZoneInfo : NeoDatis.Odb.Core.Layers.Layer2.Meta.CIZoneInfo
	{
		public long nbDeletedObjects;

		public CommittedCIZoneInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci, NeoDatis.Odb.OID
			 first, NeoDatis.Odb.OID last, long nbObjects) : base(ci, first, last, nbObjects
			)
		{
			nbDeletedObjects = 0;
		}

		public override void DecreaseNbObjects()
		{
			nbDeletedObjects++;
		}

		public virtual long GetNbDeletedObjects()
		{
			return nbDeletedObjects;
		}

		public virtual void SetNbDeletedObjects(long nbDeletedObjects)
		{
			this.nbDeletedObjects = nbDeletedObjects;
		}

		public override long GetNbObjects()
		{
			return nbObjects - nbDeletedObjects;
		}

		public override void SetNbObjects(long nb)
		{
			this.nbObjects = nb;
			this.nbDeletedObjects = 0;
		}

		public virtual void SetNbObjects(NeoDatis.Odb.Core.Layers.Layer2.Meta.CommittedCIZoneInfo
			 cizi)
		{
			this.nbObjects = cizi.nbObjects;
			this.nbDeletedObjects = cizi.nbDeletedObjects;
		}

		public override string ToString()
		{
			return "(first=" + first + ",last=" + last + ",nb=" + nbObjects + "-" + nbDeletedObjects
				 + ")";
		}
	}
}
