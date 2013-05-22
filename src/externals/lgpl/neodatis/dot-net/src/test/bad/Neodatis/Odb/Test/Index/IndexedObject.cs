namespace NeoDatis.Odb.Test.Index
{
	[System.Serializable]
	public class IndexedObject : System.IComparable
	{
		private string name;

		private int duration;

		private System.DateTime creation;

		public IndexedObject() : base()
		{
		}

		public IndexedObject(string name, int duration, System.DateTime creation) : base(
			)
		{
			this.name = name;
			this.duration = duration;
			this.creation = creation;
		}

		public virtual System.DateTime GetCreation()
		{
			return creation;
		}

		public virtual void SetCreation(System.DateTime creation)
		{
			this.creation = creation;
		}

		public virtual int GetDuration()
		{
			return duration;
		}

		public virtual void SetDuration(int duration)
		{
			this.duration = duration;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual int CompareTo(object @object)
		{
			if (@object == null || !(@object is NeoDatis.Odb.Test.Index.IndexedObject))
			{
				return -1000;
			}
			NeoDatis.Odb.Test.Index.IndexedObject io = (NeoDatis.Odb.Test.Index.IndexedObject
				)@object;
			return name.CompareTo(io.name);
		}
	}
}
