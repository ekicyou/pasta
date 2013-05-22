namespace NeoDatis.Odb.Core.Lookup
{
	/// <author>olivier</author>
	public class LookupFactory
	{
		internal static System.Collections.Generic.IDictionary<string, NeoDatis.Odb.Core.Lookup.ILookup
			> lookups = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, NeoDatis.Odb.Core.Lookup.ILookup
			>();

		public static NeoDatis.Odb.Core.Lookup.ILookup Get(string key)
		{
			lock (typeof(LookupFactory))
			{
				NeoDatis.Odb.Core.Lookup.ILookup lookup = lookups[key];
				if (lookup == null)
				{
					lookup = new NeoDatis.Odb.Core.Lookup.LookupImpl();
					lookups.Add(key, lookup);
				}
				return lookup;
			}
		}
	}
}
