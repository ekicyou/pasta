namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>
	/// Class keep track of object pointers and number of objects of a class info for
	/// a specific zone
	/// <pre>
	/// For example, to keep track of first committed and last committed object position.
	/// </summary>
	/// <remarks>
	/// Class keep track of object pointers and number of objects of a class info for
	/// a specific zone
	/// <pre>
	/// For example, to keep track of first committed and last committed object position.
	/// </pre>
	/// </remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class CIZoneInfo
	{
		public NeoDatis.Odb.OID first;

		public NeoDatis.Odb.OID last;

		protected long nbObjects;

		public NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci;

		public CIZoneInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci, NeoDatis.Odb.OID
			 first, NeoDatis.Odb.OID last, long nbObjects) : base()
		{
			this.first = first;
			this.last = last;
			this.nbObjects = nbObjects;
			this.ci = ci;
		}

		public override string ToString()
		{
			return "(first=" + first + ",last=" + last + ",nb=" + nbObjects + ")";
		}

		public virtual void Reset()
		{
			first = null;
			last = null;
			nbObjects = 0;
		}

		public virtual void Set(NeoDatis.Odb.Core.Layers.Layer2.Meta.CIZoneInfo zoneInfo)
		{
			this.nbObjects = zoneInfo.nbObjects;
			this.first = zoneInfo.first;
			this.last = zoneInfo.last;
		}

		public virtual void DecreaseNbObjects()
		{
			nbObjects--;
			if (nbObjects < 0)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
					.AddParameter("nb objects is negative! in " + ci.GetFullClassName()));
			}
		}

		public virtual void IncreaseNbObjects()
		{
			nbObjects++;
		}

		public virtual long GetNbObjects()
		{
			return nbObjects;
		}

		public virtual bool HasObjects()
		{
			return nbObjects != 0;
		}

		public virtual void SetNbObjects(long nb)
		{
			this.nbObjects = nb;
		}
	}
}
