namespace NeoDatis.Odb.Test.Update.Nullobject
{
	/// <summary>SensorMeteo</summary>
	public class SensorMeteo
	{
		private string name;

		private int state;

		private int way;

		private float km;

		private NeoDatis.Odb.Test.Update.Nullobject.Meteo meteo;

		private bool deleted;

		private bool status;

		private System.DateTime creationDate;

		private System.DateTime updateDate;

		private NeoDatis.Odb.Test.Update.Nullobject.User user;

		// S ou N
		// Sim ou Nao
		public override string ToString()
		{
			return "[" + name + "][" + state + "][" + status + "][" + way + "][" + km + "]";
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

		public virtual System.DateTime GetCreationDate()
		{
			return creationDate;
		}

		public virtual bool GetDeleted()
		{
			return deleted;
		}

		public virtual NeoDatis.Odb.Test.Update.Nullobject.Meteo GetMeteo()
		{
			return meteo;
		}

		public virtual bool GetStatus()
		{
			return status;
		}

		public virtual System.DateTime GetUpdateDate()
		{
			return updateDate;
		}

		public virtual NeoDatis.Odb.Test.Update.Nullobject.User GetUser()
		{
			return user;
		}

		public virtual void SetCreationDate(System.DateTime creationDate)
		{
			this.creationDate = creationDate;
		}

		public virtual void SetDeleted(bool deleted)
		{
			this.deleted = deleted;
		}

		public virtual void SetMeteo(NeoDatis.Odb.Test.Update.Nullobject.Meteo meteo)
		{
			this.meteo = meteo;
		}

		public virtual void SetStatus(bool status)
		{
			this.status = status;
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
