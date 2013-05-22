namespace NeoDatis.Odb.Test.Arraycollectionmap
{
	public class MyObject
	{
		private string name;

		private NeoDatis.Odb.Test.Arraycollectionmap.MyList list;

		public MyObject(string name, NeoDatis.Odb.Test.Arraycollectionmap.MyList list) : 
			base()
		{
			this.name = name;
			this.list = list;
		}

		public virtual NeoDatis.Odb.Test.Arraycollectionmap.MyList GetList()
		{
			return list;
		}

		public virtual void SetList(NeoDatis.Odb.Test.Arraycollectionmap.MyList list)
		{
			this.list = list;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}
	}
}
