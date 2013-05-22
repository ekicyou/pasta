namespace NeoDatis.Odb.Test.Update.Nullobject
{
	/// <summary>SensorAT</summary>
	public class SensorAT
	{
		private string name;

		private int state;

		private float km;

		private int lane;

		private int way;

		private NeoDatis.Odb.Test.Update.Nullobject.AT at;

		private bool deleted;

		private bool status;

		private System.DateTime creationDate;

		private System.DateTime updateDate;

		private NeoDatis.Odb.Test.Update.Nullobject.User user;

		// S ou N
		// Sim ou Nao
		public override string ToString()
		{
			return "[" + at + "][" + name + "][" + state + "][" + lane + "][" + way + "][" + 
				km + "]";
		}

		public virtual float GetKm()
		{
			return km;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual int GetState()
		{
			return state;
		}

		public virtual int GetWay()
		{
			return way;
		}

		public virtual int GetLane()
		{
			return lane;
		}

		public virtual NeoDatis.Odb.Test.Update.Nullobject.AT GetAt()
		{
			return at;
		}

		public virtual System.DateTime GetCreationDate()
		{
			return creationDate;
		}

		public virtual System.DateTime GetUpdateDate()
		{
			return updateDate;
		}

		public virtual NeoDatis.Odb.Test.Update.Nullobject.User GetUser()
		{
			return user;
		}

		public virtual bool GetDeleted()
		{
			return deleted;
		}

		public virtual bool GetStatus()
		{
			return status;
		}

		public virtual void SetDeleted(bool deleted)
		{
			this.deleted = deleted;
		}

		public virtual void SetStatus(bool status)
		{
			this.status = status;
		}

		public virtual void SetAt(NeoDatis.Odb.Test.Update.Nullobject.AT at)
		{
			this.at = at;
		}

		public virtual void SetCreationDate(System.DateTime creationDate)
		{
			this.creationDate = creationDate;
		}

		public virtual void SetUpdateDate(System.DateTime updateDate)
		{
			this.updateDate = updateDate;
		}

		public virtual void SetUser(NeoDatis.Odb.Test.Update.Nullobject.User user)
		{
			this.user = user;
		}

		public virtual void SetWay(int way)
		{
			this.way = way;
		}

		public virtual void SetLane(int lane)
		{
			this.lane = lane;
		}

		public virtual void SetKm(float km)
		{
			this.km = km;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual void SetState(int state)
		{
			this.state = state;
		}
	}
}
