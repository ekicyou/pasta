using NUnit.Framework;
namespace NeoDatis.Odb.Test.Index
{
	public class IndexedObject2
	{
		private string name;

		private NeoDatis.Odb.Test.Index.IndexedObject @object;

		public IndexedObject2() : base()
		{
		}

		public IndexedObject2(string name, NeoDatis.Odb.Test.Index.IndexedObject @object)
			 : base()
		{
			this.name = name;
			this.@object = @object;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual NeoDatis.Odb.Test.Index.IndexedObject GetObject()
		{
			return @object;
		}

		public virtual void SetObject(NeoDatis.Odb.Test.Index.IndexedObject @object)
		{
			this.@object = @object;
		}
	}
}
