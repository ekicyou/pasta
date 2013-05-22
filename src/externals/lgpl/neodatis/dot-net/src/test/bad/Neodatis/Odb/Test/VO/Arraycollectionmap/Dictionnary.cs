namespace NeoDatis.Odb.Test.VO.Arraycollectionmap
{
	public class Dictionnary
	{
		private string name;

		private System.Collections.IDictionary map;

		public Dictionnary() : this("default")
		{
		}

		public Dictionnary(string name)
		{
			this.name = name;
			map = null;
		}

		public virtual void AddEntry(object key, object value)
		{
			if (map == null)
			{
				map = new NeoDatis.Tool.Wrappers.Map.OdbHashMap();
			}
			map.Add(key, value);
		}

		public override string ToString()
		{
			return name + " | " + map;
		}

		public virtual object Get(object key)
		{
			return map[key];
		}

		public virtual void SetMap(System.Collections.IDictionary map)
		{
			this.map = map;
		}

		public virtual System.Collections.IDictionary GetMap()
		{
			return map;
		}

		public virtual string GetName()
		{
			return name;
		}
	}
}
